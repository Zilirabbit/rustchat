import { defineStore } from "pinia";
import { createChatSocket } from "../ws/client";
import type { ServerEvent } from "../types/chat";
import { useChatStore } from "./chat";

const MAX_RECONNECT_ATTEMPTS = 5;
const BASE_RECONNECT_DELAY_MS = 1000;
const MAX_RECONNECT_DELAY_MS = 8000;

export const useConnectionStore = defineStore("connection", {
  state: () => ({
    socket: null as WebSocket | null,
    connected: false,
    connecting: false,
    reconnecting: false,
    reconnectAttempts: 0,
    maxReconnectAttempts: MAX_RECONNECT_ATTEMPTS,
    reconnectTimerId: null as number | null,
    reconnectToken: "",
    shouldReconnect: false,
    lastError: "",
  }),
  actions: {
    connect(token: string) {
      if (!token) {
        return;
      }

      if (
        this.socket &&
        (this.socket.readyState === WebSocket.OPEN ||
          this.socket.readyState === WebSocket.CONNECTING)
      ) {
        return;
      }

      this.shouldReconnect = true;
      this.reconnectToken = token;
      this.openSocket(token);
    },
    openSocket(token: string) {
      this.closeSocket();
      this.clearReconnectTimer();
      this.connecting = true;
      this.reconnecting = this.reconnectAttempts > 0;
      this.lastError = this.reconnecting
        ? `实时连接中断，正在重连 (${this.reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`
        : "";

      const socket = createChatSocket(token);
      const chatStore = useChatStore();

      socket.onopen = () => {
        if (this.socket !== socket) {
          return;
        }

        this.connected = true;
        this.connecting = false;
        this.reconnecting = false;
        this.reconnectAttempts = 0;
        this.lastError = "";
      };

      socket.onmessage = (event) => {
        if (this.socket !== socket) {
          return;
        }

        try {
          const payload = JSON.parse(event.data) as ServerEvent;

          if (
            payload.type === "message_sent" ||
            payload.type === "receive_message"
          ) {
            chatStore.appendRealtimeMessage(payload.message);
            return;
          }

          if (payload.type === "error") {
            this.lastError = payload.message;
          }
        } catch {
          this.lastError = "收到无法解析的实时消息";
        }
      };

      socket.onerror = () => {
        if (this.socket !== socket) {
          return;
        }

        this.lastError = this.reconnecting
          ? "实时连接重连失败，准备再次尝试"
          : "WebSocket 连接异常";
      };

      socket.onclose = () => {
        if (this.socket !== socket) {
          return;
        }

        this.connected = false;
        this.connecting = false;
        this.socket = null;

        if (this.shouldReconnect && this.reconnectToken) {
          this.scheduleReconnect();
          return;
        }

        this.reconnecting = false;
      };

      this.socket = socket;
    },
    scheduleReconnect() {
      if (this.reconnectTimerId !== null) {
        return;
      }

      if (this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
        this.shouldReconnect = false;
        this.reconnecting = false;
        this.lastError = "实时连接重连失败，请刷新页面或重新登录";
        return;
      }

      this.reconnectAttempts += 1;
      this.reconnecting = true;
      this.lastError = `实时连接中断，正在重连 (${this.reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`;

      const delay = Math.min(
        BASE_RECONNECT_DELAY_MS * 2 ** (this.reconnectAttempts - 1),
        MAX_RECONNECT_DELAY_MS,
      );

      this.reconnectTimerId = window.setTimeout(() => {
        this.reconnectTimerId = null;

        if (this.shouldReconnect && this.reconnectToken) {
          this.openSocket(this.reconnectToken);
        }
      }, delay);
    },
    clearReconnectTimer() {
      if (this.reconnectTimerId === null) {
        return;
      }

      window.clearTimeout(this.reconnectTimerId);
      this.reconnectTimerId = null;
    },
    closeSocket() {
      if (!this.socket) {
        return;
      }

      const socket = this.socket;
      socket.onopen = null;
      socket.onmessage = null;
      socket.onerror = null;
      socket.onclose = null;

      if (
        socket.readyState === WebSocket.OPEN ||
        socket.readyState === WebSocket.CONNECTING
      ) {
        socket.close();
      }

      this.socket = null;
    },
    disconnect() {
      this.shouldReconnect = false;
      this.reconnectToken = "";
      this.clearReconnectTimer();
      this.closeSocket();
      this.connected = false;
      this.connecting = false;
      this.reconnecting = false;
      this.reconnectAttempts = 0;
      this.lastError = "";
    },
    sendTextMessage(sessionId: number, content: string) {
      const trimmedContent = content.trim();

      if (!trimmedContent) {
        return false;
      }

      if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
        this.lastError = "实时连接未建立，暂时无法发送";
        return false;
      }

      this.socket.send(
        JSON.stringify({
          type: "send_message",
          session_id: sessionId,
          content: trimmedContent,
        }),
      );

      return true;
    },
    clearError() {
      this.lastError = "";
    },
  },
});
