<template>
  <div class="admin-dashboard">
    <van-nav-bar title="管理员仪表盘" left-arrow @click-left="$router.go(-1)" />

    <div class="container">
      <van-loading v-if="loading" size="24px" vertical>加载中...</van-loading>

      <template v-else>
        <h2 class="section-title">系统概览</h2>

        <van-grid :column-num="2" :gutter="10">
          <van-grid-item>
            <van-card title="用户总数">
              <template #num>
                <div class="stat-number">
                  {{ dashboardStats?.totalUsers || 0 }}
                </div>
              </template>
            </van-card>
          </van-grid-item>

          <van-grid-item>
            <van-card title="课程总数">
              <template #num>
                <div class="stat-number">
                  {{ dashboardStats?.totalCourses || 0 }}
                </div>
              </template>
            </van-card>
          </van-grid-item>

          <van-grid-item>
            <van-card title="总报名数">
              <template #num>
                <div class="stat-number">
                  {{ dashboardStats?.totalEnrollments || 0 }}
                </div>
              </template>
            </van-card>
          </van-grid-item>

          <van-grid-item>
            <van-card title="精选课程">
              <template #num>
                <div class="stat-number">
                  {{ dashboardStats?.featuredCourses?.length || 0 }}
                </div>
              </template>
            </van-card>
          </van-grid-item>
        </van-grid>

        <h2 class="section-title">会员等级分布</h2>
        <van-card class="chart-card">
          <div id="memberLevelChart" style="height: 300px; width: 100%"></div>
        </van-card>

        <h2 class="section-title">课程类型分布</h2>
        <van-card class="chart-card">
          <div id="courseTypeChart" style="height: 300px; width: 100%"></div>
        </van-card>

        <h2 class="section-title">最新用户</h2>
        <van-cell-group>
          <van-cell
            v-for="user in dashboardStats?.latestUsers"
            :key="user._id"
            :title="user.name"
            :label="`手机号: ${user.phone} | 等级: ${user.memberLevel}`"
            :value="formatDate(user.createdAt)"
          />
        </van-cell-group>

        <h2 class="section-title">精选课程</h2>
        <van-cell-group>
          <van-cell
            v-for="course in dashboardStats?.featuredCourses"
            :key="course._id"
            :title="course.name"
            :label="`类型: ${course.type} | 报名人数: ${course.enrollmentCount}`"
            is-link
            @click="$router.push(`/courses/${course._id}`)"
          />
        </van-cell-group>
      </template>
    </div>
  </div>
</template>

<script>
import { computed, onMounted, ref } from "vue";
import { useStore } from "vuex";
import * as echarts from "echarts/core";
import { TooltipComponent, LegendComponent } from "echarts/components";
import { PieChart } from "echarts/charts";
import { CanvasRenderer } from "echarts/renderers";

// 注册必须的组件
echarts.use([TooltipComponent, LegendComponent, PieChart, CanvasRenderer]);

export default {
  name: "AdminDashboard",
  setup() {
    const store = useStore();
    const loading = ref(true);

    const dashboardStats = computed(
      () => store.getters["admin/dashboardStats"]
    );

    onMounted(async () => {
      try {
        loading.value = true;
        await store.dispatch("admin/getDashboardStats");
        loading.value = false;

        // 在数据加载完成后初始化图表
        setTimeout(() => {
          initMemberLevelChart();
          initCourseTypeChart();
        }, 100);
      } catch (error) {
        console.error("加载仪表盘数据失败:", error);
        loading.value = false;
      }
    });

    const initMemberLevelChart = () => {
      const chartDom = document.getElementById("memberLevelChart");
      if (!chartDom) return;

      const myChart = echarts.init(chartDom);

      const levels = dashboardStats.value?.usersByLevel || [];
      const data = levels.map((level) => ({
        name: level._id,
        value: level.count,
      }));

      const option = {
        tooltip: {
          trigger: "item",
          formatter: "{a} <br/>{b}: {c} ({d}%)",
        },
        legend: {
          orient: "horizontal",
          bottom: 10,
          data: data.map((item) => item.name),
        },
        series: [
          {
            name: "会员等级",
            type: "pie",
            radius: ["40%", "70%"],
            avoidLabelOverlap: false,
            label: {
              show: false,
              position: "center",
            },
            emphasis: {
              label: {
                show: true,
                fontSize: "18",
                fontWeight: "bold",
              },
            },
            labelLine: {
              show: false,
            },
            data: data,
          },
        ],
      };

      myChart.setOption(option);

      // 窗口大小调整时重绘图表
      window.addEventListener("resize", () => {
        myChart.resize();
      });
    };

    const initCourseTypeChart = () => {
      const chartDom = document.getElementById("courseTypeChart");
      if (!chartDom) return;

      const myChart = echarts.init(chartDom);

      const types = dashboardStats.value?.coursesByType || [];
      const data = types.map((type) => ({
        name: type._id,
        value: type.count,
      }));

      const option = {
        tooltip: {
          trigger: "item",
          formatter: "{a} <br/>{b}: {c} ({d}%)",
        },
        legend: {
          orient: "horizontal",
          bottom: 10,
          data: data.map((item) => item.name),
        },
        series: [
          {
            name: "课程类型",
            type: "pie",
            radius: ["40%", "70%"],
            avoidLabelOverlap: false,
            label: {
              show: false,
              position: "center",
            },
            emphasis: {
              label: {
                show: true,
                fontSize: "18",
                fontWeight: "bold",
              },
            },
            labelLine: {
              show: false,
            },
            data: data,
          },
        ],
      };

      myChart.setOption(option);

      // 窗口大小调整时重绘图表
      window.addEventListener("resize", () => {
        myChart.resize();
      });
    };

    // 日期格式化
    const formatDate = (dateString) => {
      const date = new Date(dateString);
      return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(
        2,
        "0"
      )}-${String(date.getDate()).padStart(2, "0")}`;
    };

    return {
      dashboardStats,
      loading,
      formatDate,
    };
  },
};
</script>

<style scoped>
.admin-dashboard {
  padding-bottom: 50px;
}

.container {
  padding: 16px;
}

.section-title {
  margin: 20px 0 10px;
  font-size: 18px;
  font-weight: bold;
  color: #323233;
}

.stat-number {
  font-size: 24px;
  font-weight: bold;
  color: #1989fa;
}

.chart-card {
  background-color: #fff;
  padding: 10px;
  margin-bottom: 20px;
  border-radius: 8px;
}
</style>
