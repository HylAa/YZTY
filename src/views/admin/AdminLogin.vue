<template>
  <div class="admin-login-page">
    <div class="login-card">
      <h1>管理员登录</h1>
      <p class="subtitle">请输入管理员账号密码访问后台</p>

      <form class="login-form" @submit.prevent="handleSubmit">
        <label class="form-item">
          <span>用户名</span>
          <input
            v-model.trim="username"
            type="text"
            name="username"
            autocomplete="username"
            placeholder="输入管理员账号"
            required
          />
        </label>
        <label class="form-item">
          <span>密码</span>
          <input
            v-model.trim="password"
            type="password"
            name="password"
            autocomplete="current-password"
            placeholder="输入登录密码"
            required
          />
        </label>
        <button class="submit-btn" type="submit" :disabled="loading">
          {{ loading ? "正在登录..." : "登录" }}
        </button>
      </form>

      <p v-if="error" class="error-text">{{ error }}</p>
      <p v-if="hint" class="hint-text">{{ hint }}</p>
    </div>
  </div>
</template>

<script>
import { ref } from "vue";
import api from "../../api";

export default {
  name: "AdminLogin",
  setup() {
    const username = ref("");
    const password = ref("");
    const loading = ref(false);
    const error = ref("");
    const hint = ref("");

    const handleSubmit = async () => {
      if (!username.value || !password.value) {
        error.value = "请输入完整的账号密码";
        return;
      }
      loading.value = true;
      error.value = "";
      hint.value = "";
      try {
        const response = await api.auth.login({
          username: username.value,
          password: password.value,
        });
        if (!response || response.code !== 0) {
          throw new Error(response?.message || "登录失败");
        }
        const payload = response.data || {};
        localStorage.setItem("token", payload.token);
        localStorage.setItem("adminUser", JSON.stringify(payload.user || {}));
        localStorage.setItem("user", JSON.stringify(payload.user || {}));
        hint.value = "登录成功，正在跳转...";
        window.location.href = "/admin/venues";
      } catch (err) {
        console.error("管理员登录失败", err);
        error.value = err?.message || "登录失败，请稍后重试";
      } finally {
        loading.value = false;
      }
    };

    return {
      username,
      password,
      loading,
      error,
      hint,
      handleSubmit,
    };
  },
};
</script>

<style scoped>
.admin-login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #e0f2f1, #f0f7ff);
  padding: 20px;
}

.login-card {
  width: 100%;
  max-width: 360px;
  background: #fff;
  padding: 32px 28px;
  border-radius: 16px;
  box-shadow: 0 12px 32px rgba(33, 150, 243, 0.16);
}

.login-card h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
  text-align: center;
  color: #0f172a;
}

.subtitle {
  margin: 8px 0 24px;
  font-size: 13px;
  text-align: center;
  color: #64748b;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
  color: #0f172a;
}

.form-item input {
  width: 100%;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid #d0d7df;
  font-size: 14px;
  transition: border-color 0.2s ease;
}

.form-item input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.16);
}

.submit-btn {
  margin-top: 12px;
  width: 100%;
  padding: 12px;
  border: none;
  border-radius: 999px;
  background: linear-gradient(135deg, #0f9d58, #0bb172);
  color: #fff;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.submit-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.error-text {
  margin-top: 16px;
  font-size: 13px;
  color: #d93025;
  text-align: center;
}

.hint-text {
  margin-top: 16px;
  font-size: 13px;
  color: #0f9d58;
  text-align: center;
}
</style>
