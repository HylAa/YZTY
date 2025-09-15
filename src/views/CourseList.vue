<template>
  <div class="course-list-page">
    <van-nav-bar
      title="课程列表"
      left-text="返回"
      left-arrow
      @click-left="goBack"
      fixed
    />

    <div class="page-content">
      <!-- Course Type Tabs -->
      <van-tabs v-model="activeType" sticky>
        <van-tab
          v-for="category in categories"
          :key="category.type"
          :title="category.name"
          :name="category.type"
        ></van-tab>
      </van-tabs>

      <!-- Course List -->
      <van-pull-refresh v-model="refreshing" @refresh="onRefresh">
        <div class="course-container">
          <div v-if="loading" class="loading-container">
            <van-skeleton title :row="3" />
            <van-skeleton title :row="3" />
            <van-skeleton title :row="3" />
          </div>

          <div v-else-if="filteredCourses.length === 0" class="empty-state">
            暂无{{ activeTypeName }}课程
          </div>

          <div v-else class="course-list">
            <van-card
              v-for="course in filteredCourses"
              :key="course._id"
              :price="course.price.toFixed(2)"
              :title="course.name"
              :thumb="course.image"
              @click="navigateToCourseDetail(course._id)"
            >
              <template #tags>
                <van-tag plain type="primary">{{ course.type }}</van-tag>
                <van-tag plain type="success" v-if="course.enrollmentCount > 10"
                  >热门</van-tag
                >
              </template>
              <template #desc>
                <div class="course-desc">
                  <div>{{ course.location.name }}</div>
                  <div>
                    {{ course.totalSessions }}节课 |
                    {{ course.duration }}分钟/节
                  </div>
                </div>
              </template>
              <template #footer>
                <div class="course-footer">
                  <div class="course-rating">
                    <van-icon name="star" color="#ffd21e" />
                    {{ course.rating || "暂无评分" }}
                  </div>
                  <div class="course-enrollment">
                    {{ course.enrollmentCount }}人已报名
                  </div>
                </div>
              </template>
            </van-card>
          </div>
        </div>
      </van-pull-refresh>
    </div>
  </div>
</template>

<script>
import { computed, onMounted, ref, watch } from "vue";
import { useStore } from "vuex";
import { useRouter, useRoute } from "vue-router";

export default {
  name: "CourseList",
  setup() {
    const store = useStore();
    const router = useRouter();
    const route = useRoute();

    // Course categories
    const categories = ref([
      { name: "全部", type: "" },
      { name: "健身", type: "健身" },
      { name: "游泳", type: "游泳" },
      { name: "瑜伽", type: "瑜伽" },
      { name: "篮球", type: "篮球" },
      { name: "足球", type: "足球" },
      { name: "羽毛球", type: "羽毛球" },
      { name: "乒乓球", type: "乒乓球" },
      { name: "其他", type: "其他" },
    ]);

    // Query params
    const activeType = ref(route.query.type || "");
    const phoneQuery = ref(route.query.phone || "");

    // Refreshing state
    const refreshing = ref(false);

    // Computed properties
    const loading = computed(() => store.getters["loading"]);
    const courses = computed(() => store.getters["courses/allCourses"]);

    // Filter courses by active type
    const filteredCourses = computed(() => {
      if (!activeType.value) {
        return courses.value;
      }
      return courses.value.filter((course) => course.type === activeType.value);
    });

    // Get active type name
    const activeTypeName = computed(() => {
      const category = categories.value.find(
        (cat) => cat.type === activeType.value
      );
      return category ? category.name : "全部";
    });

    // Load courses on mount
    onMounted(() => {
      fetchCourses();
      // 若带手机号查询参数，可在此触发后端按手机号筛选（如有接口）
      if (phoneQuery.value) {
        // 这里保留占位：根据实际后端API调整
        // 示例：store.dispatch('courses/fetchCourses', { phone: phoneQuery.value })
        // 临时提示
        console.log("按手机号查询课程：", phoneQuery.value);
      }
    });

    // Watch for active type changes
    watch(activeType, (newType) => {
      // Update route query
      const nextQuery = { ...route.query };
      if (newType) nextQuery.type = newType; else delete nextQuery.type;
      router.replace({ path: "/courses", query: nextQuery });
    
      // Fetch courses if needed
      fetchCourses();
    });

    // Fetch courses
    const fetchCourses = async () => {
      try {
        if (activeType.value) {
          await store.dispatch("courses/fetchCoursesByType", activeType.value);
        } else {
          await store.dispatch("courses/fetchCourses");
        }
      } catch (error) {
        console.error("Failed to fetch courses:", error);
      }
    };

    // Refresh handler
    const onRefresh = async () => {
      try {
        await fetchCourses();
      } finally {
        refreshing.value = false;
      }
    };

    // Navigation
    const navigateToCourseDetail = (id) => {
      router.push(`/courses/${id}`);
    };

    const goBack = () => {
      router.back();
    };

    return {
      categories,
      activeType,
      refreshing,
      loading,
      filteredCourses,
      activeTypeName,
      onRefresh,
      navigateToCourseDetail,
      goBack,
    };
  },
};
</script>

<style scoped>
.course-list-page {
  padding-top: 46px;
}

.page-content {
  padding-bottom: 50px;
}

.course-container {
  padding: 16px;
}

.course-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.course-desc {
  margin-top: 4px;
  font-size: 12px;
  color: #646566;
  line-height: 1.5;
}

.course-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #969799;
}

.course-rating {
  display: flex;
  align-items: center;
  gap: 4px;
}

.loading-container {
  padding: 16px 0;
}

.empty-state {
  padding: 32px 0;
  text-align: center;
  color: #969799;
}
</style>
