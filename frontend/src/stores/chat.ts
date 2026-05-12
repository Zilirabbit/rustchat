import { defineStore } from "pinia";
import { listConversations } from "../api/conversations";
import { getApiErrorMessage } from "../api/http";
import { listMessages } from "../api/messages";
import {
  addGroupMember,
  createGroupSession,
  createPrivateSession,
  leaveGroupSession,
  listGroupMembers,
  markSessionRead,
} from "../api/sessions";
import type {
  ConversationItem,
  GroupMemberListItem,
  MessageListItem,
  MessageSendStatus,
  WsChatMessage,
} from "../types/chat";

const ACTIVE_SESSION_STORAGE_KEY = "rustchat.chat.activeSessionId";
const MESSAGE_SEND_TIMEOUT_MS = 15000;
let nextLocalMessageId = 0;

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
    send_status: "sent",
  };
}

function createClientMessageId() {
  nextLocalMessageId += 1;
  return `local-${Date.now()}-${nextLocalMessageId}`;
}

function isOutgoingMessage(message: MessageListItem) {
  return Boolean(message.client_message_id && message.send_status);
}

export const useChatStore = defineStore("chat", {
  state: () => ({
    conversations: [] as ConversationItem[],
    activeSessionId: null as number | null,
    messagesBySessionId: {} as Record<number, MessageListItem[]>,
    groupMembersBySessionId: {} as Record<number, GroupMemberListItem[]>,
    ignoredSessionIds: {} as Record<number, true>,
    loadingConversations: false,
    loadingMessages: false,
    loadingGroupMembers: false,
    creatingSession: false,
    groupActionInProgress: false,
    sendTimeoutIdsByClientMessageId: {} as Record<string, number>,
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
    activeGroupMembers: (state) =>
      state.activeSessionId
        ? state.groupMembersBySessionId[state.activeSessionId] || []
        : [],
  },
  actions: {
    async loadConversations() {
      this.loadingConversations = true;
      this.error = "";

      try {
        const conversations = await listConversations();
        this.conversations = conversations.filter(
          (conversation) => !this.ignoredSessionIds[conversation.session_id],
        );
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.loadingConversations = false;
      }
    },
    async selectConversation(sessionId: number) {
      this.activeSessionId = sessionId;
      localStorage.setItem(ACTIVE_SESSION_STORAGE_KEY, String(sessionId));

      const conversation = this.conversations.find(
        (item) => item.session_id === sessionId,
      );
      const tasks = [this.loadMessages(sessionId), this.markRead(sessionId)];

      if (conversation?.session_type === "group") {
        tasks.push(this.loadGroupMembers(sessionId));
      }

      await Promise.all(tasks);
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
        const localPendingMessages = (
          this.messagesBySessionId[sessionId] || []
        ).filter(
          (message) =>
            isOutgoingMessage(message) && message.send_status !== "sent",
        );
        const serverMessages = [...page.messages].reverse().map((message) => ({
          ...message,
          send_status: "sent" as MessageSendStatus,
        }));
        this.messagesBySessionId[sessionId] = [
          ...serverMessages,
          ...localPendingMessages,
        ];
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.loadingMessages = false;
      }
    },
    async loadGroupMembers(sessionId: number) {
      this.loadingGroupMembers = true;
      this.error = "";

      try {
        const response = await listGroupMembers(sessionId);
        this.groupMembersBySessionId[sessionId] = response.members;
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.loadingGroupMembers = false;
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
    async createGroup(name: string, memberUserIds: number[]) {
      this.groupActionInProgress = true;
      this.error = "";

      try {
        const session = await createGroupSession(name, memberUserIds);
        await this.loadConversations();

        if (
          !this.conversations.some(
            (conversation) => conversation.session_id === session.session_id,
          )
        ) {
          this.conversations.unshift({
            session_id: session.session_id,
            session_type: session.session_type,
            session_name: session.name,
            last_message: null,
            last_message_time: null,
            unread_count: 0,
          });
        }

        await this.selectConversation(session.session_id);
        await this.loadGroupMembers(session.session_id);
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.groupActionInProgress = false;
      }
    },
    async addMemberToGroup(sessionId: number, userId: number) {
      this.groupActionInProgress = true;
      this.error = "";

      try {
        await addGroupMember(sessionId, userId);
        await this.loadConversations();
        await this.loadGroupMembers(sessionId);
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.groupActionInProgress = false;
      }
    },
    async leaveGroup(sessionId: number) {
      this.groupActionInProgress = true;
      this.error = "";

      try {
        await leaveGroupSession(sessionId);
        this.ignoredSessionIds[sessionId] = true;

        if (this.activeSessionId === sessionId) {
          this.activeSessionId = null;
          localStorage.removeItem(ACTIVE_SESSION_STORAGE_KEY);
        }

        delete this.messagesBySessionId[sessionId];
        delete this.groupMembersBySessionId[sessionId];
        this.conversations = this.conversations.filter(
          (conversation) => conversation.session_id !== sessionId,
        );
        await this.loadConversations();
      } catch (error) {
        this.error = getApiErrorMessage(error);
      } finally {
        this.groupActionInProgress = false;
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
    async recoverAfterReconnect() {
      const sessionId = this.activeSessionId;

      await this.loadConversations();

      if (!sessionId || this.error) {
        return;
      }

      const sessionExists = this.conversations.some(
        (conversation) => conversation.session_id === sessionId,
      );

      if (!sessionExists) {
        this.activeSessionId = null;
        delete this.messagesBySessionId[sessionId];
        delete this.groupMembersBySessionId[sessionId];
        localStorage.removeItem(ACTIVE_SESSION_STORAGE_KEY);
        return;
      }

      this.activeSessionId = sessionId;
      localStorage.setItem(ACTIVE_SESSION_STORAGE_KEY, String(sessionId));
      await Promise.all([this.loadMessages(sessionId), this.markRead(sessionId)]);
    },
    createOutgoingMessage(
      sessionId: number,
      content: string,
      senderId: number,
      senderUsername: string,
      sendStatus: MessageSendStatus,
    ) {
      const clientMessageId = createClientMessageId();
      const message: MessageListItem = {
        message_id: -nextLocalMessageId,
        session_id: sessionId,
        sender_id: senderId,
        sender_username: senderUsername,
        message_type: "text",
        content,
        created_at: new Date().toISOString(),
        client_message_id: clientMessageId,
        send_status: sendStatus,
      };
      const messages = this.messagesBySessionId[sessionId] || [];

      this.messagesBySessionId[sessionId] = [...messages, message];
      this.updateOutgoingConversationPreview(message);

      if (sendStatus === "sending") {
        this.scheduleSendTimeout(clientMessageId);
      }

      return clientMessageId;
    },
    getQueuedOutgoingMessages() {
      return Object.values(this.messagesBySessionId)
        .flat()
        .filter(
          (message) =>
            message.client_message_id && message.send_status === "queued",
        );
    },
    markOutgoingMessageSending(clientMessageId: string) {
      const message = this.findOutgoingMessage(clientMessageId);

      if (!message) {
        return;
      }

      message.send_status = "sending";
      message.send_error = "";
      this.scheduleSendTimeout(clientMessageId);
    },
    markOutgoingMessageQueued(clientMessageId: string, reason = "") {
      const message = this.findOutgoingMessage(clientMessageId);

      if (!message) {
        return;
      }

      message.send_status = "queued";
      message.send_error = reason;
      this.clearSendTimeout(clientMessageId);
    },
    requeueSendingMessages(reason: string) {
      Object.values(this.messagesBySessionId)
        .flat()
        .filter((message) => message.send_status === "sending")
        .forEach((message) => {
          if (message.client_message_id) {
            this.markOutgoingMessageQueued(message.client_message_id, reason);
          }
        });
    },
    markOutgoingMessageFailed(clientMessageId: string, reason: string) {
      const message = this.findOutgoingMessage(clientMessageId);

      if (!message) {
        return;
      }

      message.send_status = "failed";
      message.send_error = reason;
      this.clearSendTimeout(clientMessageId);
    },
    confirmOutgoingMessage(clientMessageId: string, message: WsChatMessage) {
      this.clearSendTimeout(clientMessageId);

      const confirmedMessage = {
        ...toListItem(message),
        client_message_id: clientMessageId,
        send_status: "sent" as MessageSendStatus,
      };

      for (const [sessionId, messages] of Object.entries(this.messagesBySessionId)) {
        const existingIndex = messages.findIndex(
          (item) => item.client_message_id === clientMessageId,
        );

        if (existingIndex >= 0) {
          const withoutDuplicate = messages.filter(
            (item, index) =>
              index !== existingIndex && item.message_id !== message.message_id,
          );
          withoutDuplicate.splice(
            Math.min(existingIndex, withoutDuplicate.length),
            0,
            confirmedMessage,
          );
          this.messagesBySessionId[Number(sessionId)] = withoutDuplicate;
          this.updateConversationPreview(confirmedMessage);
          return;
        }
      }

      this.appendRealtimeMessage(message);
    },
    appendRealtimeMessage(message: WsChatMessage) {
      if (this.ignoredSessionIds[message.session_id]) {
        return;
      }

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
    updateOutgoingConversationPreview(message: MessageListItem) {
      const conversation = this.conversations.find(
        (item) => item.session_id === message.session_id,
      );

      if (!conversation) {
        return;
      }

      conversation.last_message = message.content;
      conversation.last_message_time = message.created_at;
      conversation.unread_count = 0;
      this.conversations = [
        conversation,
        ...this.conversations.filter(
          (item) => item.session_id !== message.session_id,
        ),
      ];
    },
    findOutgoingMessage(clientMessageId: string) {
      return Object.values(this.messagesBySessionId)
        .flat()
        .find((message) => message.client_message_id === clientMessageId);
    },
    scheduleSendTimeout(clientMessageId: string) {
      this.clearSendTimeout(clientMessageId);
      this.sendTimeoutIdsByClientMessageId[clientMessageId] = window.setTimeout(
        () => {
          this.markOutgoingMessageFailed(clientMessageId, "发送超时，请重试");
        },
        MESSAGE_SEND_TIMEOUT_MS,
      );
    },
    clearSendTimeout(clientMessageId: string) {
      const timeoutId = this.sendTimeoutIdsByClientMessageId[clientMessageId];

      if (!timeoutId) {
        return;
      }

      window.clearTimeout(timeoutId);
      delete this.sendTimeoutIdsByClientMessageId[clientMessageId];
    },
    clearSendTimeouts() {
      Object.values(this.sendTimeoutIdsByClientMessageId).forEach((timeoutId) => {
        window.clearTimeout(timeoutId);
      });
      this.sendTimeoutIdsByClientMessageId = {};
    },
    reset() {
      this.clearSendTimeouts();
      this.conversations = [];
      this.activeSessionId = null;
      this.messagesBySessionId = {};
      this.groupMembersBySessionId = {};
      this.ignoredSessionIds = {};
      this.loadingConversations = false;
      this.loadingMessages = false;
      this.loadingGroupMembers = false;
      this.creatingSession = false;
      this.groupActionInProgress = false;
      this.sendTimeoutIdsByClientMessageId = {};
      this.error = "";
      localStorage.removeItem(ACTIVE_SESSION_STORAGE_KEY);
    },
  },
});
