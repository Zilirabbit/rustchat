<template>
  <section ref="listEl" class="message-list">
    <div v-if="loading" class="message-state">Loading history...</div>
    <div v-else-if="messages.length === 0" class="message-state">
      No messages yet.
    </div>

    <template v-else>
      <article
        v-for="message in messages"
        :key="message.message_id"
        class="message-row"
        :class="{
          mine: message.sender_id === currentUserId,
          failed: message.send_status === 'failed',
        }"
      >
        <span class="message-avatar" :class="message.sender_id === currentUserId ? 'avatar-current' : 'avatar-neutral'">
          {{ avatarLabel(message.sender_username) }}
        </span>
        <div class="message-bubble">
          <div class="message-meta">
            <span class="message-author">{{ message.sender_username }}</span>
            <time>{{ formatTime(message.created_at) }}</time>
          </div>
          <div v-if="message.message_type === 'file' && message.file_id" class="file-attachment">
            <div class="file-icon">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                <polyline points="14 2 14 8 20 8" />
                <line x1="16" y1="13" x2="8" y2="13" />
                <line x1="16" y1="17" x2="8" y2="17" />
              </svg>
            </div>
            <div class="file-info">
              <span class="file-name">{{ message.file_name || message.content }}</span>
              <span class="file-size">{{ formatFileSize(message.file_size || 0) }}</span>
            </div>
            <button
              class="file-download-btn"
              type="button"
              :title="'Download ' + (message.file_name || message.content)"
              @click="downloadFile(message.file_id, message.file_name || message.content)"
            >
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
            </button>
          </div>
          <p v-else>{{ message.content }}</p>
          <div
            v-if="message.sender_id === currentUserId && message.send_status"
            class="message-send-state"
          >
            <span>{{ sendStatusLabel(message) }}</span>
            <button
              v-if="message.send_status === 'failed' && message.client_message_id"
              type="button"
              @click="emit('retry', message.client_message_id)"
            >
              重试
            </button>
          </div>
        </div>
      </article>
    </template>
  </section>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import type { MessageListItem } from "../types/chat";
import { formatFileSize } from "../api/files";
import { http } from "../api/http";

const props = defineProps<{
  messages: MessageListItem[];
  currentUserId: number;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (event: "retry", clientMessageId: string): void;
  (event: "downloadError", message: string): void;
}>();

const listEl = ref<HTMLElement | null>(null);

watch(
  () => [props.messages.length, props.loading],
  async () => {
    await nextTick();
    if (listEl.value) {
      listEl.value.scrollTop = listEl.value.scrollHeight;
    }
  },
  { immediate: true },
);

function avatarLabel(username: string) {
  return username.slice(0, 1).toUpperCase() || "?";
}

function formatTime(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return "";
  }

  return date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function sendStatusLabel(message: MessageListItem) {
  if (message.send_status === "queued") {
    return message.send_error || "等待重连";
  }

  if (message.send_status === "sending") {
    return "发送中";
  }

  if (message.send_status === "failed") {
    return message.send_error || "发送失败";
  }

  return "已发送";
}

async function downloadFile(fileId: number | null | undefined, fileName: string) {
  if (!fileId) return;

  try {
    const response = await http.get(`/api/files/${fileId}/download`, {
      responseType: "blob",
    });

    const url = window.URL.createObjectURL(new Blob([response.data]));
    const link = document.createElement("a");
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);
  } catch (error) {
    emit("downloadError", "文件下载失败，请稍后重试");
  }
}
</script>
