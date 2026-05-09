export function createChatSocket(token: string) {
  const baseUrl = import.meta.env.VITE_WS_BASE_URL || "ws://127.0.0.1:3000";
  return new WebSocket(`${baseUrl}/ws?token=${encodeURIComponent(token)}`);
}
