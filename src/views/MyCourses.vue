<template>
  <div class="my-courses-page">
    <van-nav-bar
      title="我的课程"
      left-arrow
      fixed
      @click-left="$router.back()"
      :border="false"
      class="custom-nav"
    >
      <template #right>
        <van-icon name="service-o" size="20" />
      </template>
    </van-nav-bar>

    <div class="content">
      <div class="search-card">
        <van-field
          v-model="phone"
          label="手机号"
          type="tel"
          maxlength="20"
          clearable
          placeholder="请输入手机号查询课程"
          :disabled="loading"
        />
        <van-button
          type="primary"
          block
          round
          class="search-btn"
          :loading="loading"
          @click="handleSearch"
        >
          查询课程
        </van-button>
        <p class="search-hint">支持学员手机号或备用手机号查询</p>
      </div>

      <div v-if="errorMessage" class="error-box">
        <van-icon name="warning-o" size="18" />
        <span>{{ errorMessage }}</span>
      </div>

      <div v-if="loading" class="loading-box">
        <van-loading type="spinner" size="32px" color="#1989fa" />
        <span>查询中，请稍候...</span>
      </div>

      <template v-if="!loading && hasData">
        <div class="summary-card">
          <div class="summary-header">
            <h2>{{ studentTitle }}</h2>
            <p>手机号：{{ maskedPhone }}</p>
          </div>
          <div class="summary-stats">
            <div class="stat-item">
              <span class="label">课程数量</span>
              <span class="value">{{ summary.total_courses }}</span>
            </div>
            <div class="stat-item">
              <span class="label">已消耗金额</span>
              <span class="value">{{ formatCurrency(summary.total_consumed_amount) }}</span>
            </div>
            <div class="stat-item">
              <span class="label">剩余金额</span>
              <span class="value">{{ formatCurrency(summary.total_remaining_amount) }}</span>
            </div>
          </div>
        </div>

        <div
          v-for="(record, index) in records"
          :key="`${record.course_name || 'course'}-${index}`"
          class="course-card"
        >
          <div class="course-header">
            <div>
              <h3>{{ record.course_name || "未命名课程" }}</h3>
              <p v-if="record.class_name" class="class-name">
                班级：{{ record.class_name }}
              </p>
            </div>
            <van-tag v-if="record.course_type" type="primary">
              {{ record.course_type }}
            </van-tag>
          </div>

          <div class="course-body">
            <div class="course-row">
              <span>总课时</span>
              <span>{{ record.purchase_quantity || "-" }}</span>
            </div>
            <div class="course-row">
              <span>赠送课时</span>
              <span>{{ record.gifted_quantity || "-" }}</span>
            </div>
            <div class="course-row">
              <span>已消耗课时</span>
              <span>{{ record.consumed_quantity || "-" }}</span>
            </div>
            <div class="course-row">
              <span>剩余课时</span>
              <span class="highlight">{{ record.remaining_quantity || "-" }}</span>
            </div>
            <div class="course-row">
              <span>退转课时</span>
              <span>{{ record.refund_transfer_quantity || "-" }}</span>
            </div>
            <div class="course-row">
              <span>超上课时</span>
              <span>{{ record.over_attend_quantity || "-" }}</span>
            </div>
          </div>

          <div class="course-footer">
            <div>
              <span>课消金额：</span>
              <strong>{{ formatCurrency(record.consumed_amount) }}</strong>
            </div>
            <div>
              <span>剩余金额：</span>
              <strong>{{ formatCurrency(record.remaining_amount) }}</strong>
            </div>
          </div>

          <div class="course-extra">
            <span>到期时间：{{ formatDate(record.expire_date) }}</span>
          </div>
        </div>
      </template>

      <van-empty
        v-else-if="!loading && searched"
        description="未查询到相关课程"
        class="empty-box"
      />

      <div class="placeholder" />
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, reactive, ref } from "vue";
import { useRoute } from "vue-router";
import api from "../api";

const phone = ref("");
const loading = ref(false);
const errorMessage = ref("");
const searched = ref(false);
const records = ref([]);
const studentNames = ref([]);
const summary = reactive({
  total_courses: 0,
  total_consumed_amount: 0,
  total_remaining_amount: 0,
});

const route = useRoute();

const hasData = computed(() => records.value.length > 0);
const maskedPhone = computed(() => maskPhone(phone.value));
const studentTitle = computed(() => {
  if (studentNames.value.length === 0) {
    return "未识别学员";
  }
  return studentNames.value.join("、");
});

onMounted(() => {
  const queryPhone = route.query?.phone;
  if (typeof queryPhone === "string" && queryPhone.trim()) {
    phone.value = queryPhone.trim();
    handleSearch();
  }
});

function resetSummary() {
  summary.total_courses = 0;
  summary.total_consumed_amount = 0;
  summary.total_remaining_amount = 0;
}

function maskPhone(value) {
  if (!value) {
    return "--";
  }
  const digits = value.replace(/\\s+/g, "");
  if (digits.length < 7) {
    return digits;
  }
  return `${digits.slice(0, 3)}****${digits.slice(-4)}`;
}

function formatCurrency(amount) {
  if (amount === null || amount === undefined) {
    return "-";
  }
  const numeric = Number(amount);
  if (Number.isNaN(numeric)) {
    return "-";
  }
  return `¥${numeric.toFixed(2)}`;
}

function formatDate(date) {
  if (!date) {
    return "-";
  }
  return String(date);
}

function normalizePhone(value) {
  return value.replace(/\\s+/g, "");
}

async function handleSearch() {
  const normalized = normalizePhone(phone.value);
  phone.value = normalized;

  if (!normalized) {
    errorMessage.value = "请输入手机号";
    records.value = [];
    studentNames.value = [];
    resetSummary();
    searched.value = true;
    return;
  }

  loading.value = true;
  errorMessage.value = "";

  try {
    const res = await api.student.getCoursesByPhone(normalized);
    if (res?.code === 0) {
      const data = res.data || {};
      records.value = data.records || [];
      studentNames.value = data.student_names || [];
      summary.total_courses = data.total_courses || 0;
      summary.total_consumed_amount = data.total_consumed_amount || 0;
      summary.total_remaining_amount = data.total_remaining_amount || 0;
    } else {
      errorMessage.value = res?.message || "查询失败";
      records.value = [];
      studentNames.value = [];
      resetSummary();
    }
  } catch (error) {
    const msg =
      error?.response?.data?.message ||
      error?.message ||
      "查询失败，请稍后再试";
    errorMessage.value = msg;
    records.value = [];
    studentNames.value = [];
    resetSummary();
  } finally {
    loading.value = false;
    searched.value = true;
  }
}
</script>

<style scoped>
.my-courses-page {
  background-color: #f5f7fa;
  min-height: 100vh;
  padding-top: 54px;
}

.custom-nav {
  background-color: #f0f7ff !important;
}

:deep(.van-nav-bar__title) {
  color: #000;
  font-weight: 600;
  font-size: 18px;
}

.content {
  padding: 70px 16px 32px;
}

.search-card {
  background-color: #fff;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 10px 25px rgba(59, 130, 246, 0.1);
}

.search-btn {
  margin-top: 12px;
}

.search-hint {
  margin: 8px 0 0;
  color: #666;
  font-size: 12px;
}

.error-box {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 16px;
  padding: 12px;
  border-radius: 10px;
  background: #fdecea;
  color: #d93025;
  font-size: 13px;
}

.loading-box {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 20px;
  color: #1989fa;
}

.summary-card {
  margin-top: 20px;
  padding: 18px;
  border-radius: 12px;
  background: linear-gradient(135deg, #dbeafe, #eff6ff);
}

.summary-header h2 {
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
  align-items: flex-start;
}

.stat-item .label {
  color: #4b5563;
  font-size: 12px;
}

.stat-item .value {
  margin-top: 6px;
  font-size: 20px;
  font-weight: 600;
  color: #1d4ed8;
}

.course-card {
  margin-top: 18px;
  padding: 18px;
  border-radius: 12px;
  background-color: #fff;
  box-shadow: 0 8px 20px rgba(15, 23, 42, 0.08);
}

.course-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.course-header h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: #1f2937;
}

.class-name {
  margin-top: 4px;
  color: #6b7280;
  font-size: 13px;
}

.course-body {
  margin-top: 16px;
}

.course-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
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
  margin-top: 16px;
  font-size: 14px;
  color: #111827;
}

.course-footer strong {
  color: #1f2937;
}

.course-extra {
  margin-top: 14px;
  font-size: 12px;
  color: #6b7280;
}

.empty-box {
  margin-top: 40px;
}

.placeholder {
  height: 60px;
}
</style>
