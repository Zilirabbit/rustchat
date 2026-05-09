export interface ApiResponse<T> {
  code: number | string;
  message: string;
  data: T;
}

export interface ApiErrorResponse {
  code?: number | string;
  message?: string;
}
