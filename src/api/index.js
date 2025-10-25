import axios from "axios";

// API base URL
const API_URL = import.meta.env.VITE_API_BASE ? `${import.meta.env.VITE_API_BASE}/api` : "/api";

// Create axios instance
const apiClient = axios.create({
  baseURL: API_URL,
  headers: {
    "Content-Type": "application/json",
  },
});

// 微信相关接口使用独立实例，避免受 `/api` 前缀影响
const wechatBaseUrl = import.meta.env.VITE_API_BASE || "";
const wechatClient = axios.create({
  baseURL: wechatBaseUrl,
  headers: {
    "Content-Type": "application/json",
  },
});

// Add request interceptor for authentication
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem("token");
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Add response interceptor for error handling
apiClient.interceptors.response.use(
  (response) => {
    return response.data;
  },
  (error) => {
    // Handle token expiration
    if (error.response && error.response.status === 401) {
      // Clear local storage
      localStorage.removeItem("token");
      localStorage.removeItem("user");
      localStorage.removeItem("adminUser");

      const currentPath = window.location.pathname || "";
      const isAdmin = currentPath.startsWith("/admin");
      const targetPath = isAdmin ? "/admin/login" : "/login";

      if (currentPath !== targetPath) {
        window.location.href = targetPath;
      }
    }
    return Promise.reject(error);
  }
);

wechatClient.interceptors.response.use(
  (response) => response.data,
  (error) => Promise.reject(error)
);

export default {
  // Auth endpoints
  auth: {
    login: (credentials) => apiClient.post("/auth/login", credentials),
    register: (userData) => apiClient.post("/auth/register", userData),
    getCurrentUser: () => apiClient.get("/auth/me"),
    logout: () => apiClient.get("/auth/logout"),
  },

  // User endpoints
  user: {
    getUser: (id) => apiClient.get(`/users/${id}`),
    updateUser: (id, userData) => apiClient.put(`/users/${id}`, userData),
    enrollCourse: (userId, courseId) =>
      apiClient.post(`/users/${userId}/enroll/${courseId}`),
    updateRemainingSessions: (userId, courseId, remainingSessions) =>
      apiClient.put(`/users/${userId}/courses/${courseId}/sessions`, {
        remainingSessions,
      }),
  },

  // Course endpoints
  courses: {
    getCourses: (params) => apiClient.get("/courses", { params }),
    getCourse: (id) => apiClient.get(`/courses/${id}`),
    getFeaturedCourses: (limit) =>
      apiClient.get("/courses/featured", { params: { limit } }),
    getCoursesByType: (type) => apiClient.get(`/courses/types/${type}`),
    addReview: (courseId, reviewData) =>
      apiClient.post(`/courses/${courseId}/reviews`, reviewData),
  },

  // 学员接口
  student: {
    getCoursesByPhone: (phone) =>
      apiClient.get("/student/courses", { params: { phone } }),
  },

  // 游泳课程接口
  swim: {
    getCoursesByPhone: (phone) =>
      apiClient.get("/swim/courses", { params: { phone } }),
  },

  // 场馆占用
  venue: {
    getOverview: (date) =>
      apiClient.get("/venues/overview", {
        params: date ? { date } : undefined,
      }),
    getAvailability: ({ sport, date, startTime, endTime }) =>
      apiClient.get("/venues/availability", {
        params: { sport, date, startTime, endTime },
      }),
  },

  adminVenue: {
    updateStatus: (payload) => apiClient.post("/admin/venues/status", payload),
  },

  // WeChat endpoints
  wechat: {
    getJssdkConfig: (url) => wechatClient.post("/wechat/jssdkConfig", { url }),
    getUserInfoByCode: (code) =>
      wechatClient.post("/wechat/getUserInfo", { code }),
    bindPhone: (payload) => wechatClient.post("/wechat/bindPhone", payload),
    decryptPhoneNumber: (payload) =>
      wechatClient.post("/wechat/decryptPhoneNumber", payload),
  },
};
