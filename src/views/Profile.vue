<template>
  <div class="page-container">
    <div class="header-bg">
      <van-nav-bar
        title="我的套餐"
        right-text="?"
        left-arrow
        @click-left="goBack"
      />
    </div>

    <div class="user-info">
      <div class="user-phone">
        {{
          user
            ? `${user.phone.substring(0, 3)}****${user.phone.substring(7)}`
            : "未登录"
        }}
        {{ user ? user.name : "" }}
      </div>
      <div class="user-badges">
        <div class="badge">
          <van-icon name="card" />
          <span>全球通银卡</span>
        </div>
        <div class="badge">
          <van-icon name="star" />
          <span>{{ user ? user.memberLevel : "普通会员" }}</span>
        </div>
      </div>
    </div>

    <div class="course-package">
      <div class="package-header">
        <div class="package-title">
          体育课程套餐 (个人版) {{ totalCourses }}课时
        </div>
        <div class="package-price">({{ totalPrice }}元)</div>
      </div>
      <div class="package-actions">
        <div class="action-btn">
          <van-button plain type="primary" size="small" @click="checkRemaining"
            >余量查询</van-button
          >
        </div>
        <div class="action-btn">
          <van-button plain type="primary" size="small" @click="goToCourses"
            >换课程</van-button
          >
        </div>
      </div>
    </div>

    <div class="tab-container">
      <div class="tab-header">
        <div class="tab-item active">课程资源</div>
        <div class="tab-item">融合资源</div>
      </div>

      <van-divider />

      <div class="resource-section">
        <div class="resource-title">
          <div class="resource-icon blue"></div>
          <span>课程</span>
        </div>

        <div v-if="loading" class="loading">加载中...</div>
        <div
          v-else-if="!enrolledCourses || enrolledCourses.length === 0"
          class="empty-courses"
        >
          <van-empty description="暂无课程" />
        </div>
        <div v-else>
          <div
            v-for="course in enrolledCourses"
            :key="course._id"
            class="resource-item"
          >
            <div class="resource-detail">
              <span class="detail-prefix">-</span>
              <span>{{ course.type }} ({{ course.name }})</span>
            </div>
            <div class="resource-count">
              总计: {{ course.totalSessions }}课时
            </div>
          </div>
        </div>
      </div>

      <div class="resource-section">
        <div class="resource-title">
          <div class="resource-icon orange"></div>
          <span>教练</span>
        </div>

        <div v-if="loading" class="loading">加载中...</div>
        <div v-else-if="!coaches || coaches.length === 0" class="empty-courses">
          <div class="resource-item">
            <div class="resource-detail">
              <span class="detail-prefix">-</span>
              <span>专业教练</span>
            </div>
            <div class="resource-count">总计: {{ coachCount }}人</div>
          </div>
        </div>
      </div>

      <div class="resource-section">
        <div class="resource-title">
          <div class="resource-icon green"></div>
          <span>场地预约</span>
        </div>

        <div class="resource-item">
          <div class="resource-detail">
            <span class="detail-prefix">-</span>
            <span>全国场馆网</span>
          </div>
          <div class="resource-count">0/18</div>
        </div>
      </div>
    </div>

    <div class="bottom-actions">
      <div class="action-area">
        <div class="customer-service">
          <van-icon name="service-o" size="24" />
          <span>在线客服</span>
        </div>
        <div class="action-buttons">
          <van-button type="primary" block @click="goToCourses"
            >课程专区</van-button
          >
          <van-button
            v-if="isAdmin"
            type="warning"
            block
            @click="goToAdmin"
            style="margin-top: 10px"
            >管理后台</van-button
          >
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { mapGetters } from "vuex";

export default {
  name: "Profile",
  data() {
    return {
      coaches: [],
      coachCount: 0,
    };
  },
  computed: {
    ...mapGetters(["user", "enrolledCourses", "loading", "error"]),
    totalCourses() {
      if (!this.enrolledCourses || this.enrolledCourses.length === 0) {
        return 0;
      }
      return this.enrolledCourses.reduce(
        (total, course) => total + course.totalSessions,
        0
      );
    },
    totalPrice() {
      if (!this.enrolledCourses || this.enrolledCourses.length === 0) {
        return 0;
      }
      return this.enrolledCourses.reduce(
        (total, course) => total + course.price,
        0
      );
    },
    isAdmin() {
      return this.user && this.user.role === "admin";
    },
  },
  created() {
    this.fetchUserProfile();
    this.fetchEnrolledCourses();
    this.processCoaches();
  },
  methods: {
    async fetchUserProfile() {
      if (this.$store.getters.isAuthenticated) {
        try {
          await this.$store.dispatch("getUserProfile");
        } catch (error) {
          console.error("获取用户资料失败:", error);
        }
      } else {
        this.$router.push("/login");
      }
    },
    async fetchEnrolledCourses() {
      if (this.$store.getters.isAuthenticated) {
        try {
          await this.$store.dispatch("getEnrolledCourses");
          this.processCoaches();
        } catch (error) {
          console.error("获取已报名课程失败:", error);
        }
      }
    },
    processCoaches() {
      if (this.enrolledCourses && this.enrolledCourses.length > 0) {
        // 提取不重复的教练
        const uniqueCoaches = [
          ...new Set(this.enrolledCourses.map((course) => course.coach)),
        ];
        this.coaches = uniqueCoaches.map((coach) => ({ name: coach }));
        this.coachCount = uniqueCoaches.length;
      }
    },
    goBack() {
      this.$router.go(-1);
    },
    goToCourses() {
      this.$router.push("/courses");
    },
    goToAdmin() {
      this.$router.push("/admin/dashboard");
    },
    checkRemaining() {
      this.$router.push("/my-courses");
    },
  },
};
</script>

<style scoped>
.page-container {
  padding-bottom: 60px;
  background-color: #f5f7fa;
}

.header-bg {
  background-color: #1989fa;
  color: white;
  padding-bottom: 10px;
}

:deep(.van-nav-bar) {
  background-color: transparent;
}

:deep(.van-nav-bar__title) {
  color: white;
}

:deep(.van-nav-bar__text) {
  color: white;
}

:deep(.van-icon) {
  color: white;
}

.user-info {
  background-color: #1989fa;
  color: white;
  padding: 0 15px 20px;
}

.user-phone {
  font-size: 18px;
  font-weight: 500;
  margin-bottom: 10px;
}

.user-badges {
  display: flex;
  gap: 10px;
}

.badge {
  display: flex;
  align-items: center;
  background-color: rgba(255, 255, 255, 0.2);
  border-radius: 15px;
  padding: 2px 10px;
  font-size: 12px;
}

.badge .van-icon {
  margin-right: 4px;
  font-size: 14px;
}

.course-package {
  margin: 10px 15px;
  background-color: #1989fa;
  border-radius: 10px;
  padding: 15px;
  color: white;
}

.package-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.package-title {
  font-size: 16px;
  font-weight: 500;
}

.package-price {
  font-size: 14px;
}

.package-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.tab-container {
  background-color: white;
  margin: 10px 0;
  padding: 15px 0;
}

.tab-header {
  display: flex;
  padding: 0 15px;
}

.tab-item {
  padding: 0 15px 10px;
  font-size: 16px;
  position: relative;
  color: #969799;
}

.tab-item.active {
  color: #323233;
  font-weight: 500;
}

.tab-item.active::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 15px;
  width: 20px;
  height: 3px;
  background-color: #1989fa;
  border-radius: 3px;
}

.resource-section {
  padding: 10px 15px;
}

.resource-title {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
  font-size: 16px;
  font-weight: 500;
}

.resource-icon {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 8px;
}

.resource-icon.blue {
  background-color: #1989fa;
}

.resource-icon.orange {
  background-color: #ff976a;
}

.resource-icon.green {
  background-color: #07c160;
}

.resource-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
  font-size: 14px;
  color: #323233;
}

.resource-detail {
  display: flex;
  align-items: center;
}

.detail-prefix {
  margin-right: 8px;
  color: #969799;
}

.resource-count {
  color: #323233;
  font-weight: 500;
}

.empty-courses {
  padding: 20px 0;
  text-align: center;
}

.loading {
  padding: 20px 0;
  text-align: center;
  color: #969799;
}

.bottom-actions {
  position: fixed;
  bottom: 50px;
  left: 0;
  right: 0;
  background-color: white;
  border-top: 1px solid #ebedf0;
}

.action-area {
  display: flex;
  padding: 10px 15px;
}

.customer-service {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  margin-right: 15px;
  font-size: 12px;
  color: #323233;
}

.action-buttons {
  flex: 1;
}
</style>
