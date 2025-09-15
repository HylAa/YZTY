<template>
  <div class="package-page">
    <div class="nav-bar">
      <div class="nav-left"></div>
      <div class="nav-title">我的课程套餐</div>
      <div class="nav-right">
        <van-icon name="question-o" size="20" @click="goToCustomerService" />
        <span class="service-number" @click="goToCustomerService">客服</span>
        <!-- <van-icon name="expand-o" size="20" /> -->
      </div>
    </div>

    <div class="user-info">
      <template v-if="userInfo">
        <div class="user-avatar" v-if="userInfo.headimgurl">
          <img :src="userInfo.headimgurl" alt="头像" />
        </div>
        <h2 class="user-id">
          {{ userInfo.nickname || "用户" }}
          <span v-if="userInfo.phoneNumber">{{
            formatPhone(userInfo.phoneNumber)
          }}</span>
        </h2>
      </template>
      <template v-else>
        <h2 class="user-id">138****8999 张*</h2>
      </template>
      <div class="user-tags">
        <span class="tag card-tag">健身爱好者</span>
        <span class="tag star-tag">普通会员</span>
      </div>

      <!-- 微信授权/手机号 获取按钮 -->
      <div class="wx-actions">
        <van-button size="small" type="primary" round @click="handleGetWxUser">
          获取微信用户信息
        </van-button>
        <van-button
          size="small"
          type="success"
          round
          class="ml-8"
          @click="handleGetPhone"
        >
          获取手机号
        </van-button>
        <BindPhoneDialog v-model:show="showBindPhone" @bind-success="onBindPhone" />
        <van-button
          size="small"
          type="warning"
          round
          class="ml-8"
          @click="handleFindCoursesByPhone"
        >
          按手机号查课程
        </van-button>
      </div>
    </div>

    <div class="package-card" @click="goToPackageDetail">
      <div class="package-header">
        <div class="package-title">
          <span class="package-name">篮球/羽毛球套餐 1280元</span>
          <van-icon name="info-o" size="16" color="#fff" />
        </div>
        <div class="package-price">(1280.00元)</div>
      </div>
      <div class="package-actions">
        <button class="action-btn">剩余课时</button>
        <button class="action-btn">套餐详情</button>
      </div>
    </div>

    <div class="tab-container">
      <div class="tabs">
        <div class="tab active">套餐课程</div>
        <!-- <div class="tab">增值服务</div> -->
      </div>

      <div class="resource-list">
        <!-- 篮球 -->
        <div class="resource-item">
          <div class="resource-title">
            <div class="dot blue"></div>
            <span>篮球</span>
          </div>

          <div class="resource-detail">
            <div class="resource-subtitle">基础技能训练</div>
            <div class="resource-row">
              <div class="resource-name">
                <span>- 专业篮球课程</span>
              </div>
              <div class="resource-total">总计：30节</div>
            </div>
          </div>
        </div>

        <!-- 足球 -->
        <div class="resource-item">
          <div class="resource-title">
            <div class="dot orange"></div>
            <span>足球</span>
          </div>

          <div class="resource-detail">
            <div class="resource-row">
              <div class="resource-name">
                <span>- 足球基础训练</span>
              </div>
              <div class="resource-total">总计：20节</div>
            </div>
            <div class="resource-row">
              <div class="resource-name">
                <span>- 足球战术训练</span>
              </div>
              <div class="resource-total">总计：10节</div>
            </div>
          </div>
        </div>

        <!-- 羽毛球 -->
        <div class="resource-item">
          <div class="resource-title">
            <div class="dot red"></div>
            <span>羽毛球</span>
          </div>

          <div class="resource-detail">
            <div class="resource-row">
              <div class="resource-name">
                <span>- 羽毛球初级课程</span>
              </div>
              <div class="resource-total">18/18节</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="footer">
      <div class="service-btn" @click="goToCustomerService">
        <div class="service-icon">客</div>
        <span>在线客服</span>
      </div>
      <div class="action-tabs">
        <div class="action-tab">课程专区</div>
        <!-- <div class="action-tab special">优惠套餐</div> -->
      </div>
    </div>

    <!-- 授权弹窗 -->
    <van-dialog
      v-model="showAuthDialog"
      title="授权提示"
      confirm-button-text="确认授权"
      @confirm="handleAuthConfirm"
    >
      <p class="auth-tip">
        请授权微信获取您的头像和昵称信息，以便提供更好的服务体验
      </p>
    </van-dialog>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import BindPhoneDialog from "../components/BindPhoneDialog.vue";
import { useRouter } from "vue-router";
import { showToast, showLoadingToast, closeToast } from "vant";


import wxUtils from "../utils/wxUtils";

export default {
  name: "Home",
  setup() {
    const router = useRouter();
    const userInfo = ref(null);
    const showAuthDialog = ref(false);
    const showBindPhone = ref(false);

    // 跳转到套餐详情页
    const goToPackageDetail = () => {
      router.push("/package/1");
    };

    // 格式化手机号，隐藏中间四位
    const formatPhone = (phone) => {
      if (!phone || phone.length !== 11) return phone;
      return phone.replace(/(\d{3})(\d{4})(\d{4})/, "$1****$3");
    };

    // 检查微信授权并获取用户信息
    const checkWechatAuth = async () => {
      try {
        // 获取URL中的code参数
        const code = wxUtils.getAuthCodeFromUrl();

        if (code) {
          // 如果URL中有code，说明用户已授权，获取用户信息
          showLoadingToast({ message: "获取用户信息...", forbidClick: true });

          const userInfoData = await wxUtils.getUserInfoByCode(code);

          // 尝试获取用户手机号
          try {
            const phoneNumber = await wxUtils.getPhoneNumber();
            userInfoData.phoneNumber = phoneNumber;
          } catch (error) {
            console.log("获取手机号失败", error);
            // 获取手机号失败不影响主流程
          }

          userInfo.value = userInfoData;
          closeToast();

          // 将用户信息存储到本地
          localStorage.setItem("userInfo", JSON.stringify(userInfoData));
        } else {
          // 尝试从本地存储获取用户信息
          const storedUserInfo = localStorage.getItem("userInfo");

          if (storedUserInfo) {
            userInfo.value = JSON.parse(storedUserInfo);
          } else {
            // 显示授权提示弹窗
            showAuthDialog.value = true;
          }
        }
      } catch (error) {
        console.error("微信授权失败", error);
        showToast("获取用户信息失败");
      }
    };

    // 处理授权确认
    const handleAuthConfirm = () => {
      // 跳转到微信授权页面
      wxUtils.oauthLogin();
    };

    // 主动触发：拉取微信用户信息
    const handleGetWxUser = async () => {
      try {
        if (userInfo.value && userInfo.value.openid) {
          showToast("已获取用户信息");
          return;
        }
        const code = wxUtils.getAuthCodeFromUrl();
        if (!code) {
          // 弹窗提示或直接跳转授权
          showAuthDialog.value = true;
          return;
        }
        showLoadingToast({ message: "获取用户信息...", forbidClick: true });
        const info = await wxUtils.getUserInfoByCode(code);
        userInfo.value = info;
        localStorage.setItem("userInfo", JSON.stringify(info));
        closeshowToast();
        showToast("获取成功");
      } catch (e) {
        closeshowToast();
        showToast(e.message || "获取失败");
      }
    };

    // 主动触发：获取手机号
    const handleGetPhone = async () => {
      try {
        if (!userInfo.value) {
          showToast("请先获取微信用户信息");
          return;
        }
        showBindPhone.value = true; // H5 替代：弹出绑定手机号对话框
        const phone = await wxUtils.getPhoneNumber();
        userInfo.value.phoneNumber = phone;
        localStorage.setItem("userInfo", JSON.stringify(userInfo.value));
        closeshowToast();
        showToast("已获取手机号");
      } catch (e) {
        closeshowToast();
        showToast(e.message || "获取手机号失败");
      }
    };

    // 根据手机号查询课程
    const handleFindCoursesByPhone = () => {
      if (!userInfo.value || !userInfo.value.phoneNumber) {
        showToast("请先获取手机号");
        return;
      }
      router.push({
        path: "/courses",
        query: { phone: userInfo.value.phoneNumber },
      });
    };

    // 初始化微信JS-SDK
    const initWxConfig = async () => {
      try {
        // 获取当前页面URL
        const url = window.location.href.split("#")[0];

        // 从后端获取微信JS-SDK配置
        const config = await wxUtils.getJssdkConfig(url);

        // 初始化微信JS-SDK
        await wxUtils.initJssdkConfig(config);
      } catch (error) {
        console.error("初始化微信JS-SDK失败", error);
      }
    };

    // 跳转到企业微信客服页面
    const goToCustomerService = () => {
      // 跳转到企业微信客服页面
      window.location.href =
        "https://work.weixin.qq.com/kfid/kfc8cb013cc4c466389";
    };

    onMounted(() => {
      // 页面加载时初始化微信配置
      initWxConfig();

      // 检查微信授权
      checkWechatAuth();
    });

    const onBindPhone = ({ phoneNumber }) => {
      userInfo.value = userInfo.value || {};
      userInfo.value.phoneNumber = phoneNumber;
      localStorage.setItem("userInfo", JSON.stringify(userInfo.value));
      showToast("手机号绑定成功");
    };

    return {
      userInfo,
      showAuthDialog,
      showBindPhone,
      goToPackageDetail,
      formatPhone,
      handleAuthConfirm,
      handleGetWxUser,
      handleGetPhone,
      handleFindCoursesByPhone,
      goToCustomerService,
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

.user-info {
  padding: 15px;
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

.wx-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
}

.ml-8 { margin-left: 8px; }

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

.package-card {
  background-color: #3b82f6;
  border-radius: 10px;
  padding: 15px;
  margin: 0 15px 15px;
  color: white;
}

.package-header {
  margin-bottom: 15px;
}

.package-title {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-bottom: 5px;
}

.package-name {
  font-size: 18px;
  font-weight: 500;
}

.package-price {
  font-size: 15px;
  opacity: 0.9;
}

.package-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.action-btn {
  background: transparent;
  border: 1px solid white;
  color: white;
  padding: 5px 12px;
  border-radius: 15px;
  font-size: 14px;
}

.tab-container {
  background: white;
  border-radius: 10px;
  margin: 0 15px;
  overflow: hidden;
}

.tabs {
  display: flex;
  border-bottom: 1px solid #eee;
}

.tab {
  flex: 1;
  text-align: center;
  padding: 12px 0;
  font-size: 15px;
}

.tab.active {
  color: #3b82f6;
  font-weight: 500;
  position: relative;
}

.tab.active::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 40px;
  height: 3px;
  background-color: #3b82f6;
  border-radius: 3px;
}

.resource-list {
  padding: 15px;
}

.resource-item {
  margin-bottom: 25px;
}

.resource-title {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
  font-size: 16px;
  font-weight: 500;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  margin-right: 8px;
}

.blue {
  background-color: #3b82f6;
}

.orange {
  background-color: #f59e0b;
}

.red {
  background-color: #ef4444;
}

.resource-detail {
  padding-left: 18px;
}

.resource-subtitle {
  margin-bottom: 10px;
  color: #666;
}

.resource-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
  color: #666;
}

.resource-name {
  flex: 1;
}

.resource-total {
  font-weight: 500;
  color: #333;
  min-width: 120px;
  text-align: right;
}

.footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  background: white;
  border-top: 1px solid #eee;
  height: 60px;
}

.service-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 70px;
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
  color: white;
  font-weight: 500;
}

.action-tab.special {
  background-color: #1890ff;
}

.auth-tip {
  padding: 20px;
  text-align: center;
  color: #666;
}
</style>
