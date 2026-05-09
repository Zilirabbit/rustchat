import { defineStore } from "pinia";
import { getMe, login as loginApi } from "../api/auth";
import {
  AUTH_TOKEN_STORAGE_KEY,
  AUTH_USER_STORAGE_KEY,
  getApiErrorMessage,
} from "../api/http";
import type { UserProfile } from "../types/chat";

export const BYPASS_AUTH_TOKEN = "rustchat-bypass-auth-token";

function readStoredUser(): UserProfile | null {
  const rawUser = localStorage.getItem(AUTH_USER_STORAGE_KEY);

  if (!rawUser) {
    return null;
  }

  try {
    return JSON.parse(rawUser) as UserProfile;
  } catch {
    localStorage.removeItem(AUTH_USER_STORAGE_KEY);
    return null;
  }
}

export const useAuthStore = defineStore("auth", {
  state: () => ({
    token: localStorage.getItem(AUTH_TOKEN_STORAGE_KEY) || "",
    user: readStoredUser(),
    restoring: false,
    error: "",
  }),
  getters: {
    isAuthenticated: (state) => Boolean(state.token && state.user),
    isBypassAuthenticated: (state) => state.token === BYPASS_AUTH_TOKEN,
    username: (state) => state.user?.username || "",
  },
  actions: {
    setSession(token: string, user: UserProfile) {
      this.token = token;
      this.user = user;
      this.error = "";
      localStorage.setItem(AUTH_TOKEN_STORAGE_KEY, token);
      localStorage.setItem(AUTH_USER_STORAGE_KEY, JSON.stringify(user));
    },
    async login(username: string, password: string) {
      const session = await loginApi(username, password);
      this.setSession(session.token, session.user);
      return session;
    },
    enterBypassMode() {
      this.setSession(BYPASS_AUTH_TOKEN, {
        user_id: 0,
        username: "demo",
        avatar_url: null,
      });
    },
    async restoreSession() {
      if (!this.token) {
        return false;
      }

      if (this.isBypassAuthenticated) {
        if (!this.user) {
          this.enterBypassMode();
        }

        return true;
      }

      if (this.user) {
        return true;
      }

      this.restoring = true;
      this.error = "";

      try {
        const user = await getMe();
        this.user = user;
        localStorage.setItem(AUTH_USER_STORAGE_KEY, JSON.stringify(user));
        return true;
      } catch (error) {
        this.error = getApiErrorMessage(error);
        this.clearSession();
        return false;
      } finally {
        this.restoring = false;
      }
    },
    clearSession() {
      this.token = "";
      this.user = null;
      localStorage.removeItem(AUTH_TOKEN_STORAGE_KEY);
      localStorage.removeItem(AUTH_USER_STORAGE_KEY);
    },
  },
});
