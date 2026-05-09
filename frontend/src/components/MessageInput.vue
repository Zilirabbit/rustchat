<template>
  <form class="message-input" @submit.prevent="submit">
    <textarea
      v-model="content"
      class="message-textarea"
      rows="2"
      placeholder="输入消息"
      :disabled="disabled"
      @keydown.enter.exact.prevent="submit"
    />
    <button class="send-button" type="submit" :disabled="disabled || !content.trim()">
      发送
    </button>
  </form>
</template>

<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (event: "send", content: string): void;
}>();

const content = ref("");

function submit() {
  const trimmedContent = content.value.trim();

  if (!trimmedContent) {
    return;
  }

  emit("send", trimmedContent);
  content.value = "";
}
</script>
