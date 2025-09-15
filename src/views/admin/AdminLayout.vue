<template>
  <div class="admin-layout">
    <div class="page-content">
      <router-view />
    </div>

    <van-tabbar route>
      <van-tabbar-item to="/admin/dashboard" icon="chart-trending-o">
        仪表盘
      </van-tabbar-item>

      <van-tabbar-item to="/admin/users" icon="friends-o">
        用户管理
      </van-tabbar-item>

      <van-tabbar-item to="/admin/courses" icon="description">
        课程管理
      </van-tabbar-item>

      <van-tabbar-item to="/profile" icon="setting-o"> 设置 </van-tabbar-item>
    </van-tabbar>
  </div>
</template>

<script>
export default {
  name: "AdminLayout",
};
</script>

<style scoped>
.admin-layout {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.page-content {
  flex: 1;
  padding-bottom: 50px;
}
</style>
