import { defineStore } from "pinia";
import { createChatSocket } from "../ws/client";
import type { ServerEvent } from "../types/chat";
import { useChatStore } from "./chat";

export const useConnectionStore = defineStore("connection", {
  state: () => ({
    socket: null as WebSocket | null,
    connected: false,
    connecting: false,
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

      this.disconnect();
      this.connecting = true;
      this.lastError = "";

      const socket = createChatSocket(token);
      const chatStore = useChatStore();

      socket.onopen = () => {
        this.connected = true;
        this.connecting = false;
      };

      socket.onmessage = (event) => {
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
        this.lastError = "WebSocket 连接异常";
      };

      socket.onclose = () => {
        this.connected = false;
        this.connecting = false;
      };

      this.socket = socket;
    },
    disconnect() {
      if (this.socket) {
        this.socket.onopen = null;
        this.socket.onmessage = null;
        this.socket.onerror = null;
        this.socket.onclose = null;

        if (
          this.socket.readyState === WebSocket.OPEN ||
          this.socket.readyState === WebSocket.CONNECTING
        ) {
          this.socket.close();
        }
      }

      this.socket = null;
      this.connected = false;
      this.connecting = false;
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
