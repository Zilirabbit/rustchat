import type { ApiResponse } from "../types/api";
import type { UserSearchItem } from "../types/chat";
import { http, unwrapResponse } from "./http";

export function searchUsers(keyword: string) {
  return http
    .get<ApiResponse<UserSearchItem[]>>("/api/users", {
      params: { keyword },
    })
    .then(unwrapResponse);
}
