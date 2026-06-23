export type StickerKind = "sticker" | "gif";

export interface StickerAsset {
  id: string;
  label: string;
  kind: StickerKind;
  dataUrl: string;
  fileName: string;
  mimeType: string;
}

export interface SavedSticker extends StickerAsset {
  savedAt: string;
}

const SAVED_STICKERS_KEY = "rustchat.savedStickers";
const MAX_SAVED_STICKERS = 24;
const MAX_SAVED_STICKER_BYTES = 1.5 * 1024 * 1024;

export const DEFAULT_EMOJIS = [
  "😀",
  "😂",
  "😊",
  "😍",
  "😎",
  "😭",
  "😡",
  "👍",
  "👏",
  "🙏",
  "🔥",
  "🎉",
  "❤️",
  "✨",
  "🤝",
  "🚀",
];

function svgDataUrl(svg: string) {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
}

function stickerSvg(face: string, bg: string, detail: string) {
  return svgDataUrl(`
    <svg xmlns="http://www.w3.org/2000/svg" width="180" height="180" viewBox="0 0 180 180">
      <rect width="180" height="180" rx="34" fill="${bg}"/>
      <circle cx="90" cy="86" r="54" fill="#fff8dc" stroke="#252f3a" stroke-width="5"/>
      <text x="90" y="104" text-anchor="middle" font-size="54" font-family="Apple Color Emoji, Segoe UI Emoji, sans-serif">${face}</text>
      <text x="90" y="152" text-anchor="middle" fill="#25313d" font-size="18" font-weight="800" font-family="Arial, sans-serif">${detail}</text>
    </svg>
  `);
}

function gifSvg(face: string, bg: string, detail: string) {
  return svgDataUrl(`
    <svg xmlns="http://www.w3.org/2000/svg" width="220" height="160" viewBox="0 0 220 160">
      <rect width="220" height="160" rx="28" fill="${bg}"/>
      <circle cx="78" cy="76" r="44" fill="#fff8dc" stroke="#26323d" stroke-width="5">
        <animate attributeName="cy" values="76;64;76" dur="0.8s" repeatCount="indefinite"/>
      </circle>
      <text x="78" y="94" text-anchor="middle" font-size="44" font-family="Apple Color Emoji, Segoe UI Emoji, sans-serif">
        <animate attributeName="y" values="94;82;94" dur="0.8s" repeatCount="indefinite"/>
        ${face}
      </text>
      <text x="147" y="70" text-anchor="middle" fill="#ffffff" font-size="32" font-weight="900" font-family="Arial, sans-serif">${detail}</text>
      <path d="M124 94h62" stroke="#ffffff" stroke-width="10" stroke-linecap="round">
        <animate attributeName="stroke-dasharray" values="8 18;24 10;8 18" dur="0.7s" repeatCount="indefinite"/>
      </path>
    </svg>
  `);
}

export const DEFAULT_STICKERS: StickerAsset[] = [
  {
    id: "sticker-ok",
    label: "OK",
    kind: "sticker",
    dataUrl: stickerSvg("👌", "#dff7ea", "OK"),
    fileName: "rustchat-ok.svg",
    mimeType: "image/svg+xml",
  },
  {
    id: "sticker-laugh",
    label: "Laugh",
    kind: "sticker",
    dataUrl: stickerSvg("😂", "#e9f1ff", "LOL"),
    fileName: "rustchat-laugh.svg",
    mimeType: "image/svg+xml",
  },
  {
    id: "sticker-work",
    label: "Ship",
    kind: "sticker",
    dataUrl: stickerSvg("🚀", "#f3ecff", "SHIP"),
    fileName: "rustchat-ship.svg",
    mimeType: "image/svg+xml",
  },
  {
    id: "sticker-heart",
    label: "Nice",
    kind: "sticker",
    dataUrl: stickerSvg("💚", "#fff3dd", "NICE"),
    fileName: "rustchat-nice.svg",
    mimeType: "image/svg+xml",
  },
];

export const DEFAULT_GIFS: StickerAsset[] = [
  {
    id: "gif-wave",
    label: "Wave",
    kind: "gif",
    dataUrl: gifSvg("👋", "#00a35c", "HI"),
    fileName: "rustchat-wave.svg",
    mimeType: "image/svg+xml",
  },
  {
    id: "gif-hype",
    label: "Hype",
    kind: "gif",
    dataUrl: gifSvg("🔥", "#2457a6", "GO"),
    fileName: "rustchat-hype.svg",
    mimeType: "image/svg+xml",
  },
  {
    id: "gif-done",
    label: "Done",
    kind: "gif",
    dataUrl: gifSvg("✅", "#6c4ab6", "DONE"),
    fileName: "rustchat-done.svg",
    mimeType: "image/svg+xml",
  },
];

export function readSavedStickers(): SavedSticker[] {
  const raw = localStorage.getItem(SAVED_STICKERS_KEY);

  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];

    return parsed.filter(isSavedSticker);
  } catch {
    localStorage.removeItem(SAVED_STICKERS_KEY);
    return [];
  }
}

export function persistSavedStickers(stickers: SavedSticker[]) {
  localStorage.setItem(
    SAVED_STICKERS_KEY,
    JSON.stringify(stickers.slice(0, MAX_SAVED_STICKERS)),
  );
  window.dispatchEvent(new CustomEvent("rustchat:saved-stickers-changed"));
}

export function removeSavedSticker(id: string) {
  persistSavedStickers(readSavedStickers().filter((sticker) => sticker.id !== id));
}

export async function saveBlobAsSticker(
  blob: Blob,
  label: string,
  fileName: string,
): Promise<SavedSticker> {
  if (blob.size > MAX_SAVED_STICKER_BYTES) {
    throw new Error("表情太大，保存到本地的表情不能超过 1.5MB");
  }

  const dataUrl = await blobToDataUrl(blob);
  const mimeType = blob.type || "application/octet-stream";
  const saved: SavedSticker = {
    id: `saved-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    label: label.slice(0, 24) || "Saved",
    kind: mimeType.includes("gif") ? "gif" : "sticker",
    dataUrl,
    fileName,
    mimeType,
    savedAt: new Date().toISOString(),
  };
  const existing = readSavedStickers();

  persistSavedStickers([
    saved,
    ...existing.filter((sticker) => sticker.dataUrl !== saved.dataUrl),
  ]);

  return saved;
}

export async function dataUrlToFile(asset: StickerAsset): Promise<File> {
  const response = await fetch(asset.dataUrl);
  const blob = await response.blob();
  return new File([blob], asset.fileName, {
    type: asset.mimeType || blob.type || "application/octet-stream",
  });
}

function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("表情读取失败"));
    reader.readAsDataURL(blob);
  });
}

function isSavedSticker(value: unknown): value is SavedSticker {
  if (!value || typeof value !== "object") {
    return false;
  }

  const sticker = value as Record<string, unknown>;

  return (
    typeof sticker.id === "string" &&
    typeof sticker.label === "string" &&
    (sticker.kind === "sticker" || sticker.kind === "gif") &&
    typeof sticker.dataUrl === "string" &&
    sticker.dataUrl.startsWith("data:") &&
    typeof sticker.fileName === "string" &&
    typeof sticker.mimeType === "string" &&
    typeof sticker.savedAt === "string"
  );
}
