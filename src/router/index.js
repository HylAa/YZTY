import { createRouter, createWebHistory } from "vue-router";

// 只导入需要的组件
const PackageDetail = () => import("../views/PackageDetail.vue");
const Home = () => import("../views/Home.vue");
const CourseList = () => import("../views/CourseList.vue");

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

// 简化导航守卫，只更新页面标题
router.beforeEach((to, from, next) => {
  // 更新页面标题
  document.title = to.meta.title || "赢在体育";
  next();
});

export default router;
