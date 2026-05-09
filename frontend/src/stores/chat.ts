import { defineStore } from "pinia";

export const useChatStore = defineStore("chat", {
  state: () => ({
    activeConversationId: null as string | null,
  }),
  actions: {
    selectConversation(conversationId: string) {
      this.activeConversationId = conversationId;
    },
  },
});
