import api from "../../api";
import router from "../../router";

// Get user from localStorage
const user = localStorage.getItem("user")
  ? JSON.parse(localStorage.getItem("user"))
  : null;
const token = localStorage.getItem("token");

const state = {
  user: user,
  token: token,
  isAuthenticated: !!token,
};

const mutations = {
  SET_USER(state, user) {
    state.user = user;
  },
  SET_TOKEN(state, token) {
    state.token = token;
    state.isAuthenticated = !!token;
  },
  LOGOUT(state) {
    state.user = null;
    state.token = null;
    state.isAuthenticated = false;
  },
};

const actions = {
  // Register user
  async register({ commit, dispatch }, userData) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.auth.register(userData);

      // Save to localStorage
      localStorage.setItem("token", response.token);
      localStorage.setItem("user", JSON.stringify(response.user));

      // Update state
      commit("SET_USER", response.user);
      commit("SET_TOKEN", response.token);

      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch("setError", error.response?.data?.message || "注册失败", {
        root: true,
      });
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Login user
  async login({ commit, dispatch }, userData) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.auth.login(userData);

      // Save to localStorage
      localStorage.setItem("token", response.token);
      localStorage.setItem("user", JSON.stringify(response.user));

      // Update state
      commit("SET_USER", response.user);
      commit("SET_TOKEN", response.token);

      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch("setError", error.response?.data?.message || "登录失败", {
        root: true,
      });
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Get current user
  async getCurrentUser({ commit, dispatch }) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.auth.getCurrentUser();

      // Update state
      commit("SET_USER", response.data);

      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch(
        "setError",
        error.response?.data?.message || "获取用户信息失败",
        { root: true }
      );
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Logout user
  async logout({ commit, dispatch }) {
    try {
      dispatch("setLoading", true, { root: true });
      await api.auth.logout();

      // Remove from localStorage
      localStorage.removeItem("token");
      localStorage.removeItem("user");

      // Update state
      commit("LOGOUT");

      // Redirect to login
      router.push("/login");

      dispatch("setLoading", false, { root: true });
    } catch (error) {
      dispatch("setError", error.response?.data?.message || "退出登录失败", {
        root: true,
      });
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },
};

const getters = {
  user: (state) => state.user,
  token: (state) => state.token,
  isAuthenticated: (state) => state.isAuthenticated,
};

export default {
  namespaced: true,
  state,
  mutations,
  actions,
  getters,
};
