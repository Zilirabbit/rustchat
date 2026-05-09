import { defineStore } from "pinia";

export const useAuthStore = defineStore("auth", {
  state: () => ({
    token: "",
    username: "",
  }),
  actions: {
    setSession(token: string, username: string) {
      this.token = token;
      this.username = username;
    },
    clearSession() {
      this.token = "";
      this.username = "";
    },
  },
});
