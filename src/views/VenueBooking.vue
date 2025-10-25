<template>
  <div class="venue-query-page">
    <header class="page-header">
      <h1>场馆查询系统</h1>
      <p>查询场地可用状态</p>
    </header>

    <div class="sport-tabs">
      <button
        v-for="sport in sportList"
        :key="sport.id"
        type="button"
        class="sport-tab"
        :class="{ active: sport.id === activeSport }"
        @click="changeSport(sport.id)"
      >
        {{ sport.label }}
      </button>
    </div>

    <section class="date-selector">
      <button type="button" class="date-btn" @click="changeDate(-1)">
        ← 前一天
      </button>
      <div class="current-date">{{ formattedCurrentDate }}</div>
      <button type="button" class="date-btn" @click="changeDate(1)">
        后一天 →
      </button>
    </section>

    <section class="rules-card">
      <strong>预约规则：</strong>
      <ul>
        <li v-for="rule in activeRules" :key="rule">{{ rule }}</li>
      </ul>
    </section>

    <section class="time-section">
      <div class="time-header">
        <h2>选择查询时段</h2>
        <span v-if="rangePreview" class="range-preview">{{ rangePreview }}</span>
      </div>
      <div class="time-slots">
        <button
          v-for="slot in timeSlotOptions"
          :key="slot.key"
          type="button"
          class="time-slot"
          :class="slotClass(slot)"
          :disabled="slot.disabled"
          @click="handleSlotClick(slot)"
        >
          {{ slot.label }}
        </button>
      </div>
      <p class="time-hint">
        至少连续选择 {{ activeSportConfig.minUnits * (activeSportConfig.slotMinutes / 30) }} 个时间单位（{{ minimumDurationText }}）
      </p>
    </section>

    <section class="info-area">
      <div v-if="loading" class="info info--loading">正在加载场地状态...</div>
      <div v-else-if="error" class="info info--error">{{ error }}</div>
      <div v-else-if="success" class="info info--success">{{ success }}</div>
    </section>

    <section class="results-card">
      <div class="results-title">
        <h2>场地可用状态</h2>
        <span v-if="queriedRange" class="results-range">{{ resultRangeText }}</span>
      </div>

      <div v-if="!queriedRange" class="results-placeholder">
        请选择开始和结束时间后点击“查询可用场地”
      </div>
      <div v-else-if="displayCourts.length === 0" class="results-placeholder">
        暂无场地数据
      </div>
      <div v-else class="courts-grid">
        <div
          v-for="court in displayCourts"
          :key="court.id"
          class="court-card"
          :class="[`${activeSport}-court`, { unavailable: !court.available }]"
        >
          <div class="court-number">{{ court.displayName }}</div>
          <div class="court-status" :class="court.available ? 'available' : 'booked'">
            {{ court.available ? '可用' : '占用' }}
          </div>
          <div v-if="court.note" class="court-note">{{ court.note }}</div>
          <div v-else-if="court.updatedAt" class="court-note">
            更新于 {{ court.updatedAt }}
          </div>
        </div>
      </div>
    </section>

    <button
      class="query-btn"
      type="button"
      :disabled="isQueryDisabled"
      @click="submitQuery"
    >
      查询可用场地
    </button>
  </div>
</template>

<script>
import { ref, computed, onMounted, watch } from "vue";
import api from "../api";

const SPORT_CONFIG = [
  {
    id: "badminton",
    label: "羽毛球",
    totalCourts: 18,
    slotMinutes: 30,
    minUnits: 2,
    openMinutes: { start: 9 * 60, end: 21 * 60 },
    weekendFullHour: true,
    courtLabels: null,
    rules: [
      "营业时间：09:00-21:00",
      "周一至周五可按半小时查询，至少连续1小时",
      "周末仅支持整点预约，至少连续1小时",
      "共 18 片场地"
    ],
  },
  {
    id: "basketball",
    label: "篮球",
    totalCourts: 4,
    slotMinutes: 30,
    minUnits: 2,
    openMinutes: { start: 9 * 60, end: 21 * 60 },
    weekendFullHour: false,
    courtLabels: ["1A", "1B", "2A", "2B"],
    rules: [
      "营业时间：09:00-21:00",
      "支持半小时为单位的查询，至少连续1小时",
      "可跨越半点，示例：09:30-11:30",
      "共 4 块场地（1A/1B/2A/2B）"
    ],
  },
  {
    id: "football",
    label: "足球",
    totalCourts: 2,
    slotMinutes: 30,
    minUnits: 2,
    openMinutes: { start: 8 * 60, end: 22 * 60 + 30 },
    weekendFullHour: false,
    courtLabels: ["南场", "北场"],
    rules: [
      "营业时间：08:00-22:30",
      "半小时为单位，至少连续1小时",
      "共 2 块场地（南场 / 北场）"
    ],
  },
];

const minutesToLabel = (minutes) => {
  const h = Math.floor(minutes / 60)
    .toString()
    .padStart(2, "0");
  const m = (minutes % 60).toString().padStart(2, "0");
  return `${h}:${m}`;
};

const labelToMinutes = (label) => {
  const [hours, mins] = label.split(":").map((item) => parseInt(item, 10));
  if (Number.isNaN(hours) || Number.isNaN(mins)) {
    return 0;
  }
  return hours * 60 + mins;
};

const formatRangeText = (start, end) => `${minutesToLabel(start)} - ${minutesToLabel(end)}`;

const buildSlotKey = (sportId, startLabel, endLabel) => `${sportId}|${startLabel}-${endLabel}`;

const buildSlotCourtsBase = (meta) => {
  const baseCourts = buildBaseCourts(meta);
  return baseCourts.map((court, index) => ({
    number: index + 1,
    label: court.displayName,
    available: true,
    note: "",
    updatedAt: "",
  }));
};

const generateSlotTemplates = (meta) => {
  const templates = [];
  for (
    let start = meta.openMinutes.start;
    start + meta.slotMinutes <= meta.openMinutes.end;
    start += meta.slotMinutes
  ) {
    const end = start + meta.slotMinutes;
    const startLabel = minutesToLabel(start);
    const endLabel = minutesToLabel(end);
    templates.push({
      key: buildSlotKey(meta.id, startLabel, endLabel),
      label: startLabel,
      startTime: startLabel,
      endTime: endLabel,
      available: true,
      note: "",
      updatedAt: "",
      courts: buildSlotCourtsBase(meta),
    });
  }
  return templates;
};

const adaptTimeSlotsBySport = (source = {}) => {
  const adapted = {};
  const globalMap = new Map(
    (source.ALL || []).map((item) => [
      buildSlotKey("ALL", item.startTime, item.endTime),
      item,
    ])
  );
  SPORT_CONFIG.forEach((meta) => {
    const baseSlots = generateSlotTemplates(meta);
    const recordMap = new Map(
      (source[meta.id] || []).map((item) => [
        item.slotKey || buildSlotKey(meta.id, item.startTime, item.endTime),
        item,
      ])
    );
    adapted[meta.id] = baseSlots.map((slot) => {
      const record =
        recordMap.get(slot.key) ||
        globalMap.get(buildSlotKey("ALL", slot.startTime, slot.endTime));
      if (!record) {
        return slot;
      }
      const nextSlot = {
        ...slot,
        available: record ? Boolean(record.isAvailable) : slot.available,
        note: record?.note || "",
        updatedAt: record?.updated_at || record?.updatedAt || "",
      };

      const baseCourts = slot.courts.map((court) => ({
        ...court,
      }));
      const recordCourts = record?.courts || [];
      const courtMap = new Map(
        recordCourts.map((court) => [court.number, court])
      );
      nextSlot.courts = baseCourts.map((court) => {
        const recordCourt = courtMap.get(court.number);
        const available = recordCourt
          ? Boolean(recordCourt.isAvailable)
          : nextSlot.available;
        return {
          ...court,
          available,
          note: recordCourt?.note || "",
          updatedAt:
            recordCourt?.updated_at || recordCourt?.updatedAt || nextSlot.updatedAt,
        };
      });
      return nextSlot;
    });
  });
  return adapted;
};

export default {
  name: "VenueBooking",
  setup() {
    const activeSport = ref("badminton");
    const currentDate = ref(new Date());
    const selectedStart = ref(null); // minutes from midnight（开始时间）
    const selectedEnd = ref(null); // minutes from midnight（结束时间，独占）
    const queriedRange = ref(null); // { sportId, start, end }
    const loading = ref(false);
    const error = ref("");
    const success = ref("");
    const timeSlotsBySport = ref({});
    const lastLoadedDate = ref("");
    const availabilityResult = ref(null);

    const sportList = SPORT_CONFIG;

    const activeSportConfig = computed(() =>
      SPORT_CONFIG.find((item) => item.id === activeSport.value)
    );

    const formattedCurrentDate = computed(() => {
      const options = { year: "numeric", month: "numeric", day: "numeric", weekday: "long" };
      return currentDate.value.toLocaleDateString("zh-CN", options);
    });

    const currentDateParam = computed(() => {
      const y = currentDate.value.getFullYear();
      const m = (currentDate.value.getMonth() + 1).toString().padStart(2, "0");
      const d = currentDate.value.getDate().toString().padStart(2, "0");
      return `${y}-${m}-${d}`;
    });

    const isWeekend = computed(() => {
      const day = currentDate.value.getDay();
      return day === 0 || day === 6;
    });

    const minimumDurationText = computed(() => {
      const minutes = activeSportConfig.value.minUnits * activeSportConfig.value.slotMinutes;
      if (minutes % 60 === 0) {
        return `${minutes / 60}小时`;
      }
      return `${minutes}分钟`;
    });

    const activeRules = computed(() => activeSportConfig.value.rules);

    const timeSlotOptions = computed(() => {
      const config = activeSportConfig.value;
      const slots = [];
      const slotStateMap = new Map(
        (timeSlotsBySport.value[config.id] || []).map((item) => [
          labelToMinutes(item.startTime),
          item,
        ])
      );
      const latestStart = config.openMinutes.end - config.slotMinutes * config.minUnits;
      const selectedStartValue = selectedStart.value;
      for (let start = config.openMinutes.start; start <= latestStart; start += config.slotMinutes) {
        const isDisabled = isSlotDisabled(config, start, isWeekend.value);
        const slotState = slotStateMap.get(start);
        const hasCourtAvailable = slotState
          ? (slotState.courts || []).some((court) => court.available)
          : true;
        const slotFullyUnavailable = slotState
          ? !slotState.available && !hasCourtAvailable
          : false;
        slots.push({
          key: start,
          start,
          label: minutesToLabel(start),
          disabled: isDisabled || slotFullyUnavailable,
          unavailable: slotFullyUnavailable,
          boundaryOnly: false,
        });
      }
      // 追加闭馆时间用于选择截止时间
      slots.push({
        key: `${config.id}-end`,
        start: config.openMinutes.end,
        label: minutesToLabel(config.openMinutes.end),
        disabled: selectedStartValue === null,
        unavailable: false,
        boundaryOnly: true,
      });
      return slots;
    });

    const selectedSlotsSet = computed(() => {
      const set = new Set();
      if (selectedStart.value === null) {
        return set;
      }
      const config = activeSportConfig.value;
      const step = config.slotMinutes;
      const minEnd = selectedStart.value + config.slotMinutes * config.minUnits;
      const endExclusive =
        selectedEnd.value !== null
          ? selectedEnd.value
          : Math.min(minEnd, config.openMinutes.end);
      for (let m = selectedStart.value; m < endExclusive; m += step) {
        set.add(m);
      }
      return set;
    });

    const rangePreview = computed(() => {
      if (selectedStart.value === null) return "";
      const config = activeSportConfig.value;
      const start = selectedStart.value;
      const minEnd = start + config.slotMinutes * config.minUnits;
      const endExclusive =
        selectedEnd.value !== null
          ? selectedEnd.value
          : Math.min(minEnd, config.openMinutes.end);
      return `${formatRangeText(start, endExclusive)}（暂未提交）`;
    });

    const resultRangeText = computed(() => {
      if (!queriedRange.value) return "";
      return formatRangeText(queriedRange.value.start, queriedRange.value.end);
    });

    const isQueryDisabled = computed(
      () => selectedStart.value === null || selectedEnd.value === null || loading.value
    );

    const extractCourtId = (sportId, number) => `${sportId}-${number}`;

    const computeAvailabilityFromConfig = (sportId, startMinutes, endExclusive) => {
      const meta = SPORT_CONFIG.find((item) => item.id === sportId);
      if (!meta) {
        return null;
      }
      const baseCourts = buildBaseCourts(meta);
      const resultMap = new Map(
        baseCourts.map((court) => [
          court.id,
          {
            id: court.id,
            number: court.backendKey,
            isAvailable: true,
            note: "",
            updated_at: "",
          },
        ])
      );
      const slots = timeSlotsBySport.value[sportId] || [];
      const relevantSlots = slots.filter((slot) => {
        const slotStart = labelToMinutes(slot.startTime);
        const slotEnd = labelToMinutes(slot.endTime);
        return slotStart < endExclusive && slotEnd > startMinutes;
      });
      relevantSlots.forEach((slot) => {
        const slotCourtMap = new Map(
          (slot.courts || []).map((court) => [court.number, court])
        );
        resultMap.forEach((info) => {
          const courtRecord = slotCourtMap.get(info.number);
          const courtAvailable =
            courtRecord !== undefined
              ? courtRecord.available !== false
              : slot.available !== false;
          if (!courtAvailable) {
            info.isAvailable = false;
            const noteCandidate = courtRecord?.note || slot.note || "";
            if (noteCandidate) {
              info.note = info.note
                ? `${info.note}；${noteCandidate}`
                : noteCandidate;
            }
            const updatedCandidate =
              courtRecord?.updated_at || courtRecord?.updatedAt || slot.updatedAt || "";
            if (updatedCandidate) {
              info.updated_at = updatedCandidate;
            }
          }
        });
      });
      return {
        sport: sportId,
        courts: Array.from(resultMap.values()),
      };
    };

    const mergeAvailabilityResults = (apiResult, configResult, sportId) => {
      const mergedMap = new Map();
      (configResult?.courts || []).forEach((court) => {
        mergedMap.set(court.id, { ...court });
      });
      (apiResult?.courts || []).forEach((court) => {
        const number = court.number ?? Number(court.id?.split("-").pop() || 0);
        const id = court.id || extractCourtId(sportId, number);
        const target = mergedMap.get(id);
        const isAvailable = court.isAvailable ?? court.available ?? true;
        const noteValue = court.note || "";
        const updatedValue = court.updated_at || court.updatedAt || "";
        if (target) {
          target.isAvailable = target.isAvailable && Boolean(isAvailable);
          if (!target.note && noteValue) {
            target.note = noteValue;
          }
          if ((!target.updated_at || target.updated_at === "") && updatedValue) {
            target.updated_at = updatedValue;
          }
        } else {
          mergedMap.set(id, {
            id,
            number: number || 0,
            isAvailable: Boolean(isAvailable),
            note: noteValue,
            updated_at: updatedValue,
          });
        }
      });
      return Array.from(mergedMap.values());
    };

    const displayCourts = computed(() => {
      const meta = activeSportConfig.value;
      const baseCourts = buildBaseCourts(meta);
      const availability = availabilityResult.value;

      if (availability && availability.sport === activeSport.value) {
        const map = new Map();
        (availability.courts || []).forEach((item) => {
          map.set(item.id, item);
        });
        return baseCourts.map((court) => {
          const record = map.get(court.id);
          return {
            id: court.id,
            displayName: court.displayName,
            available: record ? Boolean(record.isAvailable) : true,
            note: record?.note || "",
            updatedAt: record?.updated_at || record?.updatedAt || "",
          };
        });
      }

      return baseCourts.map((court) => {
        return {
          id: court.id,
          displayName: court.displayName,
          available: true,
          note: "",
          updatedAt: "",
        };
      });
    });

    const invalidateResults = () => {
      availabilityResult.value = null;
      queriedRange.value = null;
      success.value = "";
    };

    const changeSport = (sportId) => {
      if (activeSport.value === sportId) return;
      activeSport.value = sportId;
      clearSelection();
    };

    const changeDate = (offset) => {
      const next = new Date(currentDate.value);
      next.setDate(next.getDate() + offset);
      currentDate.value = next;
      clearSelection();
    };

    const isSlotDisabled = (config, start, weekend) => {
      if (weekend && config.weekendFullHour && start % 60 !== 0) {
        return true;
      }
      return false;
    };

    const handleSlotClick = (slot) => {
      if (slot.disabled) return;
      const target = slot.start;
      const config = activeSportConfig.value;
      const minDuration = config.slotMinutes * config.minUnits;
      invalidateResults();

      if (selectedStart.value === null) {
        if (slot.boundaryOnly) {
          return;
        }
        selectedStart.value = target;
        selectedEnd.value = null;
        availabilityResult.value = null;
        queriedRange.value = null;
        return;
      }

      if (slot.boundaryOnly) {
        const duration = target - selectedStart.value;
        if (duration < minDuration) {
          selectedStart.value = target - minDuration;
        }
        selectedEnd.value = target;
        return;
      }

      if (target <= selectedStart.value) {
        if (target === selectedStart.value) {
          clearSelection();
        } else {
          selectedStart.value = target;
          selectedEnd.value = null;
        }
        return;
      }

      const duration = target - selectedStart.value;
      if (duration < minDuration) {
        selectedStart.value = target;
        selectedEnd.value = null;
        return;
      }

      selectedEnd.value = target;
    };

    const clearSelection = () => {
      selectedStart.value = null;
      selectedEnd.value = null;
      invalidateResults();
    };

    const fetchOverview = async () => {
      loading.value = true;
      error.value = "";
      try {
        const response = await api.venue.getOverview(currentDateParam.value);
        if (!response || response.code !== 0) {
          throw new Error(response?.message || "获取场馆数据失败");
        }
        const data = response.data || {};
        timeSlotsBySport.value = adaptTimeSlotsBySport(data.timeSlotsBySport || {});
        lastLoadedDate.value = currentDateParam.value;
      } catch (err) {
        console.error("获取场馆数据失败", err);
        error.value = err?.message || "获取场馆数据失败";
        timeSlotsBySport.value = {};
      } finally {
        loading.value = false;
      }
    };

    const ensureData = async (force = false) => {
      if (force || lastLoadedDate.value !== currentDateParam.value) {
        await fetchOverview();
      }
    };

    const submitQuery = async () => {
      if (isQueryDisabled.value || selectedStart.value === null || selectedEnd.value === null) {
        return;
      }
      loading.value = true;
      error.value = "";
      success.value = "";
      availabilityResult.value = null;
      try {
        await ensureData(true);
        if (error.value) {
          throw new Error(error.value);
        }
        // 确保在刷新配置后依旧显示加载态
        loading.value = true;

        const config = activeSportConfig.value;
        const endExclusive = selectedEnd.value;
        const startLabel = minutesToLabel(selectedStart.value);
        const endLabel = minutesToLabel(endExclusive);

        const response = await api.venue.getAvailability({
          sport: activeSport.value,
          date: currentDateParam.value,
          startTime: startLabel,
          endTime: endLabel,
        });
        if (!response || response.code !== 0) {
          throw new Error(response?.message || "查询可用场地失败");
        }
        const apiResult = response.data || null;
        const configResult = computeAvailabilityFromConfig(
          activeSport.value,
          selectedStart.value,
          endExclusive
        );
        const mergedCourts = mergeAvailabilityResults(
          apiResult,
          configResult,
          activeSport.value
        );
        const finalCourts =
          mergedCourts.length > 0
            ? mergedCourts
            : configResult?.courts || [];
        availabilityResult.value = {
          sport: activeSport.value,
          courts: finalCourts,
        };
        queriedRange.value = {
          sportId: activeSport.value,
          start: selectedStart.value,
          end: endExclusive,
        };
        success.value = "查询成功，以下为筛选结果";
      } catch (err) {
        console.error("查询可用场地失败", err);
        error.value = err?.message || "查询可用场地失败";
        queriedRange.value = null;
        availabilityResult.value = null;
      } finally {
        loading.value = false;
      }
    };

    const slotClass = (slot) => {
      const classes = [];
      if (slot.boundaryOnly) classes.push("boundary");
      if (slot.disabled) classes.push("disabled");
      if (slot.unavailable) classes.push("unavailable");
      if (selectedSlotsSet.value.has(slot.start)) classes.push("selected");
      if (selectedEnd.value !== null && slot.start === selectedEnd.value) {
        classes.push("selected-end");
      }
      return classes.join(" ");
    };

    watch(currentDateParam, () => {
      availabilityResult.value = null;
      fetchOverview();
    });

    onMounted(() => {
      fetchOverview();
    });

    return {
      sportList,
      activeSport,
      activeSportConfig,
      formattedCurrentDate,
      activeRules,
      timeSlotOptions,
      rangePreview,
      changeSport,
      changeDate,
      handleSlotClick,
      slotClass,
      submitQuery,
      isQueryDisabled,
      minimumDurationText,
      loading,
      error,
      success,
      displayCourts,
      availabilityResult,
      queriedRange,
      resultRangeText,
    };
  },
};

function buildBaseCourts(meta) {
  const list = [];
  if (meta.courtLabels && meta.courtLabels.length) {
    meta.courtLabels.forEach((label, index) => {
      list.push({
        id: `${meta.id}-${index + 1}`,
        displayName: label,
        backendKey: index + 1,
      });
    });
  } else {
    for (let i = 1; i <= meta.totalCourts; i += 1) {
      list.push({
        id: `${meta.id}-${i}`,
        displayName: `${meta.label}${i}号场`,
        backendKey: i,
      });
    }
  }
  return list;
}
</script>

<style scoped>
.venue-query-page {
  max-width: 520px;
  margin: 0 auto;
  padding: 18px 16px 40px;
  background-color: #f5f5f5;
  color: #1f2937;
  min-height: 100vh;
  box-sizing: border-box;
}

.page-header {
  text-align: center;
  border-bottom: 1px solid #e5e7eb;
  padding-bottom: 16px;
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 600;
}

.page-header p {
  margin-top: 6px;
  font-size: 13px;
  color: #6b7280;
}

.sport-tabs {
  display: flex;
  background: #fff;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 4px 16px rgba(15, 23, 42, 0.08);
  margin-bottom: 18px;
}

.sport-tab {
  flex: 1;
  border: none;
  padding: 12px 0;
  background: #fff;
  color: #374151;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.sport-tab.active {
  background: #07c160;
  color: #fff;
}

.date-selector {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #fff;
  padding: 12px 16px;
  border-radius: 12px;
  box-shadow: 0 4px 16px rgba(15, 23, 42, 0.08);
  margin-bottom: 18px;
}

.date-btn {
  border: none;
  background: none;
  font-size: 16px;
  color: #4b5563;
  cursor: pointer;
}

.current-date {
  font-weight: 600;
  color: #111827;
}

.rules-card {
  background: #e8f5e9;
  color: #276749;
  border-radius: 12px;
  padding: 14px 16px;
  margin-bottom: 20px;
  font-size: 13px;
  line-height: 1.6;
}

.rules-card ul {
  margin: 6px 0 0;
  padding-left: 18px;
}

.time-section {
  background: #fff;
  padding: 18px;
  border-radius: 16px;
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.08);
  margin-bottom: 20px;
}

.time-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.time-header h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.range-preview {
  font-size: 12px;
  color: #2563eb;
}

.time-slots {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

.time-slot {
  border: none;
  border-radius: 10px;
  padding: 10px 0;
  font-size: 14px;
  cursor: pointer;
  background: #f3f4f6;
  color: #1f2937;
  transition: transform 0.12s ease, background 0.12s ease;
}

.time-slot:hover:not(.disabled) {
  transform: translateY(-1px);
}

.time-slot.selected {
  background: #07c160;
  color: #fff;
}

.time-slot.boundary {
  border-style: dashed;
}

.time-slot.selected-end {
  border: 1px solid #07c160;
  color: #047857;
}

.time-slot.disabled {
  background: #e5e7eb;
  color: #9ca3af;
  cursor: not-allowed;
}

.time-slot.unavailable {
  background: rgba(248, 113, 113, 0.18);
  color: #b91c1c;
}

.time-hint {
  margin-top: 12px;
  font-size: 12px;
  color: #6b7280;
}

.info-area {
  min-height: 20px;
  margin-bottom: 18px;
}

.info {
  padding: 12px 14px;
  border-radius: 12px;
  font-size: 13px;
}

.info--loading {
  background: rgba(59, 130, 246, 0.12);
  color: #1d4ed8;
}

.info--error {
  background: rgba(248, 113, 113, 0.14);
  color: #b91c1c;
}

.info--success {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.results-card {
  background: #fff;
  padding: 20px;
  border-radius: 16px;
  box-shadow: 0 16px 32px rgba(15, 23, 42, 0.08);
  margin-bottom: 20px;
}

.results-title {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 18px;
}

.results-title h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.results-range {
  font-size: 12px;
  color: #0f766e;
}

.results-placeholder {
  text-align: center;
  color: #6b7280;
  font-size: 13px;
  padding: 24px 0;
}

.courts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 14px;
}

.court-card {
  border-radius: 14px;
  padding: 14px;
  min-height: 120px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  position: relative;
  overflow: hidden;
  color: #fff;
  box-shadow: inset 0 -60px 80px rgba(0, 0, 0, 0.25);
}

.court-card.unavailable {
  filter: grayscale(0.15);
  opacity: 0.9;
}

.court-number {
  font-size: 16px;
  font-weight: 600;
}

.court-status {
  align-self: flex-start;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  margin-top: auto;
}

.court-status.available {
  background: rgba(34, 197, 94, 0.85);
}

.court-status.booked {
  background: rgba(239, 68, 68, 0.9);
}

.court-note {
  margin-top: 6px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.85);
}

.badminton-court {
  background-image: linear-gradient(rgba(0, 0, 0, 0.45), rgba(0, 0, 0, 0.3)),
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect width="100" height="100" fill="%23e8f5e9"/><rect x="10" y="10" width="80" height="80" fill="none" stroke="%234caf50" stroke-width="2"/><line x1="50" y1="10" x2="50" y2="90" stroke="%234caf50" stroke-width="2"/><circle cx="50" cy="50" r="5" fill="%234caf50"/></svg>');
}

.basketball-court {
  background-image: linear-gradient(rgba(0, 0, 0, 0.45), rgba(0, 0, 0, 0.3)),
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect width="100" height="100" fill="%23ffecb3"/><rect x="10" y="10" width="80" height="80" fill="none" stroke="%23ff9800" stroke-width="2"/><circle cx="50" cy="50" r="20" fill="none" stroke="%23ff9800" stroke-width="2"/><line x1="10" y1="50" x2="90" y2="50" stroke="%23ff9800" stroke-width="2"/><circle cx="50" cy="50" r="5" fill="%23ff9800"/></svg>');
}

.football-court {
  background-image: linear-gradient(rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.35)),
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect width="100" height="100" fill="%23c8e6c9"/><rect x="10" y="10" width="80" height="80" fill="none" stroke="%232e7d32" stroke-width="2"/><circle cx="50" cy="50" r="15" fill="none" stroke="%232e7d32" stroke-width="2"/><line x1="50" y1="10" x2="50" y2="90" stroke="%232e7d32" stroke-width="2"/><rect x="10" y="35" width="10" height="30" fill="none" stroke="%232e7d32" stroke-width="2"/><rect x="80" y="35" width="10" height="30" fill="none" stroke="%232e7d32" stroke-width="2"/></svg>');
}

.query-btn {
  width: 100%;
  border: none;
  border-radius: 12px;
  padding: 14px 0;
  font-size: 16px;
  font-weight: 600;
  color: #fff;
  background: linear-gradient(135deg, #07c160, #0bb172);
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.query-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 420px) {
  .time-slots {
    grid-template-columns: repeat(3, 1fr);
  }
  .courts-grid {
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  }
}
</style>
