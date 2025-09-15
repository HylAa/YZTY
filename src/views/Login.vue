<template>
  <div class="page-container">
    <van-nav-bar title="用户登录" left-arrow @click-left="goBack" />

    <div class="login-form">
      <div class="logo">
        <img src="https://img.yzcdn.cn/vant/cat.jpeg" alt="Logo" />
      </div>

      <van-cell-group inset>
        <van-cell>
          <template #title>
            <div class="input-wrapper">
              <van-icon name="phone-o" />
              <input
                v-model="phone"
                type="tel"
                placeholder="请输入手机号"
                class="custom-input"
              />
            </div>
          </template>
        </van-cell>

        <van-cell>
          <template #title>
            <div class="input-wrapper">
              <van-icon name="lock" />
              <input
                v-model="password"
                type="password"
                placeholder="请输入密码"
                class="custom-input"
              />
            </div>
          </template>
        </van-cell>
      </van-cell-group>

      <div class="form-actions">
        <van-button type="primary" block @click="handleLogin" :loading="loading"
          >登录</van-button
        >
      </div>

      <div class="form-links">
        <router-link to="/register">注册账号</router-link>
        <span class="divider">|</span>
        <a href="#">忘记密码</a>
      </div>

      <div v-if="error" class="error-message">{{ error }}</div>
    </div>

    <div class="other-login">
      <div class="other-title">其他登录方式</div>
      <div class="other-options">
        <div class="other-option">
          <van-icon name="wechat" size="28" color="#07c160" />
          <div>微信</div>
        </div>
        <div class="other-option">
          <van-icon name="phone-circle-o" size="28" color="#ee0a24" />
          <div>手机号</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { mapGetters } from "vuex";

export default {
  name: "Login",
  data() {
    return {
      phone: "",
      password: "",
    };
  },
  computed: {
    ...mapGetters(["loading", "error"]),
  },
  methods: {
    goBack() {
      this.$router.go(-1);
    },
    async handleLogin() {
      if (!this.validateForm()) {
        return;
      }

      try {
        await this.$store.dispatch("login", {
          phone: this.phone,
          password: this.password,
        });

        // 登录成功，重定向到原来的页面或首页
        const redirectPath = this.$route.query.redirect || "/profile";
        this.$router.push(redirectPath);
      } catch (error) {
        console.error("登录失败:", error);
      }
    },
    validateForm() {
      if (!this.phone) {
        this.$store.commit("SET_ERROR", "请输入手机号");
        return false;
      }

      if (!this.password) {
        this.$store.commit("SET_ERROR", "请输入密码");
        return false;
      }

      return true;
    },
  },
};
</script>

<style scoped>
.page-container {
  padding-bottom: 50px;
  background-color: #fff;
  min-height: 100vh;
}

.login-form {
  padding: 20px 15px;
}

.logo {
  text-align: center;
  margin: 30px 0;
}

.logo img {
  width: 100px;
  height: 100px;
  border-radius: 50%;
  object-fit: cover;
}

.input-wrapper {
  display: flex;
  align-items: center;
}

.input-wrapper .van-icon {
  margin-right: 10px;
  color: #969799;
}

.custom-input {
  border: none;
  outline: none;
  width: 100%;
  font-size: 16px;
  padding: 10px 0;
}

.form-actions {
  margin-top: 30px;
}

.form-links {
  margin-top: 15px;
  text-align: center;
  font-size: 14px;
}

.form-links a {
  color: #1989fa;
  text-decoration: none;
}

.divider {
  margin: 0 10px;
  color: #dcdee0;
}

.error-message {
  margin-top: 15px;
  color: #ee0a24;
  text-align: center;
  font-size: 14px;
}

.other-login {
  margin-top: 50px;
  padding: 0 15px;
}

.other-title {
  text-align: center;
  color: #969799;
  font-size: 14px;
  margin-bottom: 20px;
  position: relative;
}

.other-title::before,
.other-title::after {
  content: "";
  position: absolute;
  top: 50%;
  width: 20%;
  height: 1px;
  background-color: #ebedf0;
}

.other-title::before {
  left: 15%;
}

.other-title::after {
  right: 15%;
}

.other-options {
  display: flex;
  justify-content: center;
  gap: 40px;
}

.other-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  font-size: 12px;
  color: #323233;
}

.other-option .van-icon {
  margin-bottom: 8px;
}
</style>
