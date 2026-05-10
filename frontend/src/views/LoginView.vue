<template>
  <main class="auth-page mongo-auth-page">
    <section class="auth-panel mongo-auth-panel" aria-label="Rust Chat 登录注册">
      <div class="auth-brand mongo-auth-brand">
        <div class="brand-lockup large">
          <span class="brand-leaf" aria-hidden="true"></span>
          <span>Rust Chat</span>
        </div>
        <h1>MongoDB-style community chat for builders.</h1>
        <p>Sign in to open your workspace, join community rooms, and keep private conversations moving in real time.</p>
      </div>

      <form class="auth-form" @submit.prevent="submitLogin">
        <label class="field">
          <span>Username</span>
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
          <span>Password</span>
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
            {{ loadingMode === "login" ? "Signing in..." : "Sign in" }}
          </button>
          <button
            class="secondary-button"
            type="button"
            :disabled="loading"
            @click="submitRegister"
          >
            {{ loadingMode === "register" ? "Creating..." : "Create account" }}
          </button>
        </div>

        <button
          class="ghost-button"
          type="button"
          :disabled="loading"
          @click="enterWithoutAuth"
        >
          Continue with demo workspace
        </button>
      </form>

      <div class="auth-proof" aria-label="Product highlights">
        <span>Atlas discussions</span>
        <span>Private Room</span>
        <span>Realtime messages</span>
      </div>
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
