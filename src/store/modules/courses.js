import api from "../../api";

const state = {
  courses: [],
  featuredCourses: [],
  course: null,
  loading: false,
  error: null,
  courseTypes: [
    { text: "全部", value: "" },
    { text: "篮球", value: "basketball" },
    { text: "足球", value: "soccer" },
    { text: "游泳", value: "swimming" },
    { text: "网球", value: "tennis" },
    { text: "健身", value: "fitness" },
    { text: "瑜伽", value: "yoga" },
    { text: "其他", value: "other" },
  ],
};

const getters = {
  allCourses: (state) => state.courses,
  featuredCourses: (state) => state.featuredCourses,
  courseById: (state) => state.course,
  coursesLoading: (state) => state.loading,
  coursesError: (state) => state.error,
  courseTypes: (state) => state.courseTypes,
  currentCourse: (state) => state.course,
  coursesByType: (state) => (type) => {
    return state.courses.filter((course) => course.type === type);
  },
};

const actions = {
  // Get all courses
  async fetchCourses({ commit, dispatch }, query = {}) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.courses.getCourses(query);
      commit("SET_COURSES", response.data);
      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch(
        "setError",
        error.response?.data?.message || "获取课程列表失败",
        { root: true }
      );
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Get single course
  async fetchCourse({ commit, dispatch }, id) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.courses.getCourse(id);
      commit("SET_COURSE", response.data);
      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch(
        "setError",
        error.response?.data?.message || "获取课程详情失败",
        { root: true }
      );
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Get featured courses
  async fetchFeaturedCourses({ commit, dispatch }, limit = 5) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.courses.getFeaturedCourses(limit);
      commit("SET_FEATURED_COURSES", response.data);
      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch(
        "setError",
        error.response?.data?.message || "获取推荐课程失败",
        { root: true }
      );
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Get courses by type
  async fetchCoursesByType({ commit, dispatch }, type) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.courses.getCoursesByType(type);
      commit("SET_COURSES", response.data);
      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch(
        "setError",
        error.response?.data?.message || "获取课程列表失败",
        { root: true }
      );
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },

  // Add review to course
  async addReview({ commit, dispatch }, { courseId, reviewData }) {
    try {
      dispatch("setLoading", true, { root: true });
      const response = await api.courses.addReview(courseId, reviewData);
      commit("SET_COURSE", response.data);
      dispatch("setLoading", false, { root: true });
      return response;
    } catch (error) {
      dispatch("setError", error.response?.data?.message || "评价失败", {
        root: true,
      });
      dispatch("setLoading", false, { root: true });
      throw error;
    }
  },
};

const mutations = {
  SET_COURSES(state, courses) {
    state.courses = courses;
  },
  SET_COURSE(state, course) {
    state.course = course;
  },
  SET_FEATURED_COURSES(state, courses) {
    state.featuredCourses = courses;
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
