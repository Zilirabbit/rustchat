import { defineStore } from "pinia";
import { createChatSocket } from "../ws/client";
import type { ServerEvent } from "../types/chat";
import { useChatStore } from "./chat";

const MAX_RECONNECT_ATTEMPTS = 5;
const BASE_RECONNECT_DELAY_MS = 1000;
const MAX_RECONNECT_DELAY_MS = 8000;

interface OutgoingMessagePayload {
  client_message_id: string;
  session_id: number;
  content: string;
}

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

        const recoveredFromReconnect =
          this.reconnecting || this.reconnectAttempts > 0;

        this.connected = true;
        this.connecting = false;
        this.reconnecting = false;
        this.reconnectAttempts = 0;
        this.lastError = "";

        void (async () => {
          if (recoveredFromReconnect) {
            await chatStore.recoverAfterReconnect();
          }

          this.flushQueuedMessages();
        })();
      };

      socket.onmessage = (event) => {
        if (this.socket !== socket) {
          return;
        }

        try {
          const payload = JSON.parse(event.data) as ServerEvent;

          if (payload.type === "message_sent") {
            if (payload.client_message_id) {
              chatStore.confirmOutgoingMessage(
                payload.client_message_id,
                payload.message,
              );
            } else {
              chatStore.appendRealtimeMessage(payload.message);
            }
            return;
          }

          if (payload.type === "receive_message") {
            chatStore.appendRealtimeMessage(payload.message);
            return;
          }

          if (payload.type === "conversation_updated") {
            void chatStore.handleConversationUpdated(payload.session_id);
            return;
          }

          if (payload.type === "error") {
            if (payload.client_message_id) {
              chatStore.markOutgoingMessageFailed(
                payload.client_message_id,
                payload.message,
              );
              return;
            }

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
        chatStore.requeueSendingMessages("连接中断，等待重连后自动补发");

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
    sendTextMessage(
      sessionId: number,
      content: string,
      senderId: number,
      senderUsername: string,
    ) {
      const trimmedContent = content.trim();

      if (!trimmedContent || !senderId || !senderUsername) {
        return false;
      }

      const isReady = this.socket?.readyState === WebSocket.OPEN;
      const chatStore = useChatStore();
      const clientMessageId = chatStore.createOutgoingMessage(
        sessionId,
        trimmedContent,
        senderId,
        senderUsername,
        isReady ? "sending" : "queued",
      );

      if (!isReady) {
        this.lastError = "实时连接未建立，消息已进入待发送队列";
        return true;
      }

      this.sendOutgoingMessage({
        client_message_id: clientMessageId,
        session_id: sessionId,
        content: trimmedContent,
      });

      return true;
    },
    retryMessage(clientMessageId: string) {
      const chatStore = useChatStore();
      const message = chatStore.findOutgoingMessage(clientMessageId);

      if (!message || !message.client_message_id) {
        return false;
      }

      if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
        chatStore.markOutgoingMessageQueued(
          clientMessageId,
          "实时连接未建立，等待重连后自动补发",
        );
        this.lastError = "实时连接未建立，消息已进入待发送队列";
        return true;
      }

      return this.sendOutgoingMessage({
        client_message_id: message.client_message_id,
        session_id: message.session_id,
        content: message.content,
      });
    },
    flushQueuedMessages() {
      if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
        return;
      }

      const chatStore = useChatStore();

      chatStore.getQueuedOutgoingMessages().forEach((message) => {
        if (!message.client_message_id) {
          return;
        }

        this.sendOutgoingMessage({
          client_message_id: message.client_message_id,
          session_id: message.session_id,
          content: message.content,
        });
      });
    },
    sendOutgoingMessage(payload: OutgoingMessagePayload) {
      const chatStore = useChatStore();

      if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
        chatStore.markOutgoingMessageQueued(
          payload.client_message_id,
          "实时连接未建立，等待重连后自动补发",
        );
        return false;
      }

      try {
        chatStore.markOutgoingMessageSending(payload.client_message_id);
        this.socket.send(
          JSON.stringify({
            type: "send_message",
            session_id: payload.session_id,
            content: payload.content,
            client_message_id: payload.client_message_id,
          }),
        );
        return true;
      } catch {
        chatStore.markOutgoingMessageFailed(
          payload.client_message_id,
          "消息发送失败，请重试",
        );
        return false;
      }
    },
    clearError() {
      this.lastError = "";
    },
  },
});
