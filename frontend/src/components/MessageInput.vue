<template>
  <form class="message-input composer" @submit.prevent="submit">
    <input
      ref="fileInputRef"
      type="file"
      class="file-input-hidden"
      :disabled="disabled || uploading || isRecording"
      @change="onFileSelected"
    />
    <input
      ref="stickerInputRef"
      type="file"
      class="file-input-hidden"
      accept="image/*,.gif"
      :disabled="disabled || uploading || isRecording"
      @change="onStickerFileSelected"
    />
    <button
      class="composer-tool"
      type="button"
      aria-label="Attach file"
      :disabled="disabled || uploading || isRecording"
      @click="triggerFilePicker"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="m21.4 11.6-8.5 8.5a5 5 0 0 1-7.1-7.1l9.2-9.2a3.4 3.4 0 0 1 4.8 4.8l-9.2 9.2a1.8 1.8 0 0 1-2.5-2.5l8.5-8.5" />
      </svg>
    </button>
    <textarea
      ref="textareaRef"
      v-model="content"
      class="message-textarea"
      rows="1"
      placeholder="Type a message"
      :disabled="disabled || isRecording"
      @keydown.enter.exact.prevent="submit"
    />
    <div v-if="uploadProgress !== null" class="upload-progress-bar">
      <div class="upload-progress-fill" :style="{ width: uploadPercent + '%' }"></div>
      <span class="upload-progress-text">{{ uploadFileName }} ({{ uploadPercent }}%)</span>
    </div>
    <button
      class="composer-tool optional"
      type="button"
      aria-label="Add emoji"
      :class="{ active: activePanel === 'emoji' }"
      :disabled="disabled || isRecording"
      @click="togglePanel('emoji')"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="10" />
        <path d="M8 14s1.5 2 4 2 4-2 4-2M9 9h.01M15 9h.01" />
      </svg>
    </button>
    <button
      class="composer-tool optional"
      type="button"
      aria-label="Add GIF"
      :class="{ active: activePanel === 'gif' }"
      :disabled="disabled || uploading || isRecording"
      @click="togglePanel('gif')"
    >
      <span>GIF</span>
    </button>
    <button
      class="composer-tool voice-button"
      type="button"
      :aria-label="isRecording ? 'Stop voice recording' : 'Record voice message'"
      :class="{ active: isRecording }"
      :disabled="disabled || uploading || isStartingRecording"
      @click="toggleRecording"
    >
      <svg v-if="!isRecording" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
        <path d="M12 19v3" />
      </svg>
      <svg v-else viewBox="0 0 24 24" aria-hidden="true">
        <rect x="7" y="7" width="10" height="10" rx="2" />
      </svg>
    </button>
    <button class="send-button" type="submit" :disabled="disabled || isRecording || !content.trim()">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="m22 2-7 20-4-9-9-4Z" />
        <path d="M22 2 11 13" />
      </svg>
      <span>Send</span>
    </button>

    <div v-if="isRecording || isStartingRecording" class="voice-recording-bar">
      <span class="recording-dot"></span>
      <span>{{ isRecording ? `Recording ${recordingTimeLabel}` : "Preparing microphone..." }}</span>
      <button type="button" :disabled="isStartingRecording" @click="cancelRecording">Cancel</button>
    </div>

    <section v-if="activePanel" class="sticker-popover" :aria-label="panelTitle">
      <div class="sticker-popover-header">
        <strong>{{ panelTitle }}</strong>
        <button type="button" aria-label="Close picker" @click="activePanel = null">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div v-if="activePanel === 'emoji'" class="emoji-grid">
        <button
          v-for="emoji in defaultEmojis"
          :key="emoji"
          type="button"
          @click="insertEmoji(emoji)"
        >
          {{ emoji }}
        </button>
      </div>

      <template v-else>
        <div class="sticker-section">
          <div class="sticker-section-title">
            <span>Default GIFs</span>
            <button
              type="button"
              :disabled="uploading"
              @click="triggerStickerPicker"
            >
              Upload GIF
            </button>
          </div>
          <div class="sticker-grid">
            <button
              v-for="asset in defaultPanelAssets"
              :key="asset.id"
              class="sticker-tile"
              type="button"
              :title="asset.label"
              :disabled="uploading"
              @click="sendAsset(asset)"
            >
              <img :src="asset.dataUrl" :alt="asset.label" />
            </button>
          </div>
        </div>

        <div class="sticker-section">
          <div class="sticker-section-title">
            <span>Saved</span>
            <button type="button" :disabled="uploading" @click="triggerStickerPicker">
              Add
            </button>
          </div>
          <p v-if="savedStickers.length === 0" class="sticker-empty">
            Save an image or GIF from a message to reuse it here.
          </p>
          <div v-else class="sticker-grid">
            <button
              v-for="asset in filteredSavedStickers"
              :key="asset.id"
              class="sticker-tile saved"
              type="button"
              :title="asset.label"
              :disabled="uploading"
              @click="sendAsset(asset)"
            >
              <img :src="asset.dataUrl" :alt="asset.label" />
            </button>
          </div>
        </div>
      </template>
    </section>
  </form>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
  DEFAULT_EMOJIS,
  DEFAULT_GIFS,
  DEFAULT_STICKERS,
  dataUrlToFile,
  readSavedStickers,
  saveBlobAsSticker,
  type SavedSticker,
  type StickerAsset,
} from "../utils/stickers";

const props = defineProps<{
  disabled?: boolean;
  uploading?: boolean;
  uploadFileName?: string;
  uploadProgress?: number | null;
}>();

const emit = defineEmits<{
  (event: "send", content: string): void;
  (event: "fileSelected", file: File): void;
  (event: "recordingError", message: string): void;
}>();

const content = ref("");
const fileInputRef = ref<HTMLInputElement | null>(null);
const stickerInputRef = ref<HTMLInputElement | null>(null);
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const activePanel = ref<"emoji" | "gif" | null>(null);
const savedStickers = ref<SavedSticker[]>(readSavedStickers());
const defaultEmojis = DEFAULT_EMOJIS;
const isRecording = ref(false);
const isStartingRecording = ref(false);
const recordingSeconds = ref(0);
let mediaRecorder: MediaRecorder | null = null;
let recordingStream: MediaStream | null = null;
let recordingTimer: number | null = null;
let recordedChunks: Blob[] = [];
let discardRecording = false;
let activeRecordingMimeType = "";

const uploadPercent = computed(() => {
  if (props.uploadProgress === null || props.uploadProgress === undefined) return 0;
  return Math.min(100, Math.round(props.uploadProgress));
});
const recordingTimeLabel = computed(() => {
  const minutes = Math.floor(recordingSeconds.value / 60)
    .toString()
    .padStart(2, "0");
  const seconds = (recordingSeconds.value % 60).toString().padStart(2, "0");
  return `${minutes}:${seconds}`;
});
const panelTitle = computed(() => {
  if (activePanel.value === "gif") return "GIFs";
  return "Emoji";
});
const defaultPanelAssets = computed(() => [...DEFAULT_GIFS, ...DEFAULT_STICKERS]);
const filteredSavedStickers = computed(() => {
  return savedStickers.value;
});

onMounted(() => {
  window.addEventListener("rustchat:saved-stickers-changed", syncSavedStickers);
});

onBeforeUnmount(() => {
  window.removeEventListener("rustchat:saved-stickers-changed", syncSavedStickers);
  stopRecording(true);
});

function syncSavedStickers() {
  savedStickers.value = readSavedStickers();
}

function triggerFilePicker() {
  if (props.disabled || props.uploading || isRecording.value) {
    return;
  }

  fileInputRef.value?.click();
}

function triggerStickerPicker() {
  if (props.disabled || props.uploading || isRecording.value) {
    return;
  }

  stickerInputRef.value?.click();
}

function onFileSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  emit("fileSelected", file);
  input.value = "";
}

async function onStickerFileSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  if (!file.type.startsWith("image/") && !file.name.toLowerCase().endsWith(".gif")) {
    input.value = "";
    return;
  }

  emit("fileSelected", file);

  try {
    const saved = await saveBlobAsSticker(file, file.name.replace(/\.[^.]+$/, ""), file.name);
    savedStickers.value = [saved, ...savedStickers.value.filter((item) => item.id !== saved.id)];
  } catch {
    // Sending still succeeds even when local saving is skipped because the file is too large.
  }

  activePanel.value = null;
  input.value = "";
}

function togglePanel(panel: "emoji" | "gif") {
  activePanel.value = activePanel.value === panel ? null : panel;
}

function insertEmoji(emoji: string) {
  const textarea = textareaRef.value;

  if (!textarea) {
    content.value += emoji;
    return;
  }

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  content.value = `${content.value.slice(0, start)}${emoji}${content.value.slice(end)}`;

  requestAnimationFrame(() => {
    textarea.focus();
    textarea.selectionStart = start + emoji.length;
    textarea.selectionEnd = start + emoji.length;
  });
}

async function sendAsset(asset: StickerAsset) {
  if (props.disabled || props.uploading || isRecording.value) {
    return;
  }

  const file = await dataUrlToFile(asset);
  emit("fileSelected", file);
  activePanel.value = null;
}

async function toggleRecording() {
  if (isRecording.value) {
    stopRecording(false);
    return;
  }

  await startRecording();
}

async function startRecording() {
  if (props.disabled || props.uploading || isRecording.value || isStartingRecording.value) {
    return;
  }

  if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === "undefined") {
    emit("recordingError", "当前浏览器不支持录音");
    return;
  }

  try {
    isStartingRecording.value = true;
    activePanel.value = null;
    recordedChunks = [];
    discardRecording = false;
    activeRecordingMimeType = "";
    recordingSeconds.value = 0;
    recordingStream = await navigator.mediaDevices.getUserMedia({ audio: true });
    const mimeType = preferredAudioMimeType();
    activeRecordingMimeType = cleanAudioMimeType(mimeType || "audio/webm");
    mediaRecorder = new MediaRecorder(
      recordingStream,
      mimeType ? { mimeType } : undefined,
    );

    mediaRecorder.addEventListener("dataavailable", (event) => {
      if (event.data.size > 0) {
        recordedChunks.push(event.data);
      }
    });
    mediaRecorder.addEventListener("stop", finishRecording);
    mediaRecorder.addEventListener("error", () => {
      emit("recordingError", "录音失败，请稍后重试");
      stopRecording(true);
    });
    mediaRecorder.start();
    isRecording.value = true;
    isStartingRecording.value = false;
    recordingTimer = window.setInterval(() => {
      recordingSeconds.value += 1;
    }, 1000);
  } catch {
    cleanupRecording();
    emit("recordingError", "无法访问麦克风，请检查浏览器权限");
  }
}

function stopRecording(discard: boolean) {
  discardRecording = discard;

  if (mediaRecorder && mediaRecorder.state !== "inactive") {
    mediaRecorder.stop();
    return;
  }

  cleanupRecording();
}

function cancelRecording() {
  stopRecording(true);
}

function finishRecording() {
  const mimeType = cleanAudioMimeType(mediaRecorder?.mimeType || activeRecordingMimeType || "audio/webm");
  const chunks = [...recordedChunks];
  const shouldDiscard = discardRecording;
  cleanupRecording();

  if (shouldDiscard || chunks.length === 0) {
    return;
  }

  const blob = new Blob(chunks, { type: mimeType });

  if (blob.size === 0) {
    emit("recordingError", "录音内容为空，请重新录制");
    return;
  }

  const extension = audioFileExtension(mimeType);
  const file = new File([blob], `voice-message-${Date.now()}.${extension}`, {
    type: mimeType,
  });
  emit("fileSelected", file);
}

function cleanupRecording() {
  if (recordingTimer !== null) {
    window.clearInterval(recordingTimer);
    recordingTimer = null;
  }

  recordingStream?.getTracks().forEach((track) => track.stop());
  recordingStream = null;
  mediaRecorder = null;
  recordedChunks = [];
  discardRecording = false;
  activeRecordingMimeType = "";
  isRecording.value = false;
  isStartingRecording.value = false;
  recordingSeconds.value = 0;
}

function preferredAudioMimeType() {
  const candidates = [
    "audio/webm;codecs=opus",
    "audio/webm",
    "audio/mp4",
    "audio/ogg;codecs=opus",
  ];

  return candidates.find((type) => MediaRecorder.isTypeSupported(type)) || "";
}

function audioFileExtension(mimeType: string) {
  if (mimeType.includes("mp4")) return "m4a";
  if (mimeType.includes("ogg")) return "ogg";
  if (mimeType.includes("mpeg")) return "mp3";
  if (mimeType.includes("wav")) return "wav";
  return "webm";
}

function cleanAudioMimeType(mimeType: string) {
  return mimeType.split(";")[0]?.trim() || "audio/webm";
}

function submit() {
  if (isRecording.value) {
    return;
  }

  const trimmedContent = content.value.trim();

  if (!trimmedContent) {
    return;
  }

  emit("send", trimmedContent);
  content.value = "";
}
</script>
