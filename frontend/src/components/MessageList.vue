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
          <div
            v-if="isImageAttachment(message)"
            class="image-attachment"
            :class="{ loading: !previewUrl(message.file_id) }"
          >
            <button
              class="image-preview-button"
              type="button"
              :title="'Download ' + (message.file_name || message.content)"
              @click="downloadFile(message.file_id, message.file_name || message.content)"
            >
              <img
                v-if="previewUrl(message.file_id)"
                :src="previewUrl(message.file_id)"
                :alt="message.file_name || message.content"
              />
              <span v-else>Loading preview...</span>
            </button>
            <div class="image-attachment-actions">
              <span>{{ message.file_name || "Sticker" }}</span>
              <button
                type="button"
                :disabled="!message.file_id"
                @click="saveSticker(message)"
              >
                Save
              </button>
              <button
                type="button"
                :disabled="!message.file_id"
                @click="downloadFile(message.file_id, message.file_name || message.content)"
              >
                Download
              </button>
            </div>
          </div>
          <div
            v-else-if="isAudioAttachment(message)"
            class="audio-attachment"
            :class="{ loading: !previewUrl(message.file_id) }"
          >
            <div class="audio-icon">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <path d="M12 19v3" />
              </svg>
            </div>
            <div class="audio-info">
              <span class="file-name">{{ message.file_name || "Voice message" }}</span>
              <audio
                v-if="previewUrl(message.file_id)"
                :src="previewUrl(message.file_id)"
                controls
                preload="metadata"
                @error="emit('downloadError', '语音加载失败，请稍后重试')"
              />
              <span v-else class="audio-loading">Loading audio...</span>
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
          <div v-else-if="message.message_type === 'file' && message.file_id" class="file-attachment">
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
import { nextTick, onBeforeUnmount, reactive, ref, watch } from "vue";
import type { MessageListItem } from "../types/chat";
import { formatFileSize } from "../api/files";
import { http } from "../api/http";
import { saveBlobAsSticker } from "../utils/stickers";

const props = defineProps<{
  messages: MessageListItem[];
  currentUserId: number;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (event: "retry", clientMessageId: string): void;
  (event: "downloadError", message: string): void;
  (event: "stickerSaved", message: string): void;
}>();

const listEl = ref<HTMLElement | null>(null);
const previewUrls = reactive<Record<number, string>>({});
const previewBlobs = new Map<number, Blob>();
const loadingPreviewFileIds = new Set<number>();

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

watch(
  () => props.messages.map((message) => `${message.file_id || ""}:${message.file_type || ""}`),
  () => {
    const activePreviewFileIds = new Set(
      props.messages
        .filter((message) => isImageAttachment(message) || isAudioAttachment(message))
        .map((message) => message.file_id)
        .filter((fileId): fileId is number => Boolean(fileId)),
    );

    Object.keys(previewUrls).forEach((fileIdKey) => {
      const fileId = Number(fileIdKey);

      if (!activePreviewFileIds.has(fileId)) {
        URL.revokeObjectURL(previewUrls[fileId]);
        delete previewUrls[fileId];
        previewBlobs.delete(fileId);
      }
    });

    props.messages
      .filter((message) => isImageAttachment(message) || isAudioAttachment(message))
      .forEach((message) => {
        if (message.file_id) {
          void ensurePreview(message.file_id);
        }
      });
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  Object.values(previewUrls).forEach((url) => URL.revokeObjectURL(url));
});

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
    const blob = await fetchFileBlob(fileId);
    const url = window.URL.createObjectURL(blob);
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

function isImageAttachment(message: MessageListItem) {
  return (
    message.message_type === "file" &&
    Boolean(message.file_id) &&
    Boolean(message.file_type?.startsWith("image/"))
  );
}

function isAudioAttachment(message: MessageListItem) {
  return (
    message.message_type === "file" &&
    Boolean(message.file_id) &&
    (Boolean(message.file_type?.startsWith("audio/")) ||
      isAudioFileName(message.file_name || message.content))
  );
}

function isAudioFileName(fileName: string | null | undefined) {
  return /\.(webm|m4a|mp3|wav|ogg|oga)$/i.test(fileName || "");
}

function previewUrl(fileId: number | null | undefined) {
  return fileId ? previewUrls[fileId] : "";
}

async function ensurePreview(fileId: number) {
  if (previewUrls[fileId] || loadingPreviewFileIds.has(fileId)) {
    return;
  }

  loadingPreviewFileIds.add(fileId);

  try {
    const blob = await fetchFileBlob(fileId);
    previewUrls[fileId] = URL.createObjectURL(blob);
  } catch {
    emit("downloadError", "媒体加载失败，请稍后重试");
  } finally {
    loadingPreviewFileIds.delete(fileId);
  }
}

async function fetchFileBlob(fileId: number) {
  const cached = previewBlobs.get(fileId);

  if (cached) {
    return cached;
  }

  const response = await http.get(`/api/files/${fileId}/download`, {
    responseType: "blob",
  });
  const contentType = response.headers["content-type"];
  const blob = new Blob([response.data], {
    type: typeof contentType === "string" ? contentType : "application/octet-stream",
  });
  previewBlobs.set(fileId, blob);

  return blob;
}

async function saveSticker(message: MessageListItem) {
  if (!message.file_id) {
    return;
  }

  try {
    const blob = await fetchFileBlob(message.file_id);
    await saveBlobAsSticker(
      blob,
      message.file_name || message.content || "Saved sticker",
      message.file_name || `rustchat-sticker-${message.file_id}`,
    );
    emit("stickerSaved", "表情已保存，可以在输入框的 GIF 面板里复用");
  } catch (error) {
    emit("downloadError", error instanceof Error ? error.message : "表情保存失败");
  }
}
</script>
