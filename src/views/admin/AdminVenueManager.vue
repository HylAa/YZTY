<template>
  <div class="admin-venue-page">
    <header class="page-header">
      <div>
        <h1>场馆占用管理</h1>
        <p class="subtitle">配置不同运动的场地与时段占用情况</p>
      </div>
      <div class="header-actions">
        <button class="ghost-btn" @click="handleLogout">退出登录</button>
      </div>
    </header>

    <section class="control-bar">
      <div class="date-selector">
        <button class="date-btn" @click="changeDate(-1)">← 前一天</button>
        <div class="current-date">{{ formattedCurrentDate }}</div>
        <button class="date-btn" @click="changeDate(1)">后一天 →</button>
      </div>
      <div class="control-actions">
        <button class="ghost-btn" :disabled="loading" @click="fetchOverview">
          刷新
        </button>
        <button class="primary-btn" :disabled="!hasChanges || saving" @click="saveChanges">
          {{ saving ? "保存中..." : "保存变更" }}
        </button>
      </div>
    </section>

    <section class="feedback-area">
      <div v-if="loading" class="info info--loading">正在加载数据...</div>
      <div v-else-if="error" class="info info--error">{{ error }}</div>
      <div v-else-if="success" class="info info--success">{{ success }}</div>
    </section>

    <section v-if="!loading && !error" class="content">
      <div v-if="!activeTimeSlots.length" class="info info--muted">暂无时段数据</div>

      <div class="time-manage">
        <div class="time-header">
          <div>
            <h2>时段占用管理</h2>
            <p class="time-hint">{{ timeSlotHint }}</p>
          </div>
          <div class="time-tabs">
            <button
              v-for="sport in sportList"
              :key="sport.id"
              type="button"
              class="time-tab"
              :class="{ active: sport.id === timeSlotSport }"
              @click="changeTimeSlotSport(sport.id)"
            >
              {{ sport.label }}
            </button>
          </div>
        </div>
        <div class="slots-grid">
          <div v-for="slot in activeTimeSlots" :key="slot.key" class="slot-item">
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
                type="text"
                placeholder="备注（可选）"
                @input="markSlotChanged(slot)"
              />
            </div>
            <div v-if="slot.courts && slot.courts.length" class="slot-courts">
              <button
                v-for="court in slot.courts"
                :key="`${slot.key}-${court.number}`"
                type="button"
                class="slot-court-btn"
                :class="{
                  available: court.available,
                  booked: !court.available,
                  dirty: court.changed,
                }"
                @click="toggleSlotCourt(slot, court)"
              >
                {{ court.label }}
              </button>
            </div>
            <div v-if="slot.updatedAt" class="meta-row">
              最近更新：{{ slot.updatedAt }}
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script>
import { ref, computed, onMounted } from "vue";
import api from "../../api";

const WEEK_MAP = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

const SPORT_CONFIG = [
  {
    id: "badminton",
    label: "羽毛球",
    courtTotal: 18,
    courtLabels: [],
    slotMinutes: 30,
    openStart: 9 * 60,
    openEnd: 21 * 60,
    minUnits: 2,
  },
  {
    id: "basketball",
    label: "篮球",
    courtTotal: 4,
    courtLabels: ["1A", "1B", "2A", "2B"],
    slotMinutes: 30,
    openStart: 9 * 60,
    openEnd: 21 * 60,
    minUnits: 2,
  },
  {
    id: "football",
    label: "足球",
    courtTotal: 2,
    courtLabels: ["南场", "北场"],
    slotMinutes: 30,
    openStart: 8 * 60,
    openEnd: 22 * 60 + 30,
    minUnits: 2,
  },
];

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

function minutesToLabel(minutes) {
  const hours = Math.floor(minutes / 60)
    .toString()
    .padStart(2, "0");
  const mins = (minutes % 60).toString().padStart(2, "0");
  return `${hours}:${mins}`;
}

function buildSlotKey(sportId, startLabel, endLabel) {
  return `${sportId}|${startLabel}-${endLabel}`;
}

function buildSlotCourtsBase(config) {
  return Array.from({ length: config.courtTotal }, (_, index) => {
    const number = index + 1;
    const label = config.courtLabels?.[index] || `${config.label}${number}号场`;
    return {
      number,
      label,
      available: true,
      initialAvailable: true,
      note: "",
      initialNote: "",
      updatedAt: "",
      changed: false,
    };
  });
}

function generateSlotTemplates(config) {
  const result = [];
  for (
    let start = config.openStart;
    start + config.slotMinutes <= config.openEnd;
    start += config.slotMinutes
  ) {
    const end = start + config.slotMinutes;
    const startLabel = minutesToLabel(start);
    const endLabel = minutesToLabel(end);
    result.push({
      key: buildSlotKey(config.id, startLabel, endLabel),
      label: startLabel,
      startTime: startLabel,
      endTime: endLabel,
      available: true,
      initialAvailable: true,
      note: "",
      initialNote: "",
      updatedAt: "",
      changed: false,
      courts: buildSlotCourtsBase(config),
    });
  }
  return result;
}

function updateCourtChangedState(slot, court) {
  const note = court.note || "";
  const initialNote = court.initialNote || "";
  court.changed = court.available !== court.initialAvailable || note !== initialNote;
  updateSlotChangedState(slot);
}

function updateSlotChangedState(slot) {
  const note = slot.note || "";
  const initialNote = slot.initialNote || "";
  const hasCourtDiff = (slot.courts || []).some((court) => court.changed);
  slot.changed = slot.available !== slot.initialAvailable || note !== initialNote || hasCourtDiff;
}

export default {
  name: "AdminVenueManager",
  setup() {
    const currentDate = ref(new Date());
    const timeSlotsBySport = ref({});
    const timeSlotSport = ref(SPORT_CONFIG[0].id);
    const loading = ref(false);
    const saving = ref(false);
    const error = ref("");
    const success = ref("");

    const formattedCurrentDate = computed(() => formatDisplayDate(currentDate.value));
    const currentDateParam = computed(() => formatDateParam(currentDate.value));

    const sportList = SPORT_CONFIG;

    const activeTimeSlots = computed(
      () => timeSlotsBySport.value[timeSlotSport.value] || []
    );

    const timeSlotMeta = computed(() =>
      SPORT_CONFIG.find((item) => item.id === timeSlotSport.value)
    );

    const timeSlotHint = computed(() => {
      const meta = timeSlotMeta.value;
      if (!meta) return "";
      const minimum = meta.slotMinutes * meta.minUnits;
      const durationText =
        minimum % 60 === 0 ? `${minimum / 60}小时` : `${minimum}分钟`;
      return `基础单位 ${meta.slotMinutes} 分钟，至少连续 ${durationText}`;
    });

    const adaptTimeSlotsBySport = (map = {}) => {
      const result = {};
      const globalMap = new Map(
        (map.ALL || []).map((item) => [
          buildSlotKey("ALL", item.startTime, item.endTime),
          item,
        ])
      );
      SPORT_CONFIG.forEach((config) => {
        const base = generateSlotTemplates(config);
        const recordMap = new Map(
          (map[config.id] || []).map((item) => [
            item.slotKey || buildSlotKey(config.id, item.startTime, item.endTime),
            item,
          ])
        );
        result[config.id] = base.map((slot) => {
          const record =
            recordMap.get(slot.key) ||
            globalMap.get(buildSlotKey("ALL", slot.startTime, slot.endTime));

          const nextSlot = {
            ...slot,
            available: record ? Boolean(record.isAvailable) : slot.available,
            note: record?.note || "",
            updatedAt: record?.updated_at || record?.updatedAt || "",
            changed: false,
          };

          nextSlot.initialAvailable = nextSlot.available;
          nextSlot.initialNote = nextSlot.note;

          const baseCourts = slot.courts.map((court) => ({ ...court }));
          const recordCourts = record?.courts || [];
          const courtMap = new Map(recordCourts.map((court) => [court.number, court]));

          nextSlot.courts = baseCourts.map((court) => {
            const recordCourt = courtMap.get(court.number);
            const available = recordCourt
              ? Boolean(recordCourt.isAvailable)
              : nextSlot.available;
            const note = recordCourt?.note || "";
            const updatedAt = recordCourt?.updated_at || recordCourt?.updatedAt || "";

            return {
              ...court,
              available,
              note,
              updatedAt,
              initialAvailable: available,
              initialNote: note,
              changed: false,
            };
          });

          return nextSlot;
        });
      });
      return result;
    };

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
        timeSlotsBySport.value = adaptTimeSlotsBySport(
          data.timeSlotsBySport || {}
        );
        if (!timeSlotsBySport.value[timeSlotSport.value]) {
          timeSlotSport.value = SPORT_CONFIG[0].id;
        }
      } catch (err) {
        console.error("获取场馆数据失败", err);
        error.value = err?.message || "获取场馆数据失败";
        timeSlotsBySport.value = {};
      } finally {
        loading.value = false;
      }
    };

    const changeDate = async (offset) => {
      currentDate.value = adjustDate(currentDate.value, offset);
      await fetchOverview();
    };

    const changeTimeSlotSport = (sportId) => {
      if (timeSlotSport.value === sportId) return;
      timeSlotSport.value = sportId;
      success.value = "";
    };

    const toggleSlot = (slot) => {
      slot.available = !slot.available;
      (slot.courts || []).forEach((court) => {
        court.available = slot.available;
        updateCourtChangedState(slot, court);
      });
      updateSlotChangedState(slot);
      success.value = "";
    };

    const markSlotChanged = (slot) => {
      updateSlotChangedState(slot);
      success.value = "";
    };

    const toggleSlotCourt = (slot, court) => {
      court.available = !court.available;
      updateCourtChangedState(slot, court);
      success.value = "";
    };

    const hasChanges = computed(() => {
      return Object.values(timeSlotsBySport.value).some((slots) =>
        (slots || []).some((slot) => slot.changed)
      );
    });

    const saveChanges = async () => {
      if (!hasChanges.value) return;
      saving.value = true;
      error.value = "";
      success.value = "";
      try {
        const slotPayload = [];
        Object.entries(timeSlotsBySport.value).forEach(([sportId, slots]) => {
          (slots || []).forEach((slot) => {
            if (!slot.changed) return;
            const courtsChanged = (slot.courts || [])
              .filter((court) => court.changed)
              .map((court) => ({
                number: court.number,
                isAvailable: court.available,
                note: court.note || undefined,
              }));
            slotPayload.push({
              type: sportId,
              startTime: slot.startTime,
              endTime: slot.endTime,
              isAvailable: slot.available,
              note: slot.note || undefined,
              courts: courtsChanged.length ? courtsChanged : undefined,
            });
          });
        });

        const response = await api.adminVenue.updateStatus({
          date: currentDateParam.value,
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
      sportList,
      timeSlotsBySport,
      timeSlotSport,
      activeTimeSlots,
      timeSlotHint,
      loading,
      saving,
      error,
      success,
      formattedCurrentDate,
      hasChanges,
      fetchOverview,
      changeDate,
      changeTimeSlotSport,
      toggleSlot,
      markSlotChanged,
      toggleSlotCourt,
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
  background: rgba(59, 130, 246, 0.12);
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

.time-manage {
  background: #fff;
  border-radius: 16px;
  padding: 18px;
  box-shadow: 0 10px 24px rgba(15, 23, 42, 0.08);
  margin-bottom: 24px;
}

.meta {
  font-size: 12px;
  color: #64748b;
}

.slot-item {
  border: 1px solid #e2e8f0;
  border-radius: 14px;
  padding: 14px;
  background: #f8fafc;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.slot-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
}

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

.slot-courts {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.slot-court-btn {
  border: none;
  border-radius: 10px;
  padding: 6px 12px;
  font-size: 12px;
  cursor: pointer;
  background: #e2e8f0;
  color: #1f2937;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.slot-court-btn.available {
  background: rgba(34, 197, 94, 0.16);
  color: #047857;
}

.slot-court-btn.booked {
  background: rgba(248, 113, 113, 0.16);
  color: #b91c1c;
}

.slot-court-btn.dirty {
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.25);
}

.slot-court-btn:active {
  transform: scale(0.97);
}

.meta-row {
  font-size: 12px;
  color: #94a3b8;
}

.time-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
}

.time-hint {
  margin: 6px 0 0;
  font-size: 12px;
  color: #64748b;
}

.time-tabs {
  display: flex;
  gap: 8px;
}

.time-tab {
  border: none;
  border-radius: 999px;
  padding: 6px 14px;
  font-size: 13px;
  cursor: pointer;
  background: #e2e8f0;
  color: #1f2937;
  transition: background 0.2s ease;
}

.time-tab.active {
  background: #0ea5e9;
  color: #fff;
}

.slots-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
}

@media (max-width: 640px) {
  .slots-grid {
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  }
}
</style>
