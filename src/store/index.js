import { createStore } from "vuex";
import axios from "axios";
import auth from "./modules/auth";
import courses from "./modules/courses";
import user from "./modules/user";
import admin from "./modules/admin";

// API基础URL
const API_URL = process.env.VUE_APP_API_URL || "http://localhost:5000/api";

// 配置axios
axios.defaults.baseURL = API_URL;

export default createStore({
  state: {
    token: localStorage.getItem("token") || "",
    user: JSON.parse(localStorage.getItem("user")) || null,
    courses: [],
    enrolledCourses: [],
    loading: false,
    error: null,
  },
  getters: {
    isAuthenticated: (state) => !!state.token,
    user: (state) => state.user,
    courses: (state) => state.courses,
    enrolledCourses: (state) => state.enrolledCourses,
    loading: (state) => state.loading,
    error: (state) => state.error,
  },
  mutations: {
    SET_TOKEN(state, token) {
      state.token = token;
    },
    SET_USER(state, user) {
      state.user = user;
    },
    SET_COURSES(state, courses) {
      state.courses = courses;
    },
    SET_ENROLLED_COURSES(state, courses) {
      state.enrolledCourses = courses;
    },
    SET_LOADING(state, status) {
      state.loading = status;
    },
    SET_ERROR(state, error) {
      state.error = error;
    },
    CLEAR_ERROR(state) {
      state.error = null;
    },
    LOGOUT(state) {
      state.token = "";
      state.user = null;
      state.enrolledCourses = [];
    },
  },
  actions: {
    // 用户登录
    async login({ commit }, { phone, password }) {
      try {
        commit("SET_LOADING", true);
        commit("CLEAR_ERROR");

        const { data } = await axios.post("/users/login", { phone, password });

        // 保存token和用户信息
        localStorage.setItem("token", data.token);
        localStorage.setItem(
          "user",
          JSON.stringify({
            id: data._id,
            phone: data.phone,
            name: data.name,
            memberLevel: data.memberLevel,
          })
        );

        commit("SET_TOKEN", data.token);
        commit("SET_USER", {
          id: data._id,
          phone: data.phone,
          name: data.name,
          memberLevel: data.memberLevel,
        });

        // 配置axios请求头
        axios.defaults.headers.common["Authorization"] = `Bearer ${data.token}`;

        return data;
      } catch (error) {
        commit("SET_ERROR", error.response?.data?.message || "登录失败");
        throw error;
      } finally {
        commit("SET_LOADING", false);
      }
    },

    // 用户注册
    async register({ commit }, userData) {
      try {
        commit("SET_LOADING", true);
        commit("CLEAR_ERROR");

        const { data } = await axios.post("/users", userData);

        // 保存token和用户信息
        localStorage.setItem("token", data.token);
        localStorage.setItem(
          "user",
          JSON.stringify({
            id: data._id,
            phone: data.phone,
            name: data.name,
            memberLevel: data.memberLevel,
          })
        );

        commit("SET_TOKEN", data.token);
        commit("SET_USER", {
          id: data._id,
          phone: data.phone,
          name: data.name,
          memberLevel: data.memberLevel,
        });

        // 配置axios请求头
        axios.defaults.headers.common["Authorization"] = `Bearer ${data.token}`;

        return data;
      } catch (error) {
        commit("SET_ERROR", error.response?.data?.message || "注册失败");
        throw error;
      } finally {
        commit("SET_LOADING", false);
      }
    },

    // 用户登出
    logout({ commit }) {
      localStorage.removeItem("token");
      localStorage.removeItem("user");
      delete axios.defaults.headers.common["Authorization"];
      commit("LOGOUT");
    },

    // 获取用户资料
    async getUserProfile({ commit, state }) {
      try {
        commit("SET_LOADING", true);
        commit("CLEAR_ERROR");

        // 配置请求头
        axios.defaults.headers.common[
          "Authorization"
        ] = `Bearer ${state.token}`;

        const { data } = await axios.get("/users/profile");

        // 更新用户信息
        const user = {
          id: data._id,
          phone: data.phone,
          name: data.name,
          memberLevel: data.memberLevel,
          avatar: data.avatar,
        };

        localStorage.setItem("user", JSON.stringify(user));
        commit("SET_USER", user);

        return data;
      } catch (error) {
        commit(
          "SET_ERROR",
          error.response?.data?.message || "获取用户资料失败"
        );
        throw error;
      } finally {
        commit("SET_LOADING", false);
      }
    },

    // 获取课程列表
    async getCourses({ commit }, { type, keyword } = {}) {
      try {
        commit("SET_LOADING", true);
        commit("CLEAR_ERROR");

        let url = "/courses";
        const params = {};

        if (type) params.type = type;
        if (keyword) params.keyword = keyword;

        const { data } = await axios.get(url, { params });

        commit("SET_COURSES", data);
        return data;
      } catch (error) {
        commit(
          "SET_ERROR",
          error.response?.data?.message || "获取课程列表失败"
        );
        throw error;
      } finally {
        commit("SET_LOADING", false);
      }
    },

    // 获取已报名的课程
    async getEnrolledCourses({ commit, state }) {
      try {
        commit("SET_LOADING", true);
        commit("CLEAR_ERROR");

        // 配置请求头
        axios.defaults.headers.common[
          "Authorization"
        ] = `Bearer ${state.token}`;

        const { data } = await axios.get("/courses/enrolled");

        commit("SET_ENROLLED_COURSES", data);
        return data;
      } catch (error) {
        commit(
          "SET_ERROR",
          error.response?.data?.message || "获取已报名课程失败"
        );
        throw error;
      } finally {
        commit("SET_LOADING", false);
      }
    },

    // 报名课程
    async enrollCourse({ commit, state, dispatch }, courseId) {
      try {
        commit("SET_LOADING", true);
        commit("CLEAR_ERROR");

        // 配置请求头
        axios.defaults.headers.common[
          "Authorization"
        ] = `Bearer ${state.token}`;

        await axios.post(`/courses/${courseId}/enroll`);

        // 重新获取已报名的课程
        await dispatch("getEnrolledCourses");

        return true;
      } catch (error) {
        commit("SET_ERROR", error.response?.data?.message || "报名课程失败");
        throw error;
      } finally {
        commit("SET_LOADING", false);
      }
    },

    setLoading({ commit }, status) {
      commit("SET_LOADING", status);
    },

    setError({ commit }, error) {
      commit("SET_ERROR", error);
    },

    clearError({ commit }) {
      commit("CLEAR_ERROR");
    },
  },
  modules: {
    auth,
    courses,
    user,
    admin,
  },
});
