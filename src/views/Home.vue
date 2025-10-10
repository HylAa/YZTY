<template>
  <div class="package-page">
    <div class="nav-bar">
      <div class="nav-left" />
      <div class="nav-title">我的课程套餐</div>
      <div class="nav-right">
        <van-icon name="question-o" size="20" @click="goToCustomerService" />
        <span class="service-number" @click="goToCustomerService">客服</span>
      </div>
    </div>

    <div class="content">
      <section class="user-section">
        <div class="user-info">
          <template v-if="userInfo">
            <div v-if="userInfo.headimgurl" class="user-avatar">
              <img :src="userInfo.headimgurl" alt="头像" />
            </div>
            <h2 class="user-id">
              {{ userInfo.nickname || "学员" }}
              <span v-if="currentPhone">{{ formattedPhone }}</span>
            </h2>
          </template>
          <template v-else>
            <h2 class="user-id">学员</h2>
          </template>

          <div class="user-tags">
            <span class="tag card-tag">体育会员</span>
            <span class="tag star-tag">专属学员</span>
          </div>

          <p v-if="needsBindPhone" class="bind-hint">
            已获取微信信息，请先绑定手机号以展示课程
          </p>
        </div>
      </section>

      <section class="courses-section">
        <div v-if="loadingCourses" class="loading-box">
          <van-loading type="spinner" size="30px" color="#1989fa" />
          <span>正在加载课程...</span>
        </div>

        <template v-else>
          <div v-if="courseError" class="error-box">
            <van-icon name="warning-o" size="18" />
            <span>{{ courseError }}</span>
          </div>

          <template v-else-if="hasCourses">
            <div class="summary-card">
              <div class="summary-header">
                <h3>{{ summaryTitle }}</h3>
                <p>手机号：{{ formattedPhone }}</p>
              </div>
              <div class="summary-stats">
                <div class="stat-item">
                  <span class="stat-label">课程数量</span>
                  <span class="stat-value">{{
                    courseSummary.totalCourses
                  }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">课消金额</span>
                  <span class="stat-value">{{
                    formatCurrency(courseSummary.totalConsumedAmount)
                  }}</span>
                </div>
                <div class="stat-item">
                  <span class="stat-label">剩余金额</span>
                  <span class="stat-value">{{
                    formatCurrency(courseSummary.totalRemainingAmount)
                  }}</span>
                </div>
              </div>
            </div>

            <!-- @click="openCourseDetail(course)" -->
            <div
              v-for="(course, index) in courses"
              :key="`${course.course_name || 'course'}-${index}`"
              class="course-card"
            >
              <div class="course-header">
                <div>
                  <h4>{{ course.course_name || "未命名课程" }}</h4>
                  <p v-if="course.class_name" class="course-meta">
                    班级：{{ course.class_name }}
                  </p>
                </div>
                <van-tag v-if="course.course_type" type="primary">
                  {{ course.course_type }}
                </van-tag>
              </div>

              <div class="course-body">
                <div class="course-row">
                  <span>总课时</span>
                  <span>{{ course.purchase_quantity || "-" }}</span>
                </div>
                <div class="course-row">
                  <span>赠送课时</span>
                  <span>{{ course.gifted_quantity || "-" }}</span>
                </div>
                <div class="course-row">
                  <span>已消耗课时</span>
                  <span>{{ course.consumed_quantity || "-" }}</span>
                </div>
                <div class="course-row">
                  <span>剩余课时</span>
                  <span class="highlight">{{
                    course.remaining_quantity || "-"
                  }}</span>
                </div>
                <div class="course-row">
                  <span>退转课时</span>
                  <span>{{ course.refund_transfer_quantity || "-" }}</span>
                </div>
                <div class="course-row">
                  <span>超上课时</span>
                  <span>{{ course.over_attend_quantity || "-" }}</span>
                </div>
              </div>

              <div class="course-footer">
                <div>
                  课消金额：
                  <strong>{{ formatCurrency(course.consumed_amount) }}</strong>
                </div>
                <div>
                  剩余金额：
                  <strong>{{ formatCurrency(course.remaining_amount) }}</strong>
                </div>
              </div>
              <div class="course-extra">
                到期时间：{{ formatDate(course.expire_date) }}
              </div>
            </div>
          </template>

          <van-empty v-else description="暂无课程数据" />
        </template>
      </section>
    </div>

    <div class="footer">
      <div class="service-btn" @click="goToCustomerService">
        <div class="service-icon">客</div>
        <span>在线客服</span>
      </div>
      <!-- <div class="action-tabs">
        <div class="action-tab">课程专区</div>
      </div> -->
    </div>

    <BindPhoneDialog
      v-model:show="showBindPhone"
      :openid="userInfo && userInfo.openid ? userInfo.openid : ''"
      @bind-success="onBindPhone"
    />

    <van-dialog
      :show="showAuthDialog"
      title="授权提示"
      confirm-button-text="确认授权"
      @update:show="showAuthDialog = $event"
      @confirm="handleAuthConfirm"
    >
      <p class="auth-tip">
        请授权微信获取您的头像和昵称信息，以便提供更好的服务体验
      </p>
    </van-dialog>
  </div>
</template>

<script>
import { ref, reactive, computed, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import { showToast, showLoadingToast, closeToast } from "vant";
import BindPhoneDialog from "../components/BindPhoneDialog.vue";
import wxUtils from "../utils/wxUtils";
import api from "../api";

export default {
  name: "Home",
  components: { BindPhoneDialog },
  setup() {
    const router = useRouter();
    const userInfo = ref(null);
    const showAuthDialog = ref(false);
    const showBindPhone = ref(false);

    const courses = ref([]);
    const courseSummary = reactive({
      totalCourses: 0,
      totalConsumedAmount: 0,
      totalRemainingAmount: 0,
    });
    const studentNames = ref([]);
    const loadingCourses = ref(false);
    const courseError = ref("");
    const lastQueriedPhone = ref("");

    const currentPhone = computed(() => {
      const info = userInfo.value;
      if (!info) return "";
      return info.phoneNumber || info.phone || "";
    });

    const formattedPhone = computed(() => {
      if (!currentPhone.value) return "未绑定手机号";
      return formatPhone(currentPhone.value);
    });

    const summaryTitle = computed(() => {
      if (studentNames.value.length > 0) {
        return studentNames.value.join("、");
      }
      if (userInfo.value && userInfo.value.nickname) {
        return userInfo.value.nickname;
      }
      return "我的课程";
    });

    const hasCourses = computed(() => courses.value.length > 0);
    const needsBindPhone = computed(
      () => Boolean(userInfo.value) && !currentPhone.value
    );

    const resetCourses = () => {
      courses.value = [];
      courseSummary.totalCourses = 0;
      courseSummary.totalConsumedAmount = 0;
      courseSummary.totalRemainingAmount = 0;
      studentNames.value = [];
    };

    const fetchCourses = async (phone) => {
      if (!phone) return;
      if (loadingCourses.value) return;

      loadingCourses.value = true;
      courseError.value = "";

      try {
        const response = await api.student.getCoursesByPhone(phone);
        if (response?.code === 0 && response.data) {
          const data = response.data;
          courses.value = Array.isArray(data.records) ? data.records : [];
          courseSummary.totalCourses = Number(
            data.total_courses ?? courses.value.length ?? 0
          );
          courseSummary.totalConsumedAmount = toNumber(
            data.total_consumed_amount
          );
          courseSummary.totalRemainingAmount = toNumber(
            data.total_remaining_amount
          );
          studentNames.value = Array.isArray(data.student_names)
            ? data.student_names
            : [];
          lastQueriedPhone.value = phone;
        } else {
          resetCourses();
          courseError.value =
            (response && response.message) || "查询课程失败，请稍后再试";
          lastQueriedPhone.value = "";
        }
      } catch (error) {
        resetCourses();
        courseError.value =
          error?.response?.data?.message ||
          error?.message ||
          "查询课程失败，请稍后再试";
        lastQueriedPhone.value = "";
      } finally {
        loadingCourses.value = false;
      }
    };

    const maybeFetchCourses = () => {
      const phone = currentPhone.value;
      if (!phone) {
        resetCourses();
        return;
      }
      if (lastQueriedPhone.value === phone && courses.value.length > 0) {
        return;
      }
      fetchCourses(phone);
    };

    const formatPhone = (phone) => {
      if (!phone || phone.length !== 11) return phone || "";
      return phone.replace(/(\d{3})(\d{4})(\d{4})/, "$1****$3");
    };

    const toNumber = (value) => {
      if (value === null || value === undefined || value === "") {
        return 0;
      }
      const numeric = Number(value);
      return Number.isFinite(numeric) ? numeric : 0;
    };

    const formatCurrency = (amount) => {
      if (amount === null || amount === undefined || amount === "") {
        return "-";
      }
      const numeric = Number(amount);
      if (Number.isNaN(numeric)) {
        return "-";
      }
      return `¥${numeric.toFixed(2)}`;
    };

    const formatDate = (date) => {
      if (!date) return "-";
      if (date instanceof Date) {
        return date.toISOString().split("T")[0];
      }
      if (typeof date === "string") {
        return date.split("T")[0];
      }
      return String(date);
    };

    const persistUserInfo = (info, { triggerBindPopup = false } = {}) => {
      if (!info) return;
      const normalizedPhone =
        info.phoneNumber || info.phone || info.phone_number || "";
      const normalized = {
        ...info,
        phoneNumber: normalizedPhone,
        phone: normalizedPhone || null,
      };
      userInfo.value = normalized;
      localStorage.setItem("userInfo", JSON.stringify(normalized));
      if (triggerBindPopup && !normalizedPhone) {
        showBindPhone.value = true;
      }
      maybeFetchCourses();
    };

    const checkWechatAuth = async () => {
      try {
        const code = wxUtils.getAuthCodeFromUrl();
        if (code) {
          showLoadingToast({ message: "获取用户信息...", forbidClick: true });
          try {
            const userInfoData = await wxUtils.getUserInfoByCode(code);
            persistUserInfo(userInfoData, { triggerBindPopup: true });
          } finally {
            closeToast();
          }
        } else {
          const storedUserInfo = localStorage.getItem("userInfo");
          if (storedUserInfo) {
            try {
              const parsed = JSON.parse(storedUserInfo);
              persistUserInfo(parsed, { triggerBindPopup: false });
            } catch (e) {
              console.warn("本地用户信息解析失败", e);
              localStorage.removeItem("userInfo");
              showAuthDialog.value = true;
            }
          } else {
            showAuthDialog.value = true;
          }
        }
      } catch (error) {
        console.error("微信授权失败", error);
        showToast("获取用户信息失败");
        closeToast();
      }
    };

    const handleAuthConfirm = () => {
      wxUtils.oauthLogin();
    };

    const goToCustomerService = () => {
      window.location.href =
        "https://work.weixin.qq.com/kfid/kfc8cb013cc4c466389";
    };

    const initWxConfig = async () => {
      try {
        const url = window.location.href.split("#")[0];
        const config = await wxUtils.getJssdkConfig(url);
        if (config && config.skip) {
          return;
        }
        await wxUtils.initJssdkConfig(config);
      } catch (error) {
        console.error("初始化微信JS-SDK失败", error);
      }
    };

    const openCourseDetail = (course) => {
      if (!currentPhone.value) {
        showToast("请先绑定手机号");
        return;
      }
      router.push({
        path: "/courses",
        query: {
          phone: currentPhone.value,
          course: course && course.course_name ? course.course_name : "",
        },
      });
    };

    const onBindPhone = ({ phoneNumber, user }) => {
      const merged = {
        ...(userInfo.value || {}),
        ...(user || {}),
        phoneNumber,
        phone: phoneNumber,
      };
      persistUserInfo(merged, { triggerBindPopup: false });
      showBindPhone.value = false;
      showToast("手机号绑定成功");
    };

    watch(currentPhone, (phone) => {
      if (!phone) {
        resetCourses();
        return;
      }
      maybeFetchCourses();
    });

    onMounted(() => {
      initWxConfig();
      checkWechatAuth();
    });

    return {
      userInfo,
      showAuthDialog,
      showBindPhone,
      courses,
      courseSummary,
      courseError,
      loadingCourses,
      hasCourses,
      summaryTitle,
      formattedPhone,
      currentPhone,
      needsBindPhone,
      formatCurrency,
      formatDate,
      goToCustomerService,
      handleAuthConfirm,
      openCourseDetail,
      onBindPhone,
    };
  },
};
</script>

<style scoped>
.package-page {
  min-height: 100vh;
  background-color: #f0f7ff;
  position: relative;
  padding-bottom: 60px;
}

.nav-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 15px;
}

.nav-title {
  font-size: 18px;
  font-weight: bold;
}

.nav-right {
  display: flex;
  align-items: center;
  gap: 5px;
}

.service-number {
  margin: 0 5px;
}

.content {
  padding: 0 15px 80px;
}

.user-section {
  margin-top: 10px;
}

.user-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.user-avatar {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  overflow: hidden;
  margin-bottom: 10px;
}

.user-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.user-id {
  font-size: 20px;
  font-weight: 500;
  margin-bottom: 10px;
}

.user-tags {
  display: flex;
  gap: 10px;
}

.tag {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  border: 1px solid;
}

.card-tag {
  border-color: #7e7e7e;
  color: #7e7e7e;
}

.star-tag {
  border-color: #3b82f6;
  color: #3b82f6;
}

.bind-hint {
  margin-top: 10px;
  color: #f97316;
  font-size: 12px;
}

.courses-section {
  margin-top: 20px;
}

.loading-box {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 20px 0;
  color: #1989fa;
}

.error-box {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px;
  border-radius: 10px;
  background: #fdecea;
  color: #d93025;
  font-size: 13px;
  margin-bottom: 12px;
}

.summary-card {
  background: linear-gradient(135deg, #dbeafe, #eff6ff);
  border-radius: 12px;
  padding: 18px;
  box-shadow: 0 10px 25px rgba(59, 130, 246, 0.1);
}

.summary-header h3 {
  margin: 0 0 6px;
  font-size: 18px;
}

.summary-header p {
  margin: 0;
  color: #4b5563;
  font-size: 13px;
}

.summary-stats {
  display: flex;
  justify-content: space-between;
  margin-top: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 12px;
  color: #4b5563;
}

.stat-value {
  margin-top: 6px;
  font-size: 20px;
  font-weight: 600;
  color: #1d4ed8;
}

.course-card {
  margin-top: 16px;
  padding: 16px;
  border-radius: 12px;
  background-color: #fff;
  box-shadow: 0 8px 20px rgba(15, 23, 42, 0.08);
  transition: transform 0.15s ease;
}

.course-card:active {
  transform: scale(0.99);
}

.course-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.course-header h4 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
}

.course-meta {
  margin-top: 4px;
  color: #6b7280;
  font-size: 13px;
}

.course-body {
  margin-top: 12px;
}

.course-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  color: #4b5563;
  font-size: 14px;
}

.course-row .highlight {
  color: #1989fa;
  font-weight: 600;
}

.course-footer {
  display: flex;
  justify-content: space-between;
  margin-top: 12px;
  font-size: 13px;
  color: #1f2937;
}

.course-footer strong {
  color: #111827;
}

.course-extra {
  margin-top: 10px;
  font-size: 12px;
  color: #6b7280;
}

.footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  background: #fff;
  border-top: 1px solid #eee;
  height: 60px;
}

.service-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  border-right: 1px solid #eee;
  font-size: 12px;
  color: #666;
}

.service-icon {
  width: 24px;
  height: 24px;
  background-color: #eee;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 3px;
}

.action-tabs {
  display: flex;
  flex: 1;
}

.action-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #33b4ff;
  color: #fff;
  font-weight: 500;
}

.auth-tip {
  padding: 20px;
  text-align: center;
  color: #666;
}
</style>
