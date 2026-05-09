import { http } from "./http";

export function listConversations() {
  return http.get("/conversations");
}
