import { http, unwrapResponse, getApiErrorMessage } from "./http";
import type { ApiResponse } from "../types/api";

const CHUNK_SIZE = 512 * 1024; // 512KB

export interface InitUploadResponse {
  upload_id: string;
}

export interface CompleteUploadResponse {
  file_id: number;
  message_id: number;
  file_name: string;
  file_size: number;
  file_type: string;
}

async function computeSHA256(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  const hashBuffer = await crypto.subtle.digest("SHA-256", buffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const size = (bytes / Math.pow(k, i)).toFixed(i > 0 ? 1 : 0);
  return `${size} ${units[i]}`;
}

export { formatFileSize };

export async function uploadFile(
  sessionId: number,
  file: File,
  onProgress?: (progress: number, total: number) => void,
): Promise<CompleteUploadResponse> {
  // 1. Compute hash
  const fileHash = await computeSHA256(file);

  // 2. Init upload
  const totalChunks = Math.max(1, Math.ceil(file.size / CHUNK_SIZE));
  const fileType = file.type || "application/octet-stream";

  const initRes = await http
    .post<ApiResponse<InitUploadResponse>>(
      "/api/files/init",
      {
        session_id: sessionId,
        file_name: file.name,
        file_size: file.size,
        file_type: fileType,
        total_chunks: totalChunks,
      },
    )
    .then(unwrapResponse);

  const uploadId = initRes.upload_id;

  // 3. Upload chunks
  for (let i = 0; i < totalChunks; i++) {
    const start = i * CHUNK_SIZE;
    const end = Math.min(start + CHUNK_SIZE, file.size);
    const chunk = file.slice(start, end);

    await http.post(`/api/files/${uploadId}/chunk?index=${i}`, chunk, {
      headers: { "Content-Type": "application/octet-stream" },
    });

    if (onProgress) {
      onProgress(i + 1, totalChunks);
    }
  }

  // 4. Complete upload with hash verification
  const completeRes = await http
    .post<ApiResponse<CompleteUploadResponse>>(
      `/api/files/${uploadId}/complete`,
      { file_hash: fileHash },
    )
    .then(unwrapResponse);

  return completeRes;
}

export function getFileDownloadUrl(fileId: number): string {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:3000";
  return `${baseUrl}/api/files/${fileId}/download`;
}
