import type { ApiResponse } from "../types/api";
import type { AuthPayload, UserProfile } from "../types/chat";
import { http, unwrapResponse } from "./http";

export function login(username: string, password: string) {
  return http
    .post<ApiResponse<AuthPayload>>("/api/login", { username, password })
    .then(unwrapResponse);
}

export function register(username: string, password: string) {
  return http
    .post<ApiResponse<UserProfile>>("/api/register", { username, password })
    .then(unwrapResponse);
}

export function getMe() {
  return http.get<ApiResponse<UserProfile>>("/api/me").then(unwrapResponse);
}
