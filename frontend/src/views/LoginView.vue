<template>
  <main class="auth-page">
    <section class="auth-panel" aria-label="RustChat 登录注册">
      <div class="auth-brand">
        <h1>RustChat</h1>
        <p>登录后开始私聊演示</p>
      </div>

      <form class="auth-form" @submit.prevent="submitLogin">
        <label class="field">
          <span>用户名</span>
          <input
            v-model="username"
            class="text-input"
            type="text"
            autocomplete="username"
            minlength="3"
            maxlength="32"
            required
          />
        </label>

        <label class="field">
          <span>密码</span>
          <input
            v-model="password"
            class="text-input"
            type="password"
            autocomplete="current-password"
            minlength="6"
            maxlength="32"
            required
          />
        </label>

        <p v-if="error" class="form-error">{{ error }}</p>

        <div class="auth-actions">
          <button class="primary-button" type="submit" :disabled="loading">
            {{ loadingMode === "login" ? "登录中..." : "登录" }}
          </button>
          <button
            class="secondary-button"
            type="button"
            :disabled="loading"
            @click="submitRegister"
          >
            {{ loadingMode === "register" ? "注册中..." : "注册" }}
          </button>
        </div>

        <button
          class="ghost-button"
          type="button"
          :disabled="loading"
          @click="enterWithoutAuth"
        >
          跳过验证进入
        </button>
      </form>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { register } from "../api/auth";
import { getApiErrorMessage } from "../api/http";
import { useAuthStore } from "../stores/auth";

const router = useRouter();
const authStore = useAuthStore();

const username = ref("");
const password = ref("");
const error = ref("");
const loadingMode = ref<"login" | "register" | "">("");
const loading = computed(() => Boolean(loadingMode.value));

async function submitLogin() {
  if (loading.value) {
    return;
  }

  loadingMode.value = "login";
  error.value = "";

  try {
    await authStore.login(username.value.trim(), password.value);
    await router.push("/chat");
  } catch (err) {
    error.value = getApiErrorMessage(err);
  } finally {
    loadingMode.value = "";
  }
}

async function submitRegister() {
  if (loading.value) {
    return;
  }

  loadingMode.value = "register";
  error.value = "";

  try {
    await register(username.value.trim(), password.value);
    await authStore.login(username.value.trim(), password.value);
    await router.push("/chat");
  } catch (err) {
    error.value = getApiErrorMessage(err);
  } finally {
    loadingMode.value = "";
  }
}

async function enterWithoutAuth() {
  authStore.enterBypassMode();
  await router.push("/chat");
}
</script>
