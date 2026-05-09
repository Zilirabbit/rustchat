import { http } from "./http";

export function login(username: string, password: string) {
  return http.post("/login", { username, password });
}

export function register(username: string, password: string) {
  return http.post("/register", { username, password });
}
