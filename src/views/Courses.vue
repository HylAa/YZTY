<template>
  <div class="page-container">
    <van-nav-bar title="课程列表" left-arrow @click-left="goBack" />

    <div class="filter-bar">
      <div class="search-box">
        <van-icon name="search" />
        <input
          v-model="keyword"
          type="text"
          placeholder="搜索课程"
          class="search-input"
          @keyup.enter="searchCourses"
        />
      </div>

      <div class="filter-tabs">
        <div
          v-for="(tab, index) in filterTabs"
          :key="index"
          :class="['filter-tab', { active: currentTab === tab.value }]"
          @click="selectTab(tab.value)"
        >
          {{ tab.name }}
        </div>
      </div>
    </div>

    <div v-if="loading" class="loading-container">
      <div class="loading">加载中...</div>
    </div>

    <div v-else-if="filteredCourses.length === 0" class="empty-container">
      <van-empty description="暂无课程" />
    </div>

    <div v-else class="course-list">
      <div
        v-for="course in filteredCourses"
        :key="course._id"
        class="course-card"
        @click="goToCourseDetail(course._id)"
      >
        <div class="course-image">
          <van-image
            width="100%"
            height="150"
            :src="course.image || 'https://img.yzcdn.cn/vant/cat.jpeg'"
          />
          <div class="course-tag">{{ course.type }}</div>
        </div>
        <div class="course-content">
          <h3 class="course-title">{{ course.name }}</h3>
          <div class="course-info">
            <div class="info-item">
              <van-icon name="user-o" />
              <span>{{ course.coach }}</span>
            </div>
            <div class="info-item">
              <van-icon name="location-o" />
              <span>{{ course.location }}</span>
            </div>
            <div class="info-item">
              <van-icon name="clock-o" />
              <span>{{ course.duration }}分钟/节</span>
            </div>
            <div class="info-item">
              <van-icon name="calendar-o" />
              <span>{{ formatDate(course.startDate) }} 开始</span>
            </div>
          </div>
          <div class="course-footer">
            <div class="course-price">¥{{ course.price }}</div>
            <van-button
              size="small"
              type="primary"
              @click.stop="enrollCourse(course._id)"
              >报名</van-button
            >
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { mapGetters } from "vuex";

export default {
  name: "Courses",
  data() {
    return {
      keyword: "",
      currentTab: "",
      filterTabs: [
        { name: "全部", value: "" },
        { name: "羽毛球", value: "羽毛球" },
        { name: "篮球", value: "篮球" },
        { name: "足球", value: "足球" },
        { name: "游泳", value: "游泳" },
      ],
    };
  },
  computed: {
    ...mapGetters(["courses", "loading", "error", "isAuthenticated"]),
    filteredCourses() {
      return this.courses;
    },
  },
  created() {
    // 从路由参数获取类型过滤条件
    if (this.$route.query.type) {
      this.currentTab = this.$route.query.type;
    }

    this.fetchCourses();
  },
  methods: {
    async fetchCourses() {
      try {
        await this.$store.dispatch("getCourses", {
          type: this.currentTab,
          keyword: this.keyword,
        });
      } catch (error) {
        console.error("获取课程失败:", error);
      }
    },
    goBack() {
      this.$router.go(-1);
    },
    selectTab(tabValue) {
      this.currentTab = tabValue;
      this.fetchCourses();
    },
    searchCourses() {
      this.fetchCourses();
    },
    goToCourseDetail(courseId) {
      this.$router.push(`/courses/${courseId}`);
    },
    async enrollCourse(courseId) {
      if (!this.isAuthenticated) {
        this.$router.push({
          path: "/login",
          query: { redirect: `/courses/${courseId}` },
        });
        return;
      }

      try {
        await this.$store.dispatch("enrollCourse", courseId);
        alert("报名成功");
      } catch (error) {
        console.error("报名失败:", error);
      }
    },
    formatDate(dateString) {
      const date = new Date(dateString);
      return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;
    },
  },
};
</script>

<style scoped>
.page-container {
  padding-bottom: 60px;
  background-color: #f7f8fa;
}

.filter-bar {
  padding: 15px;
  background-color: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  position: sticky;
  top: 0;
  z-index: 10;
}

.search-box {
  display: flex;
  align-items: center;
  background-color: #f7f8fa;
  border-radius: 4px;
  padding: 8px 12px;
  margin-bottom: 15px;
}

.search-box .van-icon {
  color: #969799;
  margin-right: 8px;
}

.search-input {
  border: none;
  background-color: transparent;
  flex: 1;
  outline: none;
  font-size: 14px;
}

.filter-tabs {
  display: flex;
  overflow-x: auto;
  margin: 0 -15px;
  padding: 0 15px;
}

.filter-tab {
  flex-shrink: 0;
  padding: 8px 12px;
  font-size: 14px;
  color: #646566;
  position: relative;
  margin-right: 15px;
}

.filter-tab.active {
  color: #1989fa;
  font-weight: 500;
}

.filter-tab.active::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 20px;
  height: 3px;
  background-color: #1989fa;
  border-radius: 3px;
}

.loading-container,
.empty-container {
  padding: 50px 0;
  text-align: center;
}

.loading {
  color: #969799;
}

.course-list {
  padding: 15px;
}

.course-card {
  background-color: #fff;
  border-radius: 8px;
  overflow: hidden;
  margin-bottom: 15px;
  box-shadow: 0 2px 12px rgba(100, 101, 102, 0.08);
}

.course-image {
  position: relative;
}

.course-tag {
  position: absolute;
  top: 10px;
  left: 10px;
  background-color: rgba(0, 0, 0, 0.6);
  color: #fff;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.course-content {
  padding: 15px;
}

.course-title {
  margin: 0 0 10px;
  font-size: 16px;
  font-weight: 500;
}

.course-info {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
  margin-bottom: 15px;
}

.info-item {
  display: flex;
  align-items: center;
  font-size: 14px;
  color: #646566;
}

.info-item .van-icon {
  margin-right: 5px;
  font-size: 16px;
}

.course-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.course-price {
  font-size: 18px;
  font-weight: 500;
  color: #ee0a24;
}
</style>
