import axios from "axios";
import type { AxiosError } from "axios";
import type { ApiErrorResponse, ApiResponse } from "../types/api";

export const AUTH_TOKEN_STORAGE_KEY = "rustchat.auth.token";
export const AUTH_USER_STORAGE_KEY = "rustchat.auth.user";

export const http = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:3000",
});

http.interceptors.request.use((config) => {
  const token = localStorage.getItem(AUTH_TOKEN_STORAGE_KEY);

  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }

  return config;
});

export function unwrapResponse<T>(response: { data: ApiResponse<T> }): T {
  return response.data.data;
}

export function getApiErrorMessage(error: unknown): string {
  const axiosError = error as AxiosError<ApiErrorResponse>;
  const message = axiosError.response?.data?.message || axiosError.message;
  return message || "请求失败，请稍后再试";
}
