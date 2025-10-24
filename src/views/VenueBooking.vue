<template>
  <div class="venue-booking-page">
    <div class="header">
      <h1>场馆占用情况</h1>
      <p>查看各类场地与时段的实时可用状态</p>
    </div>

    <div class="date-selector">
      <button class="date-btn" @click="notifyDateChange('prev')">
        ← 前一天
      </button>
      <div class="current-date">
        {{ formattedCurrentDate }}
      </div>
      <button class="date-btn" @click="notifyDateChange('next')">
        后一天 →
      </button>
    </div>

    <div class="content-area">
      <div v-if="loading" class="loading-state">正在加载场地状态...</div>
      <div v-else>
        <div v-if="error" class="info-bar error">
          {{ error }}
        </div>
        <template v-else>
          <div v-if="!categories.length" class="info-bar empty">
            暂无场地占用数据
          </div>
          <section
            v-for="category in categories"
            :key="category.type"
            class="category"
          >
            <div class="category-title">
              {{ category.displayTitle }}
            </div>
            <div :id="category.containerId" class="courts-container">
              <div
                v-for="court in category.courts"
                :key="court.id"
                class="court-card"
                :class="[
                  `${category.type}-court`,
                  { booked: !court.available },
                ]"
              >
                <div class="court-number-badge">
                  <span class="badge-text">场地 {{ court.number }}</span>
                </div>
                <div class="court-name">
                  {{ category.courtLabel }}
                </div>
                <div
                  class="status"
                  :class="court.available ? 'available' : 'booked'"
                >
                  {{ court.available ? "未占用" : "已占用" }}
                </div>
                <div v-if="court.note" class="court-note">
                  备注：{{ court.note }}
                </div>
                <div v-else-if="court.updatedAt" class="court-note">
                  更新于 {{ court.updatedAt }}
                </div>
              </div>
            </div>
          </section>

          <!-- <div v-if="timeSlots.length" class="time-slots">
            <div class="time-title">时段占用情况</div>
            <div class="slots-container">
              <div
                v-for="slot in timeSlots"
                :key="slot.key"
                class="time-slot"
                :class="{ booked: !slot.available }"
              >
                <span class="slot-time">{{ slot.label }}</span>
                <span class="slot-status">{{ slot.available ? "可用" : "占用" }}</span>
                <span v-if="slot.note" class="slot-note">{{ slot.note }}</span>
                <span v-else-if="slot.updatedAt" class="slot-note"
                  >更新时间 {{ slot.updatedAt }}</span
                >
              </div>
            </div>
          </div> -->
        </template>
      </div>
    </div>
  </div>
</template>
<script>
import { ref, computed, onMounted } from "vue";
import api from "../api";

const WEEK_MAP = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

const COURT_META = {
  badminton: {
    label: "羽毛球场地",
    name: "羽毛球",
    containerId: "badminton-courts",
  },
  basketball: {
    label: "篮球场地",
    name: "篮球",
    containerId: "basketball-courts",
  },
  football: {
    label: "足球场地",
    name: "足球",
    containerId: "football-courts",
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
  name: "VenueBooking",
  setup() {
    const currentDate = ref(new Date());
    const categories = ref([]);
    const timeSlots = ref([]);
    const loading = ref(false);
    const error = ref("");

    const formattedCurrentDate = computed(() =>
      formatDisplayDate(currentDate.value)
    );
    const currentDateParam = computed(() => formatDateParam(currentDate.value));

    const adaptCategories = (list = []) =>
      list.map((category) => {
        const type = category.type;
        const meta = COURT_META[type] || {
          label: category.title,
          name: category.title,
          containerId: `${type}-courts`,
        };
        const courts = (category.courts || []).map((court) => ({
          id: `${type}-${court.number}`,
          number: court.number,
          available: court.isAvailable,
          note: court.note ?? null,
          updatedAt: court.updated_at ?? null,
        }));
        return {
          type,
          displayTitle: `${category.title} (${
            category.total ?? courts.length
          }片)`,
          courtLabel: meta.name,
          containerId: meta.containerId,
          courts,
        };
      });

    const adaptTimeSlots = (list = []) =>
      list.map((slot) => ({
        key: slot.slotKey,
        label: `${slot.startTime}-${slot.endTime}`,
        available: slot.isAvailable,
        note: slot.note ?? null,
        updatedAt: slot.updated_at ?? null,
      }));

    const fetchOverview = async () => {
      loading.value = true;
      error.value = "";
      try {
        const response = await api.venue.getOverview(currentDateParam.value);
        if (!response || response.code !== 0) {
          throw new Error(response?.message || "获取场地数据失败");
        }
        const data = response.data || {};
        categories.value = adaptCategories(data.categories);
        timeSlots.value = adaptTimeSlots(data.timeSlots);
      } catch (err) {
        console.error("加载场地状态失败", err);
        error.value = err?.message || "加载场地状态失败";
        categories.value = [];
        timeSlots.value = [];
      } finally {
        loading.value = false;
      }
    };

    const notifyDateChange = async (direction) => {
      const offset = direction === "prev" ? -1 : 1;
      currentDate.value = adjustDate(currentDate.value, offset);
      await fetchOverview();
    };

    onMounted(fetchOverview);

    return {
      categories,
      timeSlots,
      loading,
      error,
      formattedCurrentDate,
      notifyDateChange,
    };
  },
};
</script>
<style scoped>
.venue-booking-page {
  max-width: 500px;
  margin: 0 auto;
  padding: 15px;
  padding-bottom: calc(80px + env(safe-area-inset-bottom, 0px));
  min-height: 100vh;
  background-color: #f5f5f5;
  color: #333;
  box-sizing: border-box;
  font-family: "PingFang SC", "Helvetica Neue", Arial, sans-serif;
}

.header {
  text-align: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid #eaeaea;
}

.content-area {
  margin-top: 10px;
}

.loading-state {
  padding: 40px 0;
  text-align: center;
  color: #666;
  font-size: 14px;
}

.info-bar {
  margin-bottom: 16px;
  padding: 12px;
  border-radius: 8px;
  font-size: 13px;
}

.info-bar.error {
  background-color: rgba(255, 76, 76, 0.12);
  color: #c62828;
}

.info-bar.empty {
  background-color: rgba(7, 193, 96, 0.08);
  color: #0ea75a;
  text-align: center;
}

.header h1 {
  font-size: 20px;
  color: #1a1a1a;
  margin-bottom: 5px;
}

.date-selector {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 15px;
  background-color: #fff;
  border-radius: 8px;
  padding: 12px;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
}
.date-btn {
  background: none;
  border: none;
  font-size: 18px;
  color: #666;
}

.current-date {
  font-weight: bold;
  color: #1a1a1a;
}

.category {
  margin-bottom: 20px;
}

.category-title {
  font-size: 18px;
  margin-bottom: 10px;
  padding-left: 5px;
  border-left: 4px solid #07c160;
  color: #1a1a1a;
}

.courts-container {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 18px;
  padding-top: 24px;
}

.court-card {
  background-color: #fff;
  border-radius: 12px;
  padding: 38px 12px 16px;
  text-align: center;
  box-shadow: 0 6px 14px rgba(7, 193, 96, 0.12);
  transition: all 0.2s;
  position: relative;
  overflow: visible;
  min-height: 124px;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  gap: 10px;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}

.court-card.booked {
  filter: grayscale(0.1);
  opacity: 0.85;
}

.badminton-court {
  background-image: linear-gradient(rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0.3)),
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect width="100" height="100" fill="%23e8f5e9"/><rect x="10" y="10" width="80" height="80" fill="none" stroke="%234caf50" stroke-width="2"/><line x1="50" y1="10" x2="50" y2="90" stroke="%234caf50" stroke-width="2"/><circle cx="50" cy="50" r="5" fill="%234caf50"/></svg>');
}

.basketball-court {
  background-image: linear-gradient(rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0.3)),
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect width="100" height="100" fill="%23ffecb3"/><rect x="10" y="10" width="80" height="80" fill="none" stroke="%23ff9800" stroke-width="2"/><circle cx="50" cy="50" r="20" fill="none" stroke="%23ff9800" stroke-width="2"/><line x1="10" y1="50" x2="90" y2="50" stroke="%23ff9800" stroke-width="2"/><circle cx="50" cy="50" r="5" fill="%23ff9800"/></svg>');
}

.football-court {
  background-image: linear-gradient(rgba(0, 0, 0, 0.3), rgba(0, 0, 0, 0.3)),
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect width="100" height="100" fill="%23c8e6c9"/><rect x="10" y="10" width="80" height="80" fill="none" stroke="%232e7d32" stroke-width="2"/><circle cx="50" cy="50" r="15" fill="none" stroke="%232e7d32" stroke-width="2"/><line x1="50" y1="10" x2="50" y2="90" stroke="%232e7d32" stroke-width="2"/><rect x="10" y="35" width="10" height="30" fill="none" stroke="%232e7d32" stroke-width="2"/><rect x="80" y="35" width="10" height="30" fill="none" stroke="%232e7d32" stroke-width="2"/></svg>');
}
.court-card:active {
  transform: scale(0.98);
}

.court-name {
  font-size: 16px;
  margin: 0;
  color: #fff;
  font-weight: 600;
  text-shadow: 1px 1px 3px rgba(0, 0, 0, 0.6);
  letter-spacing: 1px;
}

.court-number-badge {
  position: absolute;
  top: -14px;
  left: 50%;
  transform: translateX(-50%);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 999px;
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.95),
    rgba(255, 255, 255, 0.75)
  );
  color: #0ea75a;
  font-weight: 600;
  font-size: 12px;
  box-shadow: 0 10px 20px rgba(14, 167, 90, 0.18);
  backdrop-filter: blur(4px);
}

.badge-text {
  letter-spacing: 0.5px;
  writing-mode: horizontal-tb;
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
  font-size: 14px;
  font-weight: 600;
  color: #0ea75a;
}

.status {
  font-size: 12px;
  padding: 5px 16px;
  border-radius: 16px;
  display: inline-block;
  background-color: rgba(232, 245, 233, 0.92);
  color: #1b7d35;
  letter-spacing: 1px;
  margin-top: auto;
}

.status.booked {
  background-color: rgba(255, 235, 238, 0.9);
  color: #c62828;
}

.court-card.booked .status {
  background-color: rgba(255, 235, 238, 0.9);
  color: #c62828;
}

.court-note {
  margin-top: 6px;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.85);
  background-color: rgba(0, 0, 0, 0.25);
  padding: 4px 8px;
  border-radius: 12px;
}

.time-slots {
  background-color: #fff;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 20px;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
}

.time-title {
  font-size: 16px;
  margin-bottom: 10px;
  color: #1a1a1a;
}

.slots-container {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.time-slot {
  padding: 10px 6px;
  text-align: center;
  background-color: #f5f5f5;
  border-radius: 6px;
  font-size: 13px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.04);
}

.time-slot.booked {
  background-color: #ffcdd2;
  box-shadow: inset 0 0 0 1px rgba(198, 40, 40, 0.25);
}

.slot-time {
  font-weight: 600;
  color: #1a1a1a;
}

.slot-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 12px;
  background-color: rgba(7, 193, 96, 0.1);
  color: #0ea75a;
}

.time-slot.booked .slot-status {
  background-color: rgba(198, 40, 40, 0.1);
  color: #c62828;
}

.slot-note {
  font-size: 11px;
  color: #666;
}
</style>
