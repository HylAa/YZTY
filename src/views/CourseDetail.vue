<template>
  <div class="course-detail">
    <van-nav-bar
      title="课程详情"
      left-arrow
      fixed
      @click-left="$router.back()"
    />

    <div class="course-content" v-if="course">
      <div class="course-header">
        <van-image
          class="course-image"
          :src="course.image"
          fit="cover"
          width="100%"
          height="200"
        />
        <div class="course-info">
          <h1 class="course-name">{{ course.name }}</h1>
          <div class="course-price">
            <span class="price">¥{{ course.price }}</span>
            <span class="original-price" v-if="course.originalPrice"
              >¥{{ course.originalPrice }}</span
            >
          </div>
          <div class="course-tags">
            <van-tag plain type="primary" class="course-tag">{{
              course.type
            }}</van-tag>
            <van-tag
              plain
              type="success"
              v-if="course.remainingSessions"
              class="course-tag"
            >
              剩余{{ course.remainingSessions }}节课
            </van-tag>
          </div>
          <div class="course-stats">
            <div class="stat-item">
              <span class="stat-value">{{ course.totalStudents }}</span>
              <span class="stat-label">已报名</span>
            </div>
            <div class="stat-item">
              <span class="stat-value">{{ course.totalSessions }}</span>
              <span class="stat-label">总课时</span>
            </div>
            <div class="stat-item">
              <span class="stat-value">{{ course.rating }}</span>
              <span class="stat-label">评分</span>
            </div>
          </div>
        </div>
      </div>

      <van-tabs sticky>
        <van-tab name="info" title="课程信息">
          <div class="tab-content">
            <div class="info-section">
              <div class="section-title">课程简介</div>
              <div class="section-content">{{ course.description }}</div>
            </div>

            <div class="info-section">
              <div class="section-title">上课地点</div>
              <div class="section-content">{{ course.location.name }}</div>
              <div class="section-content">{{ course.location.address }}</div>
            </div>

            <div class="info-section">
              <div class="section-title">课程安排</div>
              <div
                class="section-content"
                v-if="course.schedule && course.schedule.length > 0"
              >
                <div
                  v-for="(schedule, index) in course.schedule"
                  :key="index"
                  class="schedule-item"
                >
                  <div class="day">{{ schedule.dayOfWeek }}</div>
                  <div class="time">
                    {{ schedule.startTime }} - {{ schedule.endTime }}
                  </div>
                </div>
              </div>
              <div class="section-content" v-else>暂无排课信息</div>
            </div>

            <div class="info-section">
              <div class="section-title">教练信息</div>
              <div class="coach-info">
                <van-image
                  class="coach-avatar"
                  :src="course.coach.avatar"
                  fit="cover"
                  round
                  width="50"
                  height="50"
                />
                <div class="coach-details">
                  <div class="coach-name">{{ course.coach.name }}</div>
                  <div class="coach-title">{{ course.coach.title }}</div>
                </div>
              </div>
              <div class="section-content">{{ course.coach.description }}</div>
            </div>
          </div>
        </van-tab>

        <van-tab name="reviews" title="学员评价">
          <div class="tab-content">
            <div class="review-summary">
              <div class="rating">{{ course.rating }}</div>
              <div class="rating-text">综合评分</div>
            </div>

            <div class="review-list">
              <div
                v-for="(review, index) in course.reviews"
                :key="index"
                class="review-item"
              >
                <div class="review-header">
                  <div class="reviewer-info">
                    <van-image
                      class="reviewer-avatar"
                      :src="review.user.avatar"
                      fit="cover"
                      round
                      width="40"
                      height="40"
                    />
                    <div class="reviewer-name">{{ review.user.name }}</div>
                  </div>
                  <div class="review-rating">
                    <van-rate
                      v-model="review.rating"
                      readonly
                      allow-half
                      size="14"
                      color="#ffd21e"
                    />
                    <div class="review-date">{{ review.date }}</div>
                  </div>
                </div>
                <div class="review-content">{{ review.content }}</div>
              </div>
            </div>
          </div>
        </van-tab>
      </van-tabs>

      <div class="course-action">
        <div class="price-info">
          <div class="current-price">¥{{ course.price }}</div>
          <div class="package-info" v-if="coursePackage">
            <span class="package-name">{{ coursePackage.name }}</span>
            <span class="remaining-sessions"
              >剩余{{ coursePackage.remainingSessions }}节课</span
            >
          </div>
        </div>
        <van-button
          type="primary"
          block
          :disabled="!isAuthenticated"
          @click="handleEnroll"
          >{{ enrollButtonText }}</van-button
        >
      </div>
    </div>

    <div v-else class="loading-container">
      <van-loading size="24px" vertical>加载中...</van-loading>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";

export default {
  name: "CourseDetail",
  setup() {
    const router = useRouter();
    const route = useRoute();

    // 模拟数据
    const isAuthenticated = ref(true);
    const course = ref(null);
    const coursePackage = ref({
      name: "运动套餐（个人版）",
      remainingSessions: 30,
    });

    onMounted(() => {
      // 模拟API调用获取课程数据
      setTimeout(() => {
        course.value = {
          id: route.params.id || "1",
          name: "专业篮球训练课程",
          price: 128,
          originalPrice: 198,
          image: "https://img01.yzcdn.cn/vant/cat.jpeg",
          type: "篮球",
          totalStudents: 356,
          totalSessions: 30,
          remainingSessions: 30,
          rating: 4.9,
          description:
            "本课程由专业篮球教练带领，针对不同水平的学员提供全面的篮球技巧训练，包括运球、投篮、防守等基本技巧，以及战术配合和比赛经验的积累。",
          location: {
            name: "星体育中心",
            address: "北京市朝阳区建国路88号",
          },
          schedule: [
            { dayOfWeek: "周一", startTime: "18:30", endTime: "20:00" },
            { dayOfWeek: "周三", startTime: "18:30", endTime: "20:00" },
            { dayOfWeek: "周六", startTime: "10:00", endTime: "11:30" },
          ],
          coach: {
            name: "王教练",
            avatar: "https://img01.yzcdn.cn/vant/cat.jpeg",
            title: "国家一级篮球教练",
            description:
              "前国家队队员，有10年以上的教学经验，培养过多名省级优秀运动员。",
          },
          reviews: [
            {
              user: {
                name: "张三",
                avatar: "https://img01.yzcdn.cn/vant/cat.jpeg",
              },
              rating: 5,
              date: "2023-03-15",
              content: "教练很专业，课程安排合理，训练强度适中，收获很大！",
            },
            {
              user: {
                name: "李四",
                avatar: "https://img01.yzcdn.cn/vant/cat.jpeg",
              },
              rating: 4.5,
              date: "2023-03-10",
              content:
                "场地很好，教练很耐心，同学们也很友好，是一次很愉快的体验。",
            },
          ],
        };
      }, 500);
    });

    const enrollButtonText = computed(() => {
      if (!isAuthenticated.value) {
        return "请先登录";
      }

      if (course.value && course.value.remainingSessions > 0) {
        return "继续学习";
      }

      return "立即报名";
    });

    const handleEnroll = () => {
      if (!isAuthenticated.value) {
        router.push("/login");
        return;
      }

      // 处理报名逻辑
      if (course.value && course.value.remainingSessions > 0) {
        // 已报名，进入学习
        console.log("继续学习");
      } else {
        // 未报名，处理报名
        console.log("立即报名");
      }
    };

    return {
      course,
      coursePackage,
      isAuthenticated,
      enrollButtonText,
      handleEnroll,
    };
  },
};
</script>

<style scoped>
.course-detail {
  padding-top: 46px;
  min-height: 100vh;
  background-color: #f7f8fa;
}

.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: calc(100vh - 46px);
}

.course-content {
  padding-bottom: 50px;
}

.course-header {
  background-color: #fff;
  margin-bottom: 10px;
}

.course-info {
  padding: 15px;
}

.course-name {
  font-size: 18px;
  font-weight: bold;
  margin: 0 0 10px 0;
}

.course-price {
  margin-bottom: 10px;
}

.price {
  font-size: 20px;
  font-weight: bold;
  color: #f44;
  margin-right: 8px;
}

.original-price {
  font-size: 14px;
  color: #999;
  text-decoration: line-through;
}

.course-tags {
  margin-bottom: 15px;
}

.course-tag {
  margin-right: 8px;
}

.course-stats {
  display: flex;
  border-top: 1px solid #ebedf0;
  padding-top: 15px;
}

.stat-item {
  flex: 1;
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 16px;
  font-weight: bold;
  color: #323233;
}

.stat-label {
  display: block;
  font-size: 12px;
  color: #969799;
  margin-top: 4px;
}

.tab-content {
  padding: 15px;
  background-color: #fff;
}

.info-section {
  margin-bottom: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: bold;
  margin-bottom: 10px;
  position: relative;
  padding-left: 12px;
}

.section-title::before {
  content: "";
  position: absolute;
  left: 0;
  top: 4px;
  width: 4px;
  height: 16px;
  background-color: #1989fa;
  border-radius: 2px;
}

.section-content {
  font-size: 14px;
  line-height: 1.6;
  color: #646566;
}

.schedule-item {
  display: flex;
  margin-bottom: 8px;
  background-color: #f7f8fa;
  padding: 8px 12px;
  border-radius: 4px;
}

.day {
  font-weight: bold;
  margin-right: 15px;
}

.coach-info {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.coach-details {
  margin-left: 12px;
}

.coach-name {
  font-size: 15px;
  font-weight: bold;
}

.coach-title {
  font-size: 12px;
  color: #969799;
}

.review-summary {
  text-align: center;
  padding: 20px 0;
  border-bottom: 1px solid #ebedf0;
  margin-bottom: 15px;
}

.rating {
  font-size: 36px;
  font-weight: bold;
  color: #ff9800;
}

.rating-text {
  font-size: 14px;
  color: #969799;
}

.review-item {
  margin-bottom: 15px;
  padding-bottom: 15px;
  border-bottom: 1px solid #ebedf0;
}

.review-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.reviewer-info {
  display: flex;
  align-items: center;
}

.reviewer-name {
  margin-left: 8px;
  font-size: 14px;
  font-weight: bold;
}

.review-rating {
  text-align: right;
}

.review-date {
  font-size: 12px;
  color: #969799;
  margin-top: 4px;
}

.review-content {
  font-size: 14px;
  line-height: 1.6;
}

.course-action {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  padding: 10px 15px;
  background-color: #fff;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.05);
}

.price-info {
  flex: 1;
  margin-right: 15px;
}

.current-price {
  font-size: 18px;
  font-weight: bold;
  color: #f44;
}

.package-info {
  font-size: 12px;
  color: #646566;
  margin-top: 2px;
}

.package-name {
  margin-right: 10px;
}

.remaining-sessions {
  color: #1989fa;
}
</style>
