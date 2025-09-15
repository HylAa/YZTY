<template>
  <div class="register-page">
    <div class="container">
      <div class="row justify-content-center">
        <div class="col-md-6 col-lg-5">
          <div class="card shadow-lg">
            <div class="card-header bg-primary text-white text-center py-3">
              <h2 class="mb-0">用户注册</h2>
            </div>
            <div class="card-body p-4">
              <div v-if="error" class="alert alert-danger">{{ error }}</div>
              <form @submit.prevent="handleRegister">
                <div class="mb-3">
                  <label for="username" class="form-label">用户名</label>
                  <input
                    type="text"
                    class="form-control"
                    id="username"
                    v-model="form.username"
                    required
                  />
                </div>
                <div class="mb-3">
                  <label for="email" class="form-label">邮箱</label>
                  <input
                    type="email"
                    class="form-control"
                    id="email"
                    v-model="form.email"
                    required
                  />
                </div>
                <div class="mb-3">
                  <label for="password" class="form-label">密码</label>
                  <input
                    type="password"
                    class="form-control"
                    id="password"
                    v-model="form.password"
                    required
                    minlength="6"
                  />
                </div>
                <div class="mb-3">
                  <label for="confirmPassword" class="form-label"
                    >确认密码</label
                  >
                  <input
                    type="password"
                    class="form-control"
                    id="confirmPassword"
                    v-model="form.confirmPassword"
                    required
                    minlength="6"
                  />
                </div>
                <div class="d-grid gap-2">
                  <button
                    type="submit"
                    class="btn btn-primary"
                    :disabled="loading"
                  >
                    {{ loading ? "注册中..." : "注册" }}
                  </button>
                </div>
              </form>
              <div class="mt-3 text-center">
                <p>
                  已有账号?
                  <router-link to="/login" class="text-decoration-none"
                    >立即登录</router-link
                  >
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from "vue";
import { useStore } from "vuex";
import { useRouter, useRoute } from "vue-router";

export default {
  name: "Register",
  setup() {
    const store = useStore();
    const router = useRouter();
    const route = useRoute();

    const form = ref({
      username: "",
      email: "",
      password: "",
      confirmPassword: "",
    });
    const loading = ref(false);
    const error = ref("");

    const handleRegister = async () => {
      // 验证两次密码是否一致
      if (form.value.password !== form.value.confirmPassword) {
        error.value = "两次输入的密码不一致";
        return;
      }

      loading.value = true;
      error.value = "";

      try {
        const userData = {
          username: form.value.username,
          email: form.value.email,
          password: form.value.password,
        };

        await store.dispatch("auth/register", userData);

        // 注册成功后自动登录
        await store.dispatch("auth/login", {
          email: form.value.email,
          password: form.value.password,
        });

        // 重定向到首页或者请求的页面
        const redirectPath = route.query.redirect || "/";
        router.push(redirectPath);
      } catch (err) {
        error.value = err.response?.data?.message || "注册失败，请稍后重试";
      } finally {
        loading.value = false;
      }
    };

    return {
      form,
      loading,
      error,
      handleRegister,
    };
  },
};
</script>

<style scoped>
.register-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  background-color: #f8f9fa;
  padding: 40px 0;
}
</style>
