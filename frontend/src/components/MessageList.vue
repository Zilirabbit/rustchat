<template>
  <section ref="listEl" class="message-list">
    <div v-if="loading" class="message-state">正在加载历史消息...</div>
    <div v-else-if="messages.length === 0" class="message-state">
      这里还没有消息
    </div>

    <article
      v-for="message in messages"
      :key="message.message_id"
      class="message-row"
      :class="{ mine: message.sender_id === currentUserId }"
    >
      <div class="message-bubble">
        <div class="message-meta">
          <span>{{ message.sender_username }}</span>
          <time>{{ formatTime(message.created_at) }}</time>
        </div>
        <p>{{ message.content }}</p>
      </div>
    </article>
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
