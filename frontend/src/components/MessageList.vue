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
        :class="{ mine: message.sender_id === currentUserId }"
      >
        <span class="message-avatar" :class="message.sender_id === currentUserId ? 'avatar-current' : 'avatar-neutral'">
          {{ avatarLabel(message.sender_username) }}
        </span>
        <div class="message-bubble">
          <div class="message-meta">
            <span class="message-author">{{ message.sender_username }}</span>
            <time>{{ formatTime(message.created_at) }}</time>
          </div>
          <p>{{ message.content }}</p>
        </div>
      </article>
    </template>
  </section>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import type { MessageListItem } from "../types/chat";

const props = defineProps<{
  messages: MessageListItem[];
  currentUserId: number;
  loading?: boolean;
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
</script>
