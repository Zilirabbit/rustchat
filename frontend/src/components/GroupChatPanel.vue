<template>
  <section class="group-panel">
    <div class="section-header">
      <h2>群聊</h2>
      <span v-if="disabled" class="muted-line">处理中...</span>
    </div>

    <div class="group-block">
      <label class="field-label" for="group-name">创建群聊</label>
      <input
        id="group-name"
        v-model="groupName"
        class="text-input"
        type="text"
        maxlength="100"
        placeholder="群聊名称"
        :disabled="disabled"
      />

      <div class="search-row">
        <input
          v-model="createKeyword"
          class="text-input"
          type="search"
          placeholder="搜索初始成员"
          autocomplete="off"
          :disabled="disabled"
          @keyup.enter="searchCreateCandidates"
        />
        <button
          class="icon-button"
          type="button"
          :disabled="disabled"
          @click="searchCreateCandidates"
        >
          搜索
        </button>
      </div>

      <p v-if="createError" class="inline-error">{{ createError }}</p>
      <p v-else-if="createLoading" class="muted-line">搜索中...</p>

      <div v-if="selectedCreateMembers.length" class="selected-users">
        <button
          v-for="user in selectedCreateMembers"
          :key="user.user_id"
          class="user-chip"
          type="button"
          :disabled="disabled"
          @click="removeCreateMember(user.user_id)"
        >
          {{ user.username }}
        </button>
      </div>

      <div v-if="createCandidates.length" class="search-results compact-results">
        <button
          v-for="user in createCandidates"
          :key="user.user_id"
          class="search-result"
          type="button"
          :disabled="disabled || isCreateMemberSelected(user.user_id)"
          @click="selectCreateMember(user)"
        >
          <span class="avatar">{{ avatarLabel(user.username) }}</span>
          <span>{{ user.username }}</span>
        </button>
      </div>

      <button
        class="primary-button"
        type="button"
        :disabled="disabled || !groupName.trim()"
        @click="submitCreateGroup"
      >
        创建群聊
      </button>
    </div>

    <div v-if="isGroupActive" class="group-block">
      <div class="group-title-row">
        <span class="field-label">当前群聊</span>
        <strong>{{ activeGroupName }}</strong>
      </div>

      <div class="member-list" aria-live="polite">
        <p v-if="loadingMembers" class="muted-line">成员加载中...</p>
        <template v-else-if="members.length">
          <div
            v-for="member in members"
            :key="member.user_id"
            class="member-row"
          >
            <span class="avatar small-avatar">{{ avatarLabel(member.username) }}</span>
            <span class="member-name">{{ member.username }}</span>
            <span class="member-role">{{ roleLabel(member.role) }}</span>
          </div>
        </template>
        <p v-else class="muted-line">暂无成员信息</p>
      </div>

      <div class="search-row">
        <input
          v-model="addKeyword"
          class="text-input"
          type="search"
          placeholder="搜索并添加成员"
          autocomplete="off"
          :disabled="disabled"
          @keyup.enter="searchAddCandidates"
        />
        <button
          class="icon-button"
          type="button"
          :disabled="disabled"
          @click="searchAddCandidates"
        >
          搜索
        </button>
      </div>

      <p v-if="addError" class="inline-error">{{ addError }}</p>
      <p v-else-if="addLoading" class="muted-line">搜索中...</p>

      <div v-if="addCandidates.length" class="search-results compact-results">
        <button
          v-for="user in addCandidates"
          :key="user.user_id"
          class="search-result"
          type="button"
          :disabled="disabled || isActiveMember(user.user_id)"
          @click="submitAddMember(user.user_id)"
        >
          <span class="avatar">{{ avatarLabel(user.username) }}</span>
          <span>{{ user.username }}</span>
        </button>
      </div>

      <button
        class="secondary-button danger-button"
        type="button"
        :disabled="disabled"
        @click="$emit('leave-group')"
      >
        退出群聊
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { getApiErrorMessage } from "../api/http";
import { searchUsers } from "../api/users";
import type {
  ConversationItem,
  GroupMemberListItem,
  UserSearchItem,
} from "../types/chat";

const props = defineProps<{
  activeConversation: ConversationItem | null;
  members: GroupMemberListItem[];
  loadingMembers?: boolean;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (event: "create-group", name: string, memberUserIds: number[]): void;
  (event: "add-member", userId: number): void;
  (event: "leave-group"): void;
}>();

const groupName = ref("");
const createKeyword = ref("");
const addKeyword = ref("");
const createLoading = ref(false);
const addLoading = ref(false);
const createError = ref("");
const addError = ref("");
const createCandidates = ref<UserSearchItem[]>([]);
const addCandidates = ref<UserSearchItem[]>([]);
const selectedCreateMembers = ref<UserSearchItem[]>([]);

const isGroupActive = computed(
  () => props.activeConversation?.session_type === "group",
);
const activeGroupName = computed(
  () => props.activeConversation?.session_name || "",
);
const activeMemberIds = computed(
  () => new Set(props.members.map((member) => member.user_id)),
);

watch(
  () => props.activeConversation?.session_id,
  () => {
    clearAddSearch();
  },
);

watch(isGroupActive, (active) => {
  if (!active) {
    clearAddSearch();
  }
});

function avatarLabel(username: string) {
  return username.slice(0, 1).toUpperCase();
}

function roleLabel(role: string) {
  return role === "owner" ? "群主" : "成员";
}

function isCreateMemberSelected(userId: number) {
  return selectedCreateMembers.value.some((user) => user.user_id === userId);
}

function isActiveMember(userId: number) {
  return activeMemberIds.value.has(userId);
}

function selectCreateMember(user: UserSearchItem) {
  if (isCreateMemberSelected(user.user_id)) {
    return;
  }

  selectedCreateMembers.value = [...selectedCreateMembers.value, user];
}

function removeCreateMember(userId: number) {
  selectedCreateMembers.value = selectedCreateMembers.value.filter(
    (user) => user.user_id !== userId,
  );
}

async function searchCreateCandidates() {
  const keyword = createKeyword.value.trim();

  if (!keyword) {
    createCandidates.value = [];
    createError.value = "";
    return;
  }

  createLoading.value = true;
  createError.value = "";

  try {
    createCandidates.value = await searchUsers(keyword);
  } catch (error) {
    createError.value = getApiErrorMessage(error);
  } finally {
    createLoading.value = false;
  }
}

async function searchAddCandidates() {
  const keyword = addKeyword.value.trim();

  if (!keyword) {
    addCandidates.value = [];
    addError.value = "";
    return;
  }

  addLoading.value = true;
  addError.value = "";

  try {
    addCandidates.value = await searchUsers(keyword);
  } catch (error) {
    addError.value = getApiErrorMessage(error);
  } finally {
    addLoading.value = false;
  }
}

function clearAddSearch() {
  addKeyword.value = "";
  addCandidates.value = [];
  addError.value = "";
}

function submitCreateGroup() {
  const name = groupName.value.trim();

  if (!name) {
    return;
  }

  emit(
    "create-group",
    name,
    selectedCreateMembers.value.map((user) => user.user_id),
  );
  groupName.value = "";
  createKeyword.value = "";
  createCandidates.value = [];
  selectedCreateMembers.value = [];
}

function submitAddMember(userId: number) {
  emit("add-member", userId);
  clearAddSearch();
}
</script>
