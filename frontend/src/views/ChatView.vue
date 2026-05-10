<template>
  <main class="chat-shell">
    <header class="topbar">
      <div class="topbar-brand">
        <strong>RustChat</strong>
        <span>{{ authStore.username }}</span>
      </div>

      <div class="topbar-actions">
        <span class="connection-pill" :class="{ online: connectionStore.connected }">
          {{ connectionLabel }}
        </span>
        <button class="secondary-button compact" type="button" @click="logout">
          退出
        </button>
      </div>
    </header>

    <div v-if="visibleError" class="toast">
      <span>{{ visibleError }}</span>
      <button type="button" @click="clearErrors">关闭</button>
    </div>

    <section class="chat-layout">
      <aside class="sidebar">
        <UserSearch
          :disabled="chatStore.creatingSession"
          @select-user="openPrivateSession"
        />
        <ConversationList
          :conversations="chatStore.conversations"
          :active-session-id="chatStore.activeSessionId"
          :loading="chatStore.loadingConversations"
          @select="chatStore.selectConversation"
        />
      </aside>

      <section class="chat-main">
        <template v-if="activeConversation">
          <header class="chat-header">
            <div>
              <h1>{{ activeConversation.session_name }}</h1>
              <p>{{ activeConversation.session_type }}</p>
            </div>
          </header>

          <MessageList
            :messages="activeMessages"
            :current-user-id="authStore.user?.user_id || 0"
            :loading="chatStore.loadingMessages"
          />

          <MessageInput
            :disabled="!connectionStore.connected"
            @send="sendMessage"
          />
        </template>

        <div v-else class="empty-chat">
          <h1>选择一个会话</h1>
          <p>从左侧会话列表进入聊天，或搜索用户创建新的私聊。</p>
        </div>
      </section>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import { useRouter } from "vue-router";
import ConversationList from "../components/ConversationList.vue";
import MessageInput from "../components/MessageInput.vue";
import MessageList from "../components/MessageList.vue";
import UserSearch from "../components/UserSearch.vue";
import { useAuthStore } from "../stores/auth";
import { useChatStore } from "../stores/chat";
import { useConnectionStore } from "../stores/connection";

const router = useRouter();
const authStore = useAuthStore();
const chatStore = useChatStore();
const connectionStore = useConnectionStore();

const activeConversation = computed(() => chatStore.activeConversation);
const activeMessages = computed(() => chatStore.activeMessages);
const visibleError = computed(
  () => chatStore.error || connectionStore.lastError || authStore.error,
);
const connectionLabel = computed(() => {
  if (connectionStore.connected) {
    return "实时已连接";
  }

  if (connectionStore.connecting) {
    return "连接中";
  }

  return "实时未连接";
});

onMounted(async () => {
  await authStore.restoreSession();

  if (authStore.isBypassAuthenticated) {
    return;
  }

  await chatStore.loadConversations();
  await chatStore.restoreActiveConversation();

  if (authStore.token) {
    connectionStore.connect(authStore.token);
  }
});

onBeforeUnmount(() => {
  connectionStore.disconnect();
});

function openPrivateSession(userId: number) {
  void chatStore.createOrOpenPrivateSession(userId);
}

function sendMessage(content: string) {
  if (!chatStore.activeSessionId) {
    return;
  }

  connectionStore.sendTextMessage(chatStore.activeSessionId, content);
}

function clearErrors() {
  chatStore.error = "";
  authStore.error = "";
  connectionStore.clearError();
}

async function logout() {
  connectionStore.disconnect();
  chatStore.reset();
  authStore.clearSession();
  await router.push("/login");
}
</script>
