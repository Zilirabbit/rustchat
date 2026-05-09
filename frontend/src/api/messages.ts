import { http } from "./http";

export function listMessages(conversationId: string) {
  return http.get(`/conversations/${conversationId}/messages`);
}
