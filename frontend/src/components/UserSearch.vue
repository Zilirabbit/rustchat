<template>
  <section class="user-search">
    <label class="field-label" for="user-search">搜索用户</label>
    <div class="search-row">
      <input
        id="user-search"
        v-model="keyword"
        class="text-input"
        type="search"
        placeholder="输入用户名"
        autocomplete="off"
        :disabled="disabled"
        @keyup.enter="runSearch"
      />
      <button class="icon-button" type="button" :disabled="disabled" @click="runSearch">
        搜索
      </button>
    </div>

    <p v-if="error" class="inline-error">{{ error }}</p>
    <p v-else-if="loading" class="muted-line">搜索中...</p>

    <div v-if="results.length" class="search-results">
      <button
        v-for="user in results"
        :key="user.user_id"
        class="search-result"
        type="button"
        :disabled="disabled"
        @click="$emit('select-user', user.user_id)"
      >
        <span class="avatar">{{ user.username.slice(0, 1).toUpperCase() }}</span>
        <span>{{ user.username }}</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { getApiErrorMessage } from "../api/http";
import { searchUsers } from "../api/users";
import type { UserSearchItem } from "../types/chat";

defineProps<{
  disabled?: boolean;
}>();

defineEmits<{
  (event: "select-user", userId: number): void;
}>();

const keyword = ref("");
const loading = ref(false);
const error = ref("");
const results = ref<UserSearchItem[]>([]);

async function runSearch() {
  const trimmedKeyword = keyword.value.trim();

  if (!trimmedKeyword) {
    results.value = [];
    error.value = "";
    return;
  }

  loading.value = true;
  error.value = "";

  try {
    results.value = await searchUsers(trimmedKeyword);
  } catch (err) {
    error.value = getApiErrorMessage(err);
  } finally {
    loading.value = false;
  }
}
</script>
