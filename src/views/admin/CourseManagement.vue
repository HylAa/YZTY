<template>
  <div class="course-management">
    <van-nav-bar title="课程管理" left-arrow @click-left="$router.go(-1)" />

    <div class="container">
      <!-- 搜索栏 -->
      <van-search
        v-model="searchQuery"
        placeholder="搜索课程名称"
        @search="onSearch"
        show-action
      >
        <template #action>
          <div @click="onSearch">搜索</div>
        </template>
      </van-search>

      <!-- 类型筛选 -->
      <div class="filter-section">
        <van-dropdown-menu>
          <van-dropdown-item
            :value="selectedType"
            @change="
              (value) => {
                selectedType = value;
                onSearch();
              }
            "
            :options="typeOptions"
          />
        </van-dropdown-menu>

        <van-button type="primary" @click="showAddDialog">添加课程</van-button>
      </div>

      <!-- 加载中 -->
      <van-loading v-if="loading" size="24px" vertical class="loading"
        >加载中...</van-loading
      >

      <!-- 课程列表 -->
      <template v-else>
        <van-empty v-if="courses.length === 0" description="没有找到课程" />

        <van-list v-else>
          <van-card
            v-for="course in courses"
            :key="course._id"
            :price="course.price"
            :title="course.name"
            :desc="course.description"
            :thumb="course.image"
            class="course-card"
          >
            <template #tags>
              <van-tag plain type="primary">{{ course.type }}</van-tag>
              <van-tag plain type="success" v-if="course.isFeatured"
                >精选</van-tag
              >
            </template>

            <template #footer>
              <div class="course-actions">
                <van-button
                  type="primary"
                  size="small"
                  @click="showEditDialog(course)"
                  >编辑</van-button
                >
                <van-button
                  type="danger"
                  size="small"
                  @click="showDeleteConfirm(course)"
                  >删除</van-button
                >
                <van-button
                  :type="course.isFeatured ? 'default' : 'warning'"
                  size="small"
                  @click="toggleFeatured(course)"
                >
                  {{ course.isFeatured ? "取消精选" : "设为精选" }}
                </van-button>
              </div>
            </template>
          </van-card>
        </van-list>

        <!-- 分页 -->
        <div class="pagination">
          <van-pagination
            :value="currentPage"
            @input="onPageChange"
            :total-items="pagination.total"
            :items-per-page="pagination.limit"
            :show-page-size="3"
            force-ellipses
          />
        </div>
      </template>
    </div>

    <!-- 编辑课程弹窗 -->
    <van-dialog
      :show="showEdit"
      @update:show="showEdit = $event"
      title="编辑课程"
      show-cancel-button
      @confirm="updateCourse"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="editForm.name"
            label="课程名称"
            placeholder="请输入课程名称"
            :rules="[{ required: true, message: '请输入课程名称' }]"
          />

          <van-field
            v-model="editForm.description"
            type="textarea"
            label="课程描述"
            placeholder="请输入课程描述"
            :rules="[{ required: true, message: '请输入课程描述' }]"
            rows="3"
          />

          <van-field name="type" label="课程类型">
            <template #input>
              <van-dropdown-menu>
                <van-dropdown-item
                  :value="editForm.type"
                  @change="editForm.type = $event"
                  :options="typeOptions"
                />
              </van-dropdown-menu>
            </template>
          </van-field>

          <van-field
            v-model.number="editForm.price"
            type="digit"
            label="价格"
            placeholder="请输入价格"
            :rules="[{ required: true, message: '请输入价格' }]"
          />

          <van-field
            v-model.number="editForm.duration"
            type="digit"
            label="时长(分钟)"
            placeholder="请输入课程时长"
            :rules="[{ required: true, message: '请输入课程时长' }]"
          />

          <van-field
            v-model.number="editForm.totalSessions"
            type="digit"
            label="总课时"
            placeholder="请输入总课时"
            :rules="[{ required: true, message: '请输入总课时' }]"
          />

          <van-field
            v-model="editForm.location.name"
            label="场地名称"
            placeholder="请输入场地名称"
            :rules="[{ required: true, message: '请输入场地名称' }]"
          />

          <van-field
            v-model="editForm.location.address"
            label="场地地址"
            placeholder="请输入场地地址"
            :rules="[{ required: true, message: '请输入场地地址' }]"
          />

          <van-field
            v-model="editForm.coach.name"
            label="教练姓名"
            placeholder="请输入教练姓名"
            :rules="[{ required: true, message: '请输入教练姓名' }]"
          />

          <van-field
            v-model="editForm.coach.introduction"
            type="textarea"
            label="教练介绍"
            placeholder="请输入教练介绍"
            :rules="[{ required: true, message: '请输入教练介绍' }]"
            rows="2"
          />

          <van-field name="isFeatured" label="是否精选">
            <template #input>
              <van-switch
                :value="editForm.isFeatured"
                @input="editForm.isFeatured = $event"
              />
            </template>
          </van-field>
        </van-cell-group>
      </van-form>
    </van-dialog>

    <!-- 添加课程弹窗 (与编辑课程弹窗相同，但标题不同) -->
    <van-dialog
      :show="showAdd"
      @update:show="showAdd = $event"
      title="添加课程"
      show-cancel-button
      @confirm="createCourse"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="addForm.name"
            label="课程名称"
            placeholder="请输入课程名称"
            :rules="[{ required: true, message: '请输入课程名称' }]"
          />

          <van-field
            v-model="addForm.description"
            type="textarea"
            label="课程描述"
            placeholder="请输入课程描述"
            :rules="[{ required: true, message: '请输入课程描述' }]"
            rows="3"
          />

          <van-field name="type" label="课程类型">
            <template #input>
              <van-dropdown-menu>
                <van-dropdown-item
                  :value="addForm.type"
                  @change="addForm.type = $event"
                  :options="typeOptions"
                />
              </van-dropdown-menu>
            </template>
          </van-field>

          <van-field
            v-model.number="addForm.price"
            type="digit"
            label="价格"
            placeholder="请输入价格"
            :rules="[{ required: true, message: '请输入价格' }]"
          />

          <van-field
            v-model.number="addForm.duration"
            type="digit"
            label="时长(分钟)"
            placeholder="请输入课程时长"
            :rules="[{ required: true, message: '请输入课程时长' }]"
          />

          <van-field
            v-model.number="addForm.totalSessions"
            type="digit"
            label="总课时"
            placeholder="请输入总课时"
            :rules="[{ required: true, message: '请输入总课时' }]"
          />

          <van-field
            v-model="addForm.location.name"
            label="场地名称"
            placeholder="请输入场地名称"
            :rules="[{ required: true, message: '请输入场地名称' }]"
          />

          <van-field
            v-model="addForm.location.address"
            label="场地地址"
            placeholder="请输入场地地址"
            :rules="[{ required: true, message: '请输入场地地址' }]"
          />

          <van-field
            v-model="addForm.coach.name"
            label="教练姓名"
            placeholder="请输入教练姓名"
            :rules="[{ required: true, message: '请输入教练姓名' }]"
          />

          <van-field
            v-model="addForm.coach.introduction"
            type="textarea"
            label="教练介绍"
            placeholder="请输入教练介绍"
            :rules="[{ required: true, message: '请输入教练介绍' }]"
            rows="2"
          />

          <van-field name="isFeatured" label="是否精选">
            <template #input>
              <van-switch
                :value="addForm.isFeatured"
                @input="addForm.isFeatured = $event"
              />
            </template>
          </van-field>
        </van-cell-group>
      </van-form>
    </van-dialog>
  </div>
</template>

<script>
import { computed, onMounted, reactive, ref } from "vue";
import { useStore } from "vuex";
import { showToast, showConfirmDialog } from "vant";

export default {
  name: "CourseManagement",
  setup() {
    const store = useStore();
    const loading = ref(false);
    const searchQuery = ref("");
    const selectedType = ref("");
    const currentPage = ref(1);
    const showEdit = ref(false);
    const showAdd = ref(false);

    // 课程类型选项
    const typeOptions = [
      { text: "全部类型", value: "" },
      { text: "健身", value: "健身" },
      { text: "游泳", value: "游泳" },
      { text: "瑜伽", value: "瑜伽" },
      { text: "篮球", value: "篮球" },
      { text: "足球", value: "足球" },
      { text: "羽毛球", value: "羽毛球" },
      { text: "乒乓球", value: "乒乓球" },
      { text: "其他", value: "其他" },
    ];

    // 编辑表单
    const editForm = reactive({
      id: "",
      name: "",
      description: "",
      type: "健身",
      price: 0,
      duration: 60,
      totalSessions: 10,
      location: {
        name: "",
        address: "",
        coordinates: {
          type: "Point",
          coordinates: [0, 0],
        },
      },
      coach: {
        name: "",
        introduction: "",
        avatar: "default-coach.jpg",
      },
      isFeatured: false,
    });

    // 添加表单
    const addForm = reactive({
      name: "",
      description: "",
      type: "健身",
      price: 0,
      duration: 60,
      totalSessions: 10,
      location: {
        name: "",
        address: "",
        coordinates: {
          type: "Point",
          coordinates: [0, 0],
        },
      },
      coach: {
        name: "",
        introduction: "",
        avatar: "default-coach.jpg",
      },
      isFeatured: false,
      image: "default-course.jpg",
      enrollmentCount: 0,
      rating: 0,
      schedule: [
        {
          dayOfWeek: "周一",
          startTime: "18:00",
          endTime: "19:00",
        },
      ],
    });

    // 获取课程列表
    const courses = computed(() => store.getters["admin/adminCourses"]);
    const pagination = computed(
      () => store.getters["admin/pagination"].courses
    );

    // 页面加载时获取课程
    onMounted(async () => {
      await fetchCourses();
    });

    // 获取课程数据
    const fetchCourses = async () => {
      try {
        loading.value = true;
        await store.dispatch("admin/getAdminCourses", {
          page: currentPage.value,
          limit: 10,
          search: searchQuery.value,
          type: selectedType.value,
        });
      } catch (error) {
        console.error("获取课程列表失败:", error);
        showToast("获取课程列表失败");
      } finally {
        loading.value = false;
      }
    };

    // 搜索
    const onSearch = () => {
      currentPage.value = 1;
      fetchCourses();
    };

    // 切换页面
    const onPageChange = (page) => {
      currentPage.value = page;
      fetchCourses();
    };

    // 显示编辑弹窗
    const showEditDialog = (course) => {
      editForm.id = course._id;
      editForm.name = course.name;
      editForm.description = course.description;
      editForm.type = course.type;
      editForm.price = course.price;
      editForm.duration = course.duration;
      editForm.totalSessions = course.totalSessions;
      editForm.location = { ...course.location };
      editForm.coach = { ...course.coach };
      editForm.isFeatured = course.isFeatured;

      showEdit.value = true;
    };

    // 显示添加弹窗
    const showAddDialog = () => {
      // 重置添加表单
      addForm.name = "";
      addForm.description = "";
      addForm.type = "健身";
      addForm.price = 0;
      addForm.duration = 60;
      addForm.totalSessions = 10;
      addForm.location.name = "";
      addForm.location.address = "";
      addForm.coach.name = "";
      addForm.coach.introduction = "";
      addForm.isFeatured = false;

      showAdd.value = true;
    };

    // 更新课程
    const updateCourse = async () => {
      try {
        loading.value = true;
        await store.dispatch("admin/updateCourse", {
          id: editForm.id,
          courseData: {
            name: editForm.name,
            description: editForm.description,
            type: editForm.type,
            price: editForm.price,
            duration: editForm.duration,
            totalSessions: editForm.totalSessions,
            location: editForm.location,
            coach: editForm.coach,
            isFeatured: editForm.isFeatured,
          },
        });

        showToast("课程更新成功");
      } catch (error) {
        console.error("更新课程失败:", error);
        showToast("更新课程失败");
      } finally {
        loading.value = false;
      }
    };

    // 创建课程
    const createCourse = async () => {
      try {
        loading.value = true;
        await store.dispatch("admin/createCourse", addForm);

        showToast("课程创建成功");
      } catch (error) {
        console.error("创建课程失败:", error);
        showToast("创建课程失败");
      } finally {
        loading.value = false;
      }
    };

    // 显示删除确认
    const showDeleteConfirm = (course) => {
      showConfirmDialog({
        title: "删除课程",
        message: `确定要删除课程 "${course.name}" 吗？此操作不可恢复。`,
      })
        .then(async () => {
          try {
            loading.value = true;
            await store.dispatch("admin/deleteCourse", course._id);
            showToast("课程删除成功");
          } catch (error) {
            console.error("删除课程失败:", error);
            showToast("删除课程失败");
          } finally {
            loading.value = false;
          }
        })
        .catch(() => {
          // 取消删除
        });
    };

    // 切换精选状态
    const toggleFeatured = async (course) => {
      try {
        loading.value = true;
        await store.dispatch("admin/updateCourse", {
          id: course._id,
          courseData: {
            ...course,
            isFeatured: !course.isFeatured,
          },
        });

        showToast(course.isFeatured ? "已取消精选" : "已设为精选");
      } catch (error) {
        console.error("更新课程失败:", error);
        showToast("操作失败");
      } finally {
        loading.value = false;
      }
    };

    return {
      loading,
      courses,
      searchQuery,
      selectedType,
      currentPage,
      pagination,
      showEdit,
      showAdd,
      editForm,
      addForm,
      typeOptions,
      onSearch,
      onPageChange,
      showEditDialog,
      showAddDialog,
      updateCourse,
      createCourse,
      showDeleteConfirm,
      toggleFeatured,
    };
  },
};
</script>

<style scoped>
.course-management {
  padding-bottom: 50px;
}

.container {
  padding-bottom: 20px;
}

.filter-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  margin-bottom: 16px;
}

.loading {
  margin: 20px auto;
  text-align: center;
}

.course-card {
  margin-bottom: 15px;
}

.course-actions {
  display: flex;
  justify-content: space-around;
  width: 100%;
  margin-top: 8px;
}

.course-actions .van-button {
  margin: 0 5px;
}

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}
</style>
