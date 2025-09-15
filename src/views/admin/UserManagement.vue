<template>
  <div class="user-management">
    <van-nav-bar title="用户管理" left-arrow @click-left="$router.go(-1)" />

    <div class="container">
      <!-- 搜索栏 -->
      <van-search
        v-model="searchQuery"
        placeholder="搜索用户姓名或手机号"
        @search="onSearch"
        show-action
      >
        <template #action>
          <div @click="onSearch">搜索</div>
        </template>
      </van-search>

      <!-- 加载中 -->
      <van-loading v-if="loading" size="24px" vertical class="loading"
        >加载中...</van-loading
      >

      <!-- 用户列表 -->
      <template v-else>
        <van-empty v-if="users.length === 0" description="没有找到用户" />

        <van-list v-else>
          <van-cell-group
            v-for="user in users"
            :key="user._id"
            :inset="true"
            class="user-card"
          >
            <van-cell :title="user.name" :value="`${user.role}`" />
            <van-cell title="手机号" :value="user.phone" />
            <van-cell title="会员等级" :value="user.memberLevel" />
            <van-cell title="注册时间" :value="formatDate(user.createdAt)" />

            <div class="user-actions">
              <van-button
                type="primary"
                size="small"
                @click="showEditDialog(user)"
                >编辑</van-button
              >
              <van-button
                type="danger"
                size="small"
                @click="showDeleteConfirm(user)"
                >删除</van-button
              >
            </div>
          </van-cell-group>
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

    <!-- 编辑用户弹窗 -->
    <van-dialog
      :show="showEdit"
      @update:show="showEdit = $event"
      title="编辑用户"
      show-cancel-button
      @confirm="updateUser"
    >
      <van-form>
        <van-cell-group inset>
          <van-field
            v-model="editForm.name"
            label="姓名"
            placeholder="请输入姓名"
            :rules="[{ required: true, message: '请输入姓名' }]"
          />

          <van-field
            v-model="editForm.phone"
            type="tel"
            label="手机号"
            placeholder="请输入手机号"
            :rules="[{ required: true, message: '请输入手机号' }]"
          />

          <van-field name="memberLevel" label="会员等级">
            <template #input>
              <van-dropdown-menu>
                <van-dropdown-item
                  :value="editForm.memberLevel"
                  @change="editForm.memberLevel = $event"
                  :options="memberLevelOptions"
                />
              </van-dropdown-menu>
            </template>
          </van-field>

          <van-field name="role" label="用户角色">
            <template #input>
              <van-dropdown-menu>
                <van-dropdown-item
                  :value="editForm.role"
                  @change="editForm.role = $event"
                  :options="roleOptions"
                />
              </van-dropdown-menu>
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
  name: "UserManagement",
  setup() {
    const store = useStore();
    const loading = ref(false);
    const searchQuery = ref("");
    const currentPage = ref(1);
    const showEdit = ref(false);

    // 会员等级选项
    const memberLevelOptions = [
      { text: "普通会员", value: "普通会员" },
      { text: "银卡会员", value: "银卡会员" },
      { text: "金卡会员", value: "金卡会员" },
      { text: "钻石会员", value: "钻石会员" },
      { text: "5星用户", value: "5星用户" },
    ];

    // 角色选项
    const roleOptions = [
      { text: "普通用户", value: "user" },
      { text: "教练", value: "coach" },
      { text: "管理员", value: "admin" },
    ];

    // 编辑表单
    const editForm = reactive({
      id: "",
      name: "",
      phone: "",
      memberLevel: "普通会员",
      role: "user",
    });

    // 获取用户列表
    const users = computed(() => store.getters["admin/users"]);
    const pagination = computed(() => store.getters["admin/pagination"].users);

    // 页面加载时获取用户
    onMounted(async () => {
      await fetchUsers();
    });

    // 获取用户数据
    const fetchUsers = async () => {
      try {
        loading.value = true;
        await store.dispatch("admin/getUsers", {
          page: currentPage.value,
          limit: 10,
          search: searchQuery.value,
        });
      } catch (error) {
        console.error("获取用户列表失败:", error);
        showToast("获取用户列表失败");
      } finally {
        loading.value = false;
      }
    };

    // 搜索
    const onSearch = () => {
      currentPage.value = 1;
      fetchUsers();
    };

    // 切换页面
    const onPageChange = (page) => {
      currentPage.value = page;
      fetchUsers();
    };

    // 显示编辑弹窗
    const showEditDialog = (user) => {
      editForm.id = user._id;
      editForm.name = user.name;
      editForm.phone = user.phone;
      editForm.memberLevel = user.memberLevel;
      editForm.role = user.role;
      showEdit.value = true;
    };

    // 更新用户
    const updateUser = async () => {
      try {
        loading.value = true;
        await store.dispatch("admin/updateUser", {
          id: editForm.id,
          userData: {
            name: editForm.name,
            phone: editForm.phone,
            memberLevel: editForm.memberLevel,
            role: editForm.role,
          },
        });

        showToast("用户更新成功");
      } catch (error) {
        console.error("更新用户失败:", error);
        showToast("更新用户失败");
      } finally {
        loading.value = false;
      }
    };

    // 显示删除确认
    const showDeleteConfirm = (user) => {
      showConfirmDialog({
        title: "删除用户",
        message: `确定要删除用户 ${user.name} 吗？此操作不可恢复。`,
      })
        .then(async () => {
          try {
            loading.value = true;
            await store.dispatch("admin/deleteUser", user._id);
            showToast("用户删除成功");
          } catch (error) {
            console.error("删除用户失败:", error);
            showToast("删除用户失败");
          } finally {
            loading.value = false;
          }
        })
        .catch(() => {
          // 取消删除
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
      loading,
      users,
      searchQuery,
      currentPage,
      pagination,
      showEdit,
      editForm,
      memberLevelOptions,
      roleOptions,
      onSearch,
      onPageChange,
      showEditDialog,
      updateUser,
      showDeleteConfirm,
      formatDate,
    };
  },
};
</script>

<style scoped>
.user-management {
  padding-bottom: 50px;
}

.container {
  padding-bottom: 20px;
}

.loading {
  margin: 20px auto;
  text-align: center;
}

.user-card {
  margin-bottom: 15px;
}

.user-actions {
  display: flex;
  justify-content: space-around;
  padding: 10px;
}

.user-actions .van-button {
  margin: 0 5px;
}

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}
</style>
