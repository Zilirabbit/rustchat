import type { ApiResponse } from "../types/api";
import type { MessageListPage } from "../types/chat";
import { http, unwrapResponse } from "./http";

export function listMessages(sessionId: number, limit = 20) {
  return http
    .get<ApiResponse<MessageListPage>>("/api/messages", {
      params: {
        session_id: sessionId,
        limit,
      },
    })
    .then(unwrapResponse);
}
