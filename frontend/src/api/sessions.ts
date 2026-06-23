import type { ApiResponse } from "../types/api";
import type {
  AddGroupMemberResponse,
  CreateGroupSessionResponse,
  CreatePrivateSessionResponse,
  LeaveGroupSessionResponse,
  ListGroupMembersResponse,
  MarkSessionReadResponse,
  RemoveGroupMemberResponse,
} from "../types/chat";
import { http, unwrapResponse } from "./http";

export function createPrivateSession(targetUserId: number) {
  return http
    .post<ApiResponse<CreatePrivateSessionResponse>>("/api/sessions/private", {
      target_user_id: targetUserId,
    })
    .then(unwrapResponse);
}

export function createGroupSession(name: string, memberUserIds: number[]) {
  return http
    .post<ApiResponse<CreateGroupSessionResponse>>("/api/sessions/group", {
      name,
      member_user_ids: memberUserIds,
    })
    .then(unwrapResponse);
}

export function addGroupMember(sessionId: number, userId: number) {
  return http
    .post<ApiResponse<AddGroupMemberResponse>>(
      `/api/sessions/${sessionId}/members`,
      {
        user_id: userId,
      },
    )
    .then(unwrapResponse);
}

export function listGroupMembers(sessionId: number) {
  return http
    .get<ApiResponse<ListGroupMembersResponse>>(
      `/api/sessions/${sessionId}/members`,
    )
    .then(unwrapResponse);
}

export function leaveGroupSession(sessionId: number) {
  return http
    .delete<ApiResponse<LeaveGroupSessionResponse>>(
      `/api/sessions/${sessionId}/members/me`,
    )
    .then(unwrapResponse);
}

export function removeGroupMember(sessionId: number, userId: number) {
  return http
    .delete<ApiResponse<RemoveGroupMemberResponse>>(
      `/api/sessions/${sessionId}/members/${userId}`,
    )
    .then(unwrapResponse);
}

export function markSessionRead(sessionId: number) {
  return http
    .post<ApiResponse<MarkSessionReadResponse>>(
      `/api/sessions/${sessionId}/read`,
    )
    .then(unwrapResponse);
}
