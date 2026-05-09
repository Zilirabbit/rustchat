export function createChatSocket(token: string) {
  const baseUrl = import.meta.env.VITE_WS_BASE_URL;
  return new WebSocket(`${baseUrl}/ws?token=${encodeURIComponent(token)}`);
}
