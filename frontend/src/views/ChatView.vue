<template>
  <main class="chat-shell mongo-chat-shell">
    <div v-if="visibleError" class="toast">
      <span>{{ visibleError }}</span>
      <button type="button" @click="clearErrors">Dismiss</button>
    </div>

    <section class="chat-layout">
      <aside class="workspace-sidebar" aria-label="Rust Chat conversations">
        <div class="workspace-brand-row">
          <div class="brand-lockup">
            <span class="brand-leaf" aria-hidden="true"></span>
            <strong>Rust Chat</strong>
          </div>
          <span class="connection-pill" :class="connectionStatusClass">
            {{ connectionLabel }}
          </span>
        </div>

        <section class="room-section session-actions">
          <div class="room-section-title">
            <span>New Private Room</span>
          </div>
          <form class="private-search" @submit.prevent="searchPrivateUsers">
            <input
              v-model="privateKeyword"
              type="search"
              placeholder="Search username"
              :disabled="actionsDisabled"
            />
            <button type="submit" :disabled="actionsDisabled">Go</button>
          </form>
          <p v-if="privateSearchError" class="inline-error">{{ privateSearchError }}</p>
          <p v-else-if="privateSearchLoading" class="muted-line">Searching...</p>
          <div v-if="privateCandidates.length" class="private-results">
            <button
              v-for="user in privateCandidates"
              :key="user.user_id"
              type="button"
              @click="openPrivateSession(user.user_id)"
            >
              <span class="mini-avatar">{{ avatarLabel(user.username) }}</span>
              <span>{{ user.username }}</span>
            </button>
          </div>
        </section>

        <section class="room-section session-actions">
          <div class="room-section-title">
            <span>New Group Room</span>
          </div>
          <input
            v-model="groupName"
            class="session-input"
            type="text"
            maxlength="100"
            placeholder="Group name"
            :disabled="actionsDisabled"
          />
          <form class="private-search" @submit.prevent="searchGroupUsers">
            <input
              v-model="groupKeyword"
              type="search"
              placeholder="Search members"
              :disabled="actionsDisabled"
            />
            <button type="submit" :disabled="actionsDisabled">Go</button>
          </form>
          <p v-if="groupSearchError" class="inline-error">{{ groupSearchError }}</p>
          <p v-else-if="groupSearchLoading" class="muted-line">Searching...</p>
          <div v-if="selectedGroupMembers.length" class="selected-users compact-selected">
            <button
              v-for="user in selectedGroupMembers"
              :key="user.user_id"
              class="user-chip"
              type="button"
              :disabled="actionsDisabled"
              @click="removeGroupMember(user.user_id)"
            >
              {{ user.username }}
            </button>
          </div>
          <div v-if="groupCandidates.length" class="private-results">
            <button
              v-for="user in groupCandidates"
              :key="user.user_id"
              type="button"
              :disabled="isGroupMemberSelected(user.user_id)"
              @click="selectGroupMember(user)"
            >
              <span class="mini-avatar">{{ avatarLabel(user.username) }}</span>
              <span>{{ user.username }}</span>
            </button>
          </div>
          <button
            class="primary-button create-group-button"
            type="button"
            :disabled="actionsDisabled || !groupName.trim()"
            @click="createGroup"
          >
            Create group
          </button>
        </section>

        <section class="room-section conversation-section">
          <div class="room-section-title">
            <span>Group Rooms</span>
          </div>
          <p v-if="chatStore.loadingConversations" class="muted-line">Loading rooms...</p>
          <p v-else-if="groupConversations.length === 0" class="empty-list">
            No group rooms yet.
          </p>
          <button
            v-for="conversation in groupConversations"
            :key="conversation.session_id"
            class="room-item"
            :class="{ active: conversation.session_id === chatStore.activeSessionId }"
            type="button"
            @click="chatStore.selectConversation(conversation.session_id)"
          >
            <span>#</span>
            <strong>{{ conversation.session_name }}</strong>
            <em v-if="conversation.unread_count">{{ conversation.unread_count }}</em>
          </button>
        </section>

        <section class="room-section conversation-section">
          <div class="room-section-title">
            <span>Private Room</span>
          </div>
          <p v-if="!chatStore.loadingConversations && privateConversations.length === 0" class="empty-list">
            No private rooms yet.
          </p>
          <button
            v-for="conversation in privateConversations"
            :key="conversation.session_id"
            class="private-room"
            :class="{ active: conversation.session_id === chatStore.activeSessionId }"
            type="button"
            @click="chatStore.selectConversation(conversation.session_id)"
          >
            <span class="mini-avatar">{{ avatarLabel(conversation.session_name) }}</span>
            <span>
              <strong>{{ conversation.session_name }}</strong>
              <small>{{ conversation.last_message || "No messages yet" }}</small>
            </span>
            <em v-if="conversation.unread_count">{{ conversation.unread_count }}</em>
          </button>
        </section>

        <div class="sidebar-profile">
          <span class="profile-avatar">{{ avatarLabel(authStore.username || "demo") }}</span>
          <span>
            <strong>{{ authStore.username || "demo" }}</strong>
            <small>@{{ authStore.username || "demo" }}</small>
          </span>
          <button type="button" aria-label="Log out" @click="logout">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
              <path d="m16 17 5-5-5-5M21 12H9" />
            </svg>
          </button>
        </div>
      </aside>

      <section class="chat-main">
        <header class="chat-header channel-header">
          <div class="channel-title">
            <span>{{ activeConversation?.session_type === "private" ? "@" : "#" }}</span>
            <div>
              <h1>{{ activeConversation?.session_name || "Select a room" }}</h1>
              <p>{{ activeConversation ? activeConversation.session_type : "Choose a real conversation from the sidebar." }}</p>
            </div>
          </div>
        </header>

        <MessageList
          v-if="activeConversation"
          :messages="activeMessages"
          :current-user-id="authStore.user?.user_id || 0"
          :loading="chatStore.loadingMessages"
        />
        <div v-else class="empty-chat">
          <h1>No room selected</h1>
          <p>Create or select a private room or group room to start chatting.</p>
        </div>

        <MessageInput
          :disabled="!connectionStore.connected || !chatStore.activeSessionId"
          @send="sendMessage"
        />
      </section>

      <aside class="info-sidebar" aria-label="Room information">
        <section v-if="activeConversation?.session_type === 'group'" class="room-members-panel">
          <h2>{{ activeConversation.session_name }}</h2>
          <p class="muted-line">Group members from the existing members API.</p>

          <form class="private-search add-member-search" @submit.prevent="searchAddUsers">
            <input
              v-model="addKeyword"
              type="search"
              placeholder="Search user to add"
              :disabled="actionsDisabled"
            />
            <button type="submit" :disabled="actionsDisabled">Go</button>
          </form>
          <p v-if="addSearchError" class="inline-error">{{ addSearchError }}</p>
          <p v-else-if="addSearchLoading" class="muted-line">Searching...</p>
          <div v-if="addCandidates.length" class="private-results member-add-results">
            <button
              v-for="user in addCandidates"
              :key="user.user_id"
              type="button"
              :disabled="isActiveMember(user.user_id)"
              @click="addGroupMember(user.user_id)"
            >
              <span class="mini-avatar">{{ avatarLabel(user.username) }}</span>
              <span>{{ user.username }}</span>
            </button>
          </div>

          <div class="member-panel-header">
            <h3>Members</h3>
            <span>{{ chatStore.activeGroupMembers.length }}</span>
          </div>
          <p v-if="chatStore.loadingGroupMembers" class="muted-line">
            Loading members...
          </p>
          <p v-else-if="chatStore.activeGroupMembers.length === 0" class="empty-list">
            No member data yet.
          </p>
          <div v-else class="real-member-list">
            <div
              v-for="member in chatStore.activeGroupMembers"
              :key="member.user_id"
              class="real-member-row"
            >
              <span class="member-photo avatar-neutral">
                {{ avatarLabel(member.username) }}
              </span>
              <span>
                <strong>{{ member.username }}</strong>
                <small>{{ roleLabel(member.role) }} · Joined {{ formatDate(member.joined_at) }}</small>
              </span>
            </div>
          </div>

          <button
            class="secondary-button danger-button leave-room-button"
            type="button"
            :disabled="actionsDisabled"
            @click="leaveGroup"
          >
            Leave group
          </button>
        </section>

        <section v-else class="room-members-panel">
          <h2>Room Info</h2>
          <p class="muted-line">
            Group member details appear here when a real group room is selected.
          </p>
        </section>
      </aside>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import MessageInput from "../components/MessageInput.vue";
import MessageList from "../components/MessageList.vue";
import { getApiErrorMessage } from "../api/http";
import { searchUsers } from "../api/users";
import { useAuthStore } from "../stores/auth";
import { useChatStore } from "../stores/chat";
import { useConnectionStore } from "../stores/connection";
import type { UserSearchItem } from "../types/chat";

const router = useRouter();
const authStore = useAuthStore();
const chatStore = useChatStore();
const connectionStore = useConnectionStore();
const privateKeyword = ref("");
const privateSearchLoading = ref(false);
const privateSearchError = ref("");
const privateCandidates = ref<UserSearchItem[]>([]);
const groupName = ref("");
const groupKeyword = ref("");
const groupSearchLoading = ref(false);
const groupSearchError = ref("");
const groupCandidates = ref<UserSearchItem[]>([]);
const selectedGroupMembers = ref<UserSearchItem[]>([]);
const addKeyword = ref("");
const addSearchLoading = ref(false);
const addSearchError = ref("");
const addCandidates = ref<UserSearchItem[]>([]);

const activeConversation = computed(() => chatStore.activeConversation);
const activeMessages = computed(() => chatStore.activeMessages);
const visibleError = computed(
  () => chatStore.error || connectionStore.lastError || authStore.error,
);
const connectionLabel = computed(() => {
  if (connectionStore.connected) {
    return "Realtime online";
  }

  if (connectionStore.reconnecting) {
    return `Reconnecting ${connectionStore.reconnectAttempts}/${connectionStore.maxReconnectAttempts}`;
  }

  if (connectionStore.connecting) {
    return "Connecting";
  }

  return "Offline";
});
const connectionStatusClass = computed(() => ({
  online: connectionStore.connected,
  reconnecting: connectionStore.reconnecting,
  offline:
    !connectionStore.connected &&
    !connectionStore.connecting &&
    !connectionStore.reconnecting,
}));
const groupConversations = computed(() =>
  chatStore.conversations.filter(
    (conversation) => conversation.session_type === "group",
  ),
);
const privateConversations = computed(() =>
  chatStore.conversations.filter(
    (conversation) => conversation.session_type === "private",
  ),
);
const actionsDisabled = computed(
  () => chatStore.creatingSession || chatStore.groupActionInProgress,
);
const activeMemberIds = computed(
  () => new Set(chatStore.activeGroupMembers.map((member) => member.user_id)),
);

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
  privateCandidates.value = [];
  privateKeyword.value = "";
}

async function searchPrivateUsers() {
  const keyword = privateKeyword.value.trim();

  if (!keyword) {
    privateCandidates.value = [];
    privateSearchError.value = "";
    return;
  }

  privateSearchLoading.value = true;
  privateSearchError.value = "";

  try {
    privateCandidates.value = await searchUsers(keyword);
  } catch (error) {
    privateSearchError.value = getApiErrorMessage(error);
  } finally {
    privateSearchLoading.value = false;
  }
}

async function searchGroupUsers() {
  const keyword = groupKeyword.value.trim();

  if (!keyword) {
    groupCandidates.value = [];
    groupSearchError.value = "";
    return;
  }

  groupSearchLoading.value = true;
  groupSearchError.value = "";

  try {
    groupCandidates.value = await searchUsers(keyword);
  } catch (error) {
    groupSearchError.value = getApiErrorMessage(error);
  } finally {
    groupSearchLoading.value = false;
  }
}

function selectGroupMember(user: UserSearchItem) {
  if (isGroupMemberSelected(user.user_id)) {
    return;
  }

  selectedGroupMembers.value = [...selectedGroupMembers.value, user];
}

function removeGroupMember(userId: number) {
  selectedGroupMembers.value = selectedGroupMembers.value.filter(
    (user) => user.user_id !== userId,
  );
}

function isGroupMemberSelected(userId: number) {
  return selectedGroupMembers.value.some((user) => user.user_id === userId);
}

function createGroup() {
  const name = groupName.value.trim();

  if (!name) {
    return;
  }

  void chatStore.createGroup(
    name,
    selectedGroupMembers.value.map((user) => user.user_id),
  );
  groupName.value = "";
  groupKeyword.value = "";
  groupCandidates.value = [];
  selectedGroupMembers.value = [];
}

async function searchAddUsers() {
  const keyword = addKeyword.value.trim();

  if (!keyword) {
    addCandidates.value = [];
    addSearchError.value = "";
    return;
  }

  addSearchLoading.value = true;
  addSearchError.value = "";

  try {
    addCandidates.value = await searchUsers(keyword);
  } catch (error) {
    addSearchError.value = getApiErrorMessage(error);
  } finally {
    addSearchLoading.value = false;
  }
}

function addGroupMember(userId: number) {
  if (!chatStore.activeSessionId || isActiveMember(userId)) {
    return;
  }

  void chatStore.addMemberToGroup(chatStore.activeSessionId, userId);
  addKeyword.value = "";
  addCandidates.value = [];
}

function isActiveMember(userId: number) {
  return activeMemberIds.value.has(userId);
}

function leaveGroup() {
  if (!chatStore.activeSessionId) {
    return;
  }

  void chatStore.leaveGroup(chatStore.activeSessionId);
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

function avatarLabel(value: string) {
  return value.slice(0, 1).toUpperCase() || "?";
}

function roleLabel(role: string) {
  return role === "owner" ? "Owner" : "Member";
}

function formatDate(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return "";
  }

  return date.toLocaleDateString();
}
</script>
