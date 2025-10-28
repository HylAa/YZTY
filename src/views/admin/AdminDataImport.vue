<template>
  <div class="admin-import-page">
    <header class="page-header">
      <div>
        <h1>数据导入</h1>
        <p class="subtitle">上传最新 Excel，覆盖 student_course_record 与 swim_customer_record</p>
      </div>
      <div class="header-actions">
        <button class="ghost-btn" type="button" @click="goBack">返回场馆管理</button>
      </div>
    </header>

    <section class="notice">
      <p>提示：导入会清空目标表并重新写入，请确保已有数据已备份。</p>
      <p>仅支持 Excel（.xlsx）文件，首个工作表需保持系统指定表头顺序。</p>
    </section>

    <section class="import-card">
      <header>
        <h2>学员课程记录（student_course_record）</h2>
        <p>表头：学员姓名、手机号身份、手机号、所在班级、课程名称、课程类型、购买数量... 学员创建人</p>
      </header>
      <div class="import-body">
        <div class="file-picker">
          <input ref="studentInputRef" type="file" accept=".xlsx" @change="handleStudentFile" />
          <span class="file-name">{{ studentFileName }}</span>
        </div>
        <button class="primary-btn" type="button" :disabled="studentLoading" @click="submitStudentImport">
          {{ studentLoading ? "导入中..." : "导入学员课程" }}
        </button>
      </div>
      <p v-if="studentError" class="error-text">{{ studentError }}</p>
      <p v-if="studentResult" class="success-text">
        成功覆盖 {{ studentResult.totalRows }} 条学员课程记录。
      </p>
      <p
        v-if="studentResult && studentResult.missingColumns && studentResult.missingColumns.length"
        class="warning-text"
      >
        以下字段未写入数据库：{{ studentResult.missingColumns.join("、") }}。请确认数据库中存在对应列。
      </p>
    </section>

    <section class="import-card">
      <header>
        <h2>游泳客户记录（swim_customer_record）</h2>
        <p>表头：门店、姓名、手机号、性别、出生日期、证件类型... 备注</p>
      </header>
      <div class="import-body">
        <div class="file-picker">
          <input ref="swimInputRef" type="file" accept=".xlsx" @change="handleSwimFile" />
          <span class="file-name">{{ swimFileName }}</span>
        </div>
        <button class="primary-btn" type="button" :disabled="swimLoading" @click="submitSwimImport">
          {{ swimLoading ? "导入中..." : "导入游泳客户" }}
        </button>
      </div>
      <p v-if="swimError" class="error-text">{{ swimError }}</p>
      <p v-if="swimResult" class="success-text">成功覆盖 {{ swimResult.totalRows }} 条游泳客户记录。</p>
      <p
        v-if="swimResult && swimResult.missingColumns && swimResult.missingColumns.length"
        class="warning-text"
      >
        以下字段未写入数据库：{{ swimResult.missingColumns.join("、") }}。请确认数据库中存在对应列。
      </p>
    </section>
  </div>
</template>

<script>
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import api from "../../api";

export default {
  name: "AdminDataImport",
  setup() {
    const router = useRouter();

    const studentFile = ref(null);
    const studentLoading = ref(false);
    const studentError = ref("");
    const studentResult = ref(null);
    const studentInputRef = ref(null);

    const swimFile = ref(null);
    const swimLoading = ref(false);
    const swimError = ref("");
    const swimResult = ref(null);
    const swimInputRef = ref(null);

    const getFileName = (fileRef) => {
      const file = fileRef.value;
      return file ? file.name : "未选择文件";
    };

    const studentFileName = computed(() => getFileName(studentFile));
    const swimFileName = computed(() => getFileName(swimFile));

    const resetInput = (inputRef) => {
      if (inputRef.value) {
        inputRef.value.value = "";
      }
    };

    const extractErrorMessage = (error) => {
      if (error?.response?.data?.message) {
        return error.response.data.message;
      }
      return error?.message || "导入失败，请稍后再试";
    };

    const handleStudentFile = (event) => {
      const [file] = event.target.files || [];
      studentFile.value = file || null;
      studentError.value = "";
    };

    const handleSwimFile = (event) => {
      const [file] = event.target.files || [];
      swimFile.value = file || null;
      swimError.value = "";
    };

    const submitStudentImport = async () => {
      if (!studentFile.value) {
        studentError.value = "请选择需要导入的 Excel 文件";
        return;
      }
      studentLoading.value = true;
      studentError.value = "";
      try {
        const formData = new FormData();
        formData.append("file", studentFile.value);
        const response = await api.adminImport.uploadStudentCourses(formData);
        if (!response || response.code !== 0) {
          throw new Error(response?.message || "导入失败");
        }
        const summary = response.data || {};
        studentResult.value = {
          totalRows: summary.total_rows ?? 0,
          missingColumns: summary.missing_columns || [],
        };
        studentFile.value = null;
        resetInput(studentInputRef);
      } catch (err) {
        studentError.value = extractErrorMessage(err);
        studentResult.value = null;
      } finally {
        studentLoading.value = false;
      }
    };

    const submitSwimImport = async () => {
      if (!swimFile.value) {
        swimError.value = "请选择需要导入的 Excel 文件";
        return;
      }
      swimLoading.value = true;
      swimError.value = "";
      try {
        const formData = new FormData();
        formData.append("file", swimFile.value);
        const response = await api.adminImport.uploadSwimCustomers(formData);
        if (!response || response.code !== 0) {
          throw new Error(response?.message || "导入失败");
        }
        const summary = response.data || {};
        swimResult.value = {
          totalRows: summary.total_rows ?? 0,
          missingColumns: summary.missing_columns || [],
        };
        swimFile.value = null;
        resetInput(swimInputRef);
      } catch (err) {
        swimError.value = extractErrorMessage(err);
        swimResult.value = null;
      } finally {
        swimLoading.value = false;
      }
    };

    const goBack = () => {
      router.push("/admin/venues");
    };

    return {
      studentInputRef,
      swimInputRef,
      studentFileName,
      swimFileName,
      studentLoading,
      swimLoading,
      studentError,
      swimError,
      studentResult,
      swimResult,
      handleStudentFile,
      handleSwimFile,
      submitStudentImport,
      submitSwimImport,
      goBack,
    };
  },
};
</script>

<style scoped>
.admin-import-page {
  min-height: 100vh;
  padding: 24px 20px 48px;
  background: linear-gradient(135deg, #f4f8ff, #f8fffb);
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.page-header h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #0f172a;
}

.subtitle {
  margin: 6px 0 0;
  font-size: 13px;
  color: #64748b;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.ghost-btn,
.primary-btn {
  padding: 10px 16px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.ghost-btn {
  border: 1px solid #cbd5f5;
  background: transparent;
  color: #3b82f6;
}

.ghost-btn:hover {
  background: rgba(59, 130, 246, 0.08);
}

.primary-btn {
  border: none;
  background: linear-gradient(135deg, #2563eb, #10b981);
  color: #fff;
  min-width: 150px;
}

.primary-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.notice {
  padding: 14px 16px;
  background: rgba(59, 130, 246, 0.08);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 12px;
  font-size: 13px;
  color: #1e293b;
  line-height: 1.6;
}

.import-card {
  background: #fff;
  border-radius: 16px;
  padding: 20px;
  box-shadow: 0 12px 28px rgba(15, 23, 42, 0.08);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.import-card > header h2 {
  margin: 0;
  font-size: 18px;
  color: #0f172a;
}

.import-card > header p {
  margin: 6px 0 0;
  font-size: 12px;
  color: #64748b;
}

.import-body {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}

.file-picker {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.file-picker input[type="file"] {
  padding: 8px;
  border-radius: 10px;
  border: 1px dashed #94a3b8;
  background: #f8fafc;
  cursor: pointer;
}

.file-name {
  font-size: 13px;
  color: #475569;
}

.error-text {
  margin: 0;
  font-size: 13px;
  color: #dc2626;
}

.success-text {
  margin: 0;
  font-size: 13px;
  color: #0f9d58;
}

.warning-text {
  margin: 4px 0 0;
  font-size: 12px;
  color: #d97706;
}

@media (max-width: 640px) {
  .admin-import-page {
    padding: 16px 14px 32px;
  }

  .page-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .notice {
    font-size: 12px;
  }

  .import-card > header p {
    font-size: 11px;
  }

  .file-picker {
    flex-direction: column;
    align-items: flex-start;
  }

  .file-name {
    font-size: 12px;
  }
}
</style>
