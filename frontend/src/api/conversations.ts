import type { ApiResponse } from "../types/api";
import type { ConversationItem } from "../types/chat";
import { http, unwrapResponse } from "./http";

export function listConversations() {
  return http
    .get<ApiResponse<ConversationItem[]>>("/api/conversations")
    .then(unwrapResponse);
}
