import { defineStore } from "pinia";
import { listConversations } from "../api/conversations";
import { getApiErrorMessage } from "../api/http";
import { listMessages } from "../api/messages";
import { createPrivateSession, markSessionRead } from "../api/sessions";
import type {
  ConversationItem,
  MessageListItem,
  WsChatMessage,
} from "../types/chat";

const ACTIVE_SESSION_STORAGE_KEY = "rustchat.chat.activeSessionId";

function readStoredActiveSessionId() {
  const rawSessionId = localStorage.getItem(ACTIVE_SESSION_STORAGE_KEY);

  if (!rawSessionId) {
    return null;
  }

  const sessionId = Number(rawSessionId);

  if (!Number.isSafeInteger(sessionId) || sessionId <= 0) {
    localStorage.removeItem(ACTIVE_SESSION_STORAGE_KEY);
    return null;
  }

  return sessionId;
}

function toListItem(message: WsChatMessage): MessageListItem {
  return {
    message_id: message.message_id,
    session_id: message.session_id,
    sender_id: message.sender_id,
    sender_username: message.sender_username,
    message_type: "text",
    content: message.content,
    created_at: message.created_at,
  };
}

export const useChatStore = defineStore("chat", {
  state: () => ({
    conversations: [] as ConversationItem[],
    activeSessionId: null as number | null,
    messagesBySessionId: {} as Record<number, MessageListItem[]>,
    loadingConversations: false,
    loadingMessages: false,
    creatingSession: false,
    error: "",
  }),
  getters: {
    activeConversation: (state) =>
      state.conversations.find(
        (conversation) => conversation.session_id === state.activeSessionId,
      ) || null,
    activeMessages: (state) =>
      state.activeSessionId
        ? state.messagesBySessionId[state.activeSessionId] || []
        : [],
  },
  actions: {
    async loadConversations() {
      this.loadingConversations = true;
      this.error = "";

      try {
        this.conversations = await listConversations();
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.loadingConversations = false;
      }
    },
    async selectConversation(sessionId: number) {
      this.activeSessionId = sessionId;
      localStorage.setItem(ACTIVE_SESSION_STORAGE_KEY, String(sessionId));

      await Promise.all([this.loadMessages(sessionId), this.markRead(sessionId)]);
    },
    async restoreActiveConversation() {
      const sessionId = readStoredActiveSessionId();

      if (!sessionId) {
        return false;
      }

      const sessionExists = this.conversations.some(
        (conversation) => conversation.session_id === sessionId,
      );

      if (!sessionExists) {
        localStorage.removeItem(ACTIVE_SESSION_STORAGE_KEY);
        this.activeSessionId = null;
        return false;
      }

      await this.selectConversation(sessionId);
      return true;
    },
    async loadMessages(sessionId: number) {
      this.loadingMessages = true;
      this.error = "";

      try {
        const page = await listMessages(sessionId);
        this.messagesBySessionId[sessionId] = [...page.messages].reverse();
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.loadingMessages = false;
      }
    },
    async createOrOpenPrivateSession(targetUserId: number) {
      this.creatingSession = true;
      this.error = "";

      try {
        const session = await createPrivateSession(targetUserId);
        await this.loadConversations();

        if (
          !this.conversations.some(
            (conversation) => conversation.session_id === session.session_id,
          )
        ) {
          this.conversations.unshift({
            session_id: session.session_id,
            session_type: session.session_type,
            session_name: `用户 ${session.peer_user_id}`,
            last_message: null,
            last_message_time: null,
            unread_count: 0,
          });
        }

        await this.selectConversation(session.session_id);
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.creatingSession = false;
      }
    },
    async markRead(sessionId: number) {
      try {
        await markSessionRead(sessionId);
        const conversation = this.conversations.find(
          (item) => item.session_id === sessionId,
        );

        if (conversation) {
          conversation.unread_count = 0;
        }
      } catch (error) {
        this.error = getApiErrorMessage(error);
      }
    },
    appendRealtimeMessage(message: WsChatMessage) {
      const item = toListItem(message);
      const messages = this.messagesBySessionId[item.session_id] || [];

      if (
        !messages.some(
          (existingMessage) => existingMessage.message_id === item.message_id,
        )
      ) {
        this.messagesBySessionId[item.session_id] = [...messages, item];
      }

      this.updateConversationPreview(item);
    },
    updateConversationPreview(message: MessageListItem) {
      const conversation = this.conversations.find(
        (item) => item.session_id === message.session_id,
      );

      if (!conversation) {
        void this.loadConversations();
        return;
      }

      conversation.last_message = message.content;
      conversation.last_message_time = message.created_at;

      if (this.activeSessionId === message.session_id) {
        conversation.unread_count = 0;
        void this.markRead(message.session_id);
      } else {
        conversation.unread_count += 1;
      }

      this.conversations = [
        conversation,
        ...this.conversations.filter(
          (item) => item.session_id !== message.session_id,
        ),
      ];
    },
    reset() {
      this.conversations = [];
      this.activeSessionId = null;
      this.messagesBySessionId = {};
      this.loadingConversations = false;
      this.loadingMessages = false;
      this.creatingSession = false;
      this.error = "";
      localStorage.removeItem(ACTIVE_SESSION_STORAGE_KEY);
    },
  },
});
