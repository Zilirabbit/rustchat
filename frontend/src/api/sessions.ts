import { http } from "./http";

export function listSessions() {
  return http.get("/sessions");
}
