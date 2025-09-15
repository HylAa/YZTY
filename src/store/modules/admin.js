import axios from "axios";

const state = {
  dashboardStats: null,
  users: [],
  adminCourses: [],
  pagination: {
    users: {
      page: 1,
      limit: 10,
      total: 0,
      pages: 0,
    },
    courses: {
      page: 1,
      limit: 10,
      total: 0,
      pages: 0,
    },
  },
  loading: false,
  error: null,
};

const getters = {
  dashboardStats: (state) => state.dashboardStats,
  users: (state) => state.users,
  adminCourses: (state) => state.adminCourses,
  pagination: (state) => state.pagination,
  loading: (state) => state.loading,
  error: (state) => state.error,
};

const actions = {
  // 获取管理员仪表盘统计数据
  async getDashboardStats({ commit, rootState }) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const { data } = await axios.get("/admin/dashboard");

      commit("SET_DASHBOARD_STATS", data.data);
      return data.data;
    } catch (error) {
      commit(
        "SET_ERROR",
        error.response?.data?.message || "获取仪表盘数据失败"
      );
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },

  // 获取所有用户
  async getUsers(
    { commit, rootState },
    { page = 1, limit = 10, search = "" } = {}
  ) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const { data } = await axios.get("/admin/users", {
        params: { page, limit, search },
      });

      commit("SET_USERS", data.data);
      commit("SET_USER_PAGINATION", {
        page: data.pagination.page,
        limit: data.pagination.limit,
        total: data.total,
        pages: data.pagination.pages,
      });

      return data;
    } catch (error) {
      commit("SET_ERROR", error.response?.data?.message || "获取用户列表失败");
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },

  // 更新用户
  async updateUser({ commit, rootState, dispatch }, { id, userData }) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const { data } = await axios.put(`/admin/users/${id}`, userData);

      // 刷新用户列表
      await dispatch("getUsers", {
        page: state.pagination.users.page,
        limit: state.pagination.users.limit,
      });

      return data;
    } catch (error) {
      commit("SET_ERROR", error.response?.data?.message || "更新用户失败");
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },

  // 删除用户
  async deleteUser({ commit, rootState, dispatch }, id) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const { data } = await axios.delete(`/admin/users/${id}`);

      // 刷新用户列表
      await dispatch("getUsers", {
        page: state.pagination.users.page,
        limit: state.pagination.users.limit,
      });

      return data;
    } catch (error) {
      commit("SET_ERROR", error.response?.data?.message || "删除用户失败");
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },

  // 获取所有课程（管理员视图）
  async getAdminCourses(
    { commit, rootState },
    { page = 1, limit = 10, search = "", type = "" } = {}
  ) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const params = { page, limit };
      if (search) params.search = search;
      if (type) params.type = type;

      const { data } = await axios.get("/admin/courses", { params });

      commit("SET_ADMIN_COURSES", data.data);
      commit("SET_COURSE_PAGINATION", {
        page: data.pagination.page,
        limit: data.pagination.limit,
        total: data.total,
        pages: data.pagination.pages,
      });

      return data;
    } catch (error) {
      commit("SET_ERROR", error.response?.data?.message || "获取课程列表失败");
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },

  // 创建课程
  async createCourse({ commit, rootState, dispatch }, courseData) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const { data } = await axios.post("/admin/courses", courseData);

      // 刷新课程列表
      await dispatch("getAdminCourses", {
        page: state.pagination.courses.page,
        limit: state.pagination.courses.limit,
      });

      return data;
    } catch (error) {
      commit("SET_ERROR", error.response?.data?.message || "创建课程失败");
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },

  // 更新课程
  async updateCourse({ commit, rootState, dispatch }, { id, courseData }) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const { data } = await axios.put(`/admin/courses/${id}`, courseData);

      // 刷新课程列表
      await dispatch("getAdminCourses", {
        page: state.pagination.courses.page,
        limit: state.pagination.courses.limit,
      });

      return data;
    } catch (error) {
      commit("SET_ERROR", error.response?.data?.message || "更新课程失败");
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },

  // 删除课程
  async deleteCourse({ commit, rootState, dispatch }, id) {
    try {
      commit("SET_LOADING", true);
      commit("CLEAR_ERROR");

      // 配置请求头
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${rootState.auth.token}`;

      const { data } = await axios.delete(`/admin/courses/${id}`);

      // 刷新课程列表
      await dispatch("getAdminCourses", {
        page: state.pagination.courses.page,
        limit: state.pagination.courses.limit,
      });

      return data;
    } catch (error) {
      commit("SET_ERROR", error.response?.data?.message || "删除课程失败");
      throw error;
    } finally {
      commit("SET_LOADING", false);
    }
  },
};

const mutations = {
  SET_DASHBOARD_STATS(state, stats) {
    state.dashboardStats = stats;
  },
  SET_USERS(state, users) {
    state.users = users;
  },
  SET_ADMIN_COURSES(state, courses) {
    state.adminCourses = courses;
  },
  SET_USER_PAGINATION(state, pagination) {
    state.pagination.users = pagination;
  },
  SET_COURSE_PAGINATION(state, pagination) {
    state.pagination.courses = pagination;
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
};

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
