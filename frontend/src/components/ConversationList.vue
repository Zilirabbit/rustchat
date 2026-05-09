<template>
  <section class="conversation-list">
    <div class="section-header">
      <h2>会话</h2>
      <span v-if="loading" class="muted-line">加载中...</span>
    </div>

    <div v-if="!loading && conversations.length === 0" class="empty-list">
      搜索用户并开始私聊
    </div>

    <button
      v-for="conversation in conversations"
      :key="conversation.session_id"
      class="conversation-item"
      :class="{ active: conversation.session_id === activeSessionId }"
      type="button"
      @click="$emit('select', conversation.session_id)"
    >
      <span class="avatar">{{ conversation.session_name.slice(0, 1).toUpperCase() }}</span>
      <span class="conversation-body">
        <span class="conversation-title-row">
          <strong>{{ conversation.session_name }}</strong>
          <time>{{ formatTime(conversation.last_message_time) }}</time>
        </span>
        <span class="conversation-preview">
          {{ conversation.last_message || "还没有消息" }}
        </span>
      </span>
      <span v-if="conversation.unread_count > 0" class="badge">
        {{ conversation.unread_count }}
      </span>
    </button>
  </section>
</template>

<script setup lang="ts">
import type { ConversationItem } from "../types/chat";

defineProps<{
  conversations: ConversationItem[];
  activeSessionId: number | null;
  loading?: boolean;
}>();

defineEmits<{
  (event: "select", sessionId: number): void;
}>();

function formatTime(value: string | null) {
  if (!value) {
    return "";
  }

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
