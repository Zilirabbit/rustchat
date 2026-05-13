<template>
  <form class="message-input composer" @submit.prevent="submit">
    <button class="composer-tool" type="button" aria-label="Add attachment">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 5v14M5 12h14" />
      </svg>
    </button>
    <input
      ref="fileInputRef"
      type="file"
      class="file-input-hidden"
      @change="onFileSelected"
    />
    <button class="composer-tool" type="button" aria-label="Attach file" @click="triggerFilePicker">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="m21.4 11.6-8.5 8.5a5 5 0 0 1-7.1-7.1l9.2-9.2a3.4 3.4 0 0 1 4.8 4.8l-9.2 9.2a1.8 1.8 0 0 1-2.5-2.5l8.5-8.5" />
      </svg>
    </button>
    <textarea
      v-model="content"
      class="message-textarea"
      rows="1"
      placeholder="Type a message"
      :disabled="disabled"
      @keydown.enter.exact.prevent="submit"
    />
    <div v-if="uploadProgress !== null" class="upload-progress-bar">
      <div class="upload-progress-fill" :style="{ width: uploadPercent + '%' }"></div>
      <span class="upload-progress-text">{{ uploadFileName }} ({{ uploadPercent }}%)</span>
    </div>
    <button class="composer-tool optional" type="button" aria-label="Add emoji">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="10" />
        <path d="M8 14s1.5 2 4 2 4-2 4-2M9 9h.01M15 9h.01" />
      </svg>
    </button>
    <button class="composer-tool optional" type="button" aria-label="Add GIF">
      <span>GIF</span>
    </button>
    <button class="send-button" type="submit" :disabled="disabled || !content.trim()">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="m22 2-7 20-4-9-9-4Z" />
        <path d="M22 2 11 13" />
      </svg>
      <span>Send</span>
    </button>
  </form>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{
  disabled?: boolean;
  uploading?: boolean;
  uploadFileName?: string;
  uploadProgress?: number | null;
}>();

const emit = defineEmits<{
  (event: "send", content: string): void;
  (event: "fileSelected", file: File): void;
}>();

const content = ref("");
const fileInputRef = ref<HTMLInputElement | null>(null);

const uploadPercent = computed(() => {
  if (props.uploadProgress === null || props.uploadProgress === undefined) return 0;
  return Math.min(100, Math.round(props.uploadProgress));
});

function triggerFilePicker() {
  fileInputRef.value?.click();
}

function onFileSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  emit("fileSelected", file);
  input.value = "";
}

function submit() {
  const trimmedContent = content.value.trim();

  if (!trimmedContent) {
    return;
  }

  emit("send", trimmedContent);
  content.value = "";
}
</script>
