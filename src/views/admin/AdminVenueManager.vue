<template>
  <div class="admin-venue-page">
    <header class="page-header">
      <div>
        <h1>场馆占用管理</h1>
        <p class="subtitle">配置当天各场地及时段的占用状态</p>
      </div>
      <div class="header-actions">
        <button class="ghost-btn" @click="handleLogout">退出登录</button>
      </div>
    </header>

    <section class="control-bar">
      <div class="date-selector">
        <button class="date-btn" @click="changeDate('prev')">← 前一天</button>
        <div class="current-date">
          {{ formattedCurrentDate }}
        </div>
        <button class="date-btn" @click="changeDate('next')">后一天 →</button>
      </div>
      <div class="control-actions">
        <button class="ghost-btn" :disabled="loading" @click="fetchOverview">
          刷新
        </button>
        <button
          class="primary-btn"
          :disabled="!hasChanges || saving"
          @click="saveChanges"
        >
          {{ saving ? "保存中..." : "保存变更" }}
        </button>
      </div>
    </section>

    <section class="feedback-area">
      <div v-if="loading" class="info info--loading">正在加载数据...</div>
      <div v-else-if="error" class="info info--error">
        {{ error }}
      </div>
      <div v-else-if="success" class="info info--success">
        {{ success }}
      </div>
    </section>

    <section v-if="!loading && !error" class="content">
      <div v-if="!categories.length" class="info info--muted">暂无场地数据</div>

      <div
        v-for="category in categories"
        :key="category.type"
        class="category-block"
      >
        <div class="category-header">
          <h2>{{ category.displayTitle }}</h2>
          <span class="meta">共 {{ category.courts.length }} 片</span>
        </div>

        <div class="courts-grid">
          <div
            v-for="court in category.courts"
            :key="court.id"
            class="court-item"
          >
            <div class="court-top">
              <div class="court-number">场地 {{ court.number }}</div>
              <button
                class="toggle-btn"
                :class="court.available ? 'available' : 'booked'"
                @click="toggleCourt(category.type, court)"
              >
                {{ court.available ? "可用" : "占用" }}
              </button>
            </div>
            <div class="note-row">
              <input
                v-model="court.note"
                type="text"
                placeholder="备注（可选）"
                @input="markCourtChanged(court)"
              />
            </div>
            <div v-if="court.updatedAt" class="meta-row">
              最近更新：{{ court.updatedAt }}
            </div>
          </div>
        </div>
      </div>

      <!-- <div v-if="timeSlots.length" class="time-block">
        <div class="category-header">
          <h2>时段占用管理</h2>
          <span class="meta">共 {{ timeSlots.length }} 个时段</span>
        </div>
        <div class="slots-grid">
          <div v-for="slot in timeSlots" :key="slot.key" class="slot-item">
            <div class="slot-top">
              <div class="slot-label">{{ slot.label }}</div>
              <button
                class="toggle-btn"
                :class="slot.available ? 'available' : 'booked'"
                @click="toggleSlot(slot)"
              >
                {{ slot.available ? "可用" : "占用" }}
              </button>
            </div>
            <div class="note-row">
              <input
                v-model="slot.note"
                @input="markSlotChanged(slot)"
                type="text"
                placeholder="备注（可选）"
              />
            </div>
            <div v-if="slot.updatedAt" class="meta-row">
              最近更新：{{ slot.updatedAt }}
            </div>
          </div>
        </div>
      </div> -->
    </section>
  </div>
</template>

<script>
import { ref, computed, onMounted } from "vue";
import api from "../../api";

const WEEK_MAP = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

const COURT_META = {
  badminton: {
    label: "羽毛球场地",
  },
  basketball: {
    label: "篮球场地",
  },
  football: {
    label: "足球场地",
  },
};

function formatDisplayDate(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const week = WEEK_MAP[date.getDay()];
  return `${year}年${month}月${day}日 ${week}`;
}

function formatDateParam(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function adjustDate(date, offset) {
  const next = new Date(date);
  next.setDate(next.getDate() + offset);
  return next;
}

export default {
  name: "AdminVenueManager",
  setup() {
    const currentDate = ref(new Date());
    const categories = ref([]);
    const timeSlots = ref([]);
    const loading = ref(false);
    const saving = ref(false);
    const error = ref("");
    const success = ref("");

    const formattedCurrentDate = computed(() =>
      formatDisplayDate(currentDate.value)
    );
    const currentDateParam = computed(() => formatDateParam(currentDate.value));

    const adaptCategories = (list = []) =>
      list.map((category) => {
        const type = category.type;
        const meta = COURT_META[type] || { label: category.title };
        return {
          type,
          displayTitle: `${meta.label ?? category.title} (${
            category.total ?? 0
          }片)`,
          courts: (category.courts || []).map((court) => ({
            id: `${type}-${court.number}`,
            number: court.number,
            available: court.isAvailable,
            note: court.note ?? "",
            updatedAt: court.updated_at ?? "",
            changed: false,
          })),
        };
      });

    const adaptTimeSlots = (list = []) =>
      list.map((slot) => ({
        key: slot.slotKey,
        label: `${slot.startTime}-${slot.endTime}`,
        available: slot.isAvailable,
        note: slot.note ?? "",
        updatedAt: slot.updated_at ?? "",
        changed: false,
      }));

    const fetchOverview = async () => {
      loading.value = true;
      error.value = "";
      success.value = "";
      try {
        const response = await api.venue.getOverview(currentDateParam.value);
        if (!response || response.code !== 0) {
          throw new Error(response?.message || "获取场馆数据失败");
        }
        const data = response.data || {};
        categories.value = adaptCategories(data.categories);
        timeSlots.value = adaptTimeSlots(data.timeSlots);
      } catch (err) {
        console.error("获取场馆数据失败", err);
        error.value = err?.message || "获取场馆数据失败";
        categories.value = [];
        timeSlots.value = [];
      } finally {
        loading.value = false;
      }
    };

    const changeDate = async (direction) => {
      const offset = direction === "prev" ? -1 : 1;
      currentDate.value = adjustDate(currentDate.value, offset);
      await fetchOverview();
    };

    const toggleCourt = (type, court) => {
      court.available = !court.available;
      court.changed = true;
      success.value = "";
    };

    const markCourtChanged = (court) => {
      court.changed = true;
      success.value = "";
    };

    const toggleSlot = (slot) => {
      slot.available = !slot.available;
      slot.changed = true;
      success.value = "";
    };

    const markSlotChanged = (slot) => {
      slot.changed = true;
      success.value = "";
    };

    const hasChanges = computed(() => {
      const courtChanged = categories.value.some((category) =>
        category.courts.some((court) => court.changed)
      );
      const slotChanged = timeSlots.value.some((slot) => slot.changed);
      return courtChanged || slotChanged;
    });

    const saveChanges = async () => {
      if (!hasChanges.value) return;
      saving.value = true;
      error.value = "";
      success.value = "";
      try {
        const courtPayload = [];
        categories.value.forEach((category) => {
          category.courts.forEach((court) => {
            if (court.changed) {
              courtPayload.push({
                type: category.type,
                number: court.number,
                isAvailable: court.available,
                note: court.note || undefined,
              });
            }
          });
        });

        const slotPayload = timeSlots.value
          .filter((slot) => slot.changed)
          .map((slot) => ({
            slotKey: slot.key,
            isAvailable: slot.available,
            note: slot.note || undefined,
          }));

        const response = await api.adminVenue.updateStatus({
          date: currentDateParam.value,
          courts: courtPayload.length ? courtPayload : undefined,
          timeSlots: slotPayload.length ? slotPayload : undefined,
        });

        if (!response || response.code !== 0) {
          throw new Error(response?.message || "更新失败");
        }

        success.value = response.message || "更新成功";
        await fetchOverview();
      } catch (err) {
        console.error("更新场地状态失败", err);
        error.value = err?.message || "更新场地状态失败";
      } finally {
        saving.value = false;
      }
    };

    const handleLogout = () => {
      localStorage.removeItem("token");
      localStorage.removeItem("adminUser");
      localStorage.removeItem("user");
      window.location.href = "/admin/login";
    };

    const ensureAuth = async () => {
      try {
        await api.auth.getCurrentUser();
      } catch (err) {
        console.warn("管理员身份校验失败", err);
        // 拦截器会自动跳转
      }
    };

    onMounted(async () => {
      await ensureAuth();
      await fetchOverview();
    });

    return {
      categories,
      timeSlots,
      loading,
      saving,
      error,
      success,
      formattedCurrentDate,
      hasChanges,
      fetchOverview,
      changeDate,
      toggleCourt,
      markCourtChanged,
      toggleSlot,
      markSlotChanged,
      saveChanges,
      handleLogout,
    };
  },
};
</script>

<style scoped>
.admin-venue-page {
  min-height: 100vh;
  padding: 24px 16px 48px;
  background: #f5f7fb;
  color: #0f172a;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
}

.subtitle {
  margin: 6px 0 0;
  color: #64748b;
  font-size: 13px;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.control-bar {
  display: flex;
  flex-direction: column;
  gap: 16px;
  align-items: stretch;
}

@media (min-width: 640px) {
  .control-bar {
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
  }
}

.date-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  background: #fff;
  border-radius: 12px;
  padding: 10px 16px;
  box-shadow: 0 6px 16px rgba(15, 23, 42, 0.08);
}

.date-btn {
  border: none;
  background: none;
  font-size: 15px;
  color: #475569;
  cursor: pointer;
}

.current-date {
  font-weight: 600;
  color: #0f172a;
}

.control-actions {
  display: flex;
  gap: 10px;
}

.ghost-btn,
.primary-btn {
  border: none;
  border-radius: 999px;
  padding: 10px 20px;
  font-size: 14px;
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.ghost-btn {
  background: #e2e8f0;
  color: #0f172a;
}

.primary-btn {
  background: linear-gradient(135deg, #0f9d58, #0bb172);
  color: #fff;
}

.primary-btn:disabled,
.ghost-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.feedback-area {
  margin: 20px 0;
}

.info {
  padding: 12px 16px;
  border-radius: 12px;
  font-size: 13px;
}

.info--loading {
  background: rgba(59, 130, 246, 0.08);
  color: #1d4ed8;
}

.info--error {
  background: rgba(239, 68, 68, 0.12);
  color: #b91c1c;
}

.info--success {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.info--muted {
  background: rgba(148, 163, 184, 0.12);
  color: #475569;
}

.category-block,
.time-block {
  background: #fff;
  border-radius: 16px;
  padding: 18px;
  box-shadow: 0 10px 24px rgba(15, 23, 42, 0.08);
  margin-bottom: 24px;
}

.category-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.category-header h2 {
  margin: 0;
  font-size: 18px;
}

.meta {
  font-size: 12px;
  color: #64748b;
}

.courts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
  gap: 16px;
}

.court-item,
.slot-item {
  border: 1px solid #e2e8f0;
  border-radius: 14px;
  padding: 14px;
  background: #f8fafc;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.court-top,
.slot-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
}

.court-number,
.slot-label {
  font-weight: 600;
  color: #0f172a;
}

.toggle-btn {
  border: none;
  border-radius: 999px;
  padding: 6px 14px;
  font-size: 13px;
  cursor: pointer;
  transition: transform 0.15s ease;
}

.toggle-btn.available {
  background: rgba(16, 185, 129, 0.16);
  color: #047857;
}

.toggle-btn.booked {
  background: rgba(248, 113, 113, 0.16);
  color: #b91c1c;
}

.toggle-btn:active {
  transform: scale(0.97);
}

.note-row input {
  width: 100%;
  border: 1px solid #cbd5f5;
  border-radius: 10px;
  padding: 8px 10px;
  font-size: 13px;
  background: #fff;
}

.note-row input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.meta-row {
  font-size: 12px;
  color: #94a3b8;
}

.slots-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
}
</style>
