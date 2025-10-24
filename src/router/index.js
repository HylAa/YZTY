import { createRouter, createWebHistory } from "vue-router";

// 只导入需要的组件
const PackageDetail = () => import("../views/PackageDetail.vue");
const Home = () => import("../views/Home.vue");
const CourseList = () => import("../views/CourseList.vue");
const VenueBooking = () => import("../views/VenueBooking.vue");
const AdminLogin = () => import("../views/admin/AdminLogin.vue");
const AdminVenueManager = () => import("../views/admin/AdminVenueManager.vue");

const routes = [
  {
    path: "/",
    name: "Home",
    component: Home,
    meta: { title: "我的套餐 - 赢在体育" },
  },
  {
    path: "/courses",
    name: "CourseList",
    component: CourseList,
    meta: { title: "课程列表 - 赢在体育" },
  },
  {
    path: "/booking",
    name: "VenueBooking",
    component: VenueBooking,
    meta: { title: "场馆占用 - 赢在体育" },
  },
  {
    path: "/admin/login",
    name: "AdminLogin",
    component: AdminLogin,
    meta: { title: "管理员登录 - 赢在体育" },
  },
  {
    path: "/admin/venues",
    name: "AdminVenueManager",
    component: AdminVenueManager,
    meta: { title: "场馆占用管理 - 赢在体育", requiresAdmin: true },
  },
  {
    path: "/package/:id",
    name: "PackageDetail",
    component: PackageDetail,
    meta: { title: "套餐详情 - 赢在体育" },
    props: true,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

// 简化导航守卫，增加管理员校验
router.beforeEach((to, from, next) => {
  document.title = to.meta.title || "赢在体育";

  if (to.meta.requiresAdmin) {
    const token = localStorage.getItem("token");
    if (!token) {
      next({ path: "/admin/login", query: { redirect: to.fullPath } });
      return;
    }
  }

  if (to.path === "/admin/login" && localStorage.getItem("token")) {
    next({ path: "/admin/venues" });
    return;
  }

  next();
});

export default router;
