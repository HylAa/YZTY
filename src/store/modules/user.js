import api from "../../api";
import { Toast } from "vant";

const state = {
  enrolledCourses: [],
  loading: false,
  error: null,
};

const getters = {
  userCourses: (state) => state.enrolledCourses,
  userCoursesLoading: (state) => state.loading,
  userCoursesError: (state) => state.error,
  enrolledCourses: (state) => state.enrolledCourses,
  isEnrolled: (state) => (courseId) => {
    return state.enrolledCourses.some(
      (enrollment) => enrollment.course._id === courseId
    );
  },
  getRemainingSessionsForCourse: (state) => (courseId) => {
    const enrollment = state.enrolledCourses.find(
      (enrollment) => enrollment.course._id === courseId
    );
    return enrollment ? enrollment.remainingSessions : 0;
  },
};

const actions = {
  // Get user's enrolled courses
  async fetchEnrolledCourses({ commit, dispatch, rootGetters }) {
    try {
      // Check if user is authenticated
      if (!rootGetters["auth/isAuthenticated"]) {
        return;
      }

      dispatch("setLoading", true, { root: true });
      const response = await api.auth.getCurrentUser();

      // Extract enrolled courses from user data
      if (response.data && response.data.enrolledCourses) {
        commit("SET_ENROLLED_COURSES", response.data.enrolledCourses);
      }

      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch(
        "setError",
        error.response?.data?.message || "获取已报名课程失败",
        { root: true }
      );
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Enroll in a course
  async enrollCourse({ dispatch, rootGetters }, courseId) {
    try {
      // Check if user is authenticated
      if (!rootGetters["auth/isAuthenticated"]) {
        throw new Error("请先登录");
      }

      const userId = rootGetters["auth/user"].id;

      dispatch("setLoading", true, { root: true });
      const response = await api.users.enrollCourse(userId, courseId);

      // Update enrolled courses
      await dispatch("fetchEnrolledCourses");

      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch("setError", error.response?.data?.message || "报名课程失败", {
        root: true,
      });
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Update remaining sessions for a course
  async updateRemainingSessions(
    { dispatch, rootGetters },
    { courseId, remainingSessions }
  ) {
    try {
      // Check if user is authenticated
      if (!rootGetters["auth/isAuthenticated"]) {
        throw new Error("请先登录");
      }

      const userId = rootGetters["auth/user"].id;

      dispatch("setLoading", true, { root: true });
      const response = await api.users.updateRemainingSessions(
        userId,
        courseId,
        remainingSessions
      );

      // Update enrolled courses
      await dispatch("fetchEnrolledCourses");

      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch("setError", error.response?.data?.message || "更新课时失败", {
        root: true,
      });
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Update user profile
  async updateProfile({ commit, rootState }, userData) {
    try {
      commit("setLoading", true);

      const token = rootState.auth.token;
      if (!token) {
        throw new Error("No authentication token");
      }

      const response = await api.users.updateProfile(userData);

      if (response.data.success) {
        // Update user in auth module
        commit("auth/setUser", response.data.user, { root: true });

        // Update localStorage
        localStorage.setItem("user", JSON.stringify(response.data.user));

        Toast.success("个人资料已更新");
        return true;
      }
    } catch (error) {
      console.error("Failed to update profile:", error);
      const message = error.response?.data?.message || "更新失败，请稍后再试";
      commit("setError", message);
      Toast.fail(message);
      return false;
    } finally {
      commit("setLoading", false);
    }
  },
};

const mutations = {
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
};

export default {
  namespaced: true,
  state,
  getters,
  actions,
  mutations,
};
