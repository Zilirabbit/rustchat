import type { ApiResponse } from "../types/api";
import type {
  CreatePrivateSessionResponse,
  MarkSessionReadResponse,
} from "../types/chat";
import { http, unwrapResponse } from "./http";

export function createPrivateSession(targetUserId: number) {
  return http
    .post<ApiResponse<CreatePrivateSessionResponse>>("/api/sessions/private", {
      target_user_id: targetUserId,
    })
    .then(unwrapResponse);
}

export function markSessionRead(sessionId: number) {
  return http
    .post<ApiResponse<MarkSessionReadResponse>>(
      `/api/sessions/${sessionId}/read`,
    )
    .then(unwrapResponse);
}
