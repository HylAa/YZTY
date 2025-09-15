<template>
  <div class="my-courses-page">
    <van-nav-bar
      title="我的套餐"
      left-arrow
      fixed
      @click-left="$router.back()"
      :border="false"
      class="custom-nav"
    >
      <template #right>
        <van-icon name="question-o" size="20" />
        <span class="service-number">10086</span>
        <van-icon name="expand-o" size="20" class="ml-2" />
      </template>
    </van-nav-bar>

    <div class="content">
      <div class="user-info">
        <h2>136****3596 梁*</h2>
        <div class="user-tags">
          <van-tag plain color="#7e7e7e" class="mr-2">全球通银卡</van-tag>
          <van-tag plain color="#3b82f6">5星用户</van-tag>
        </div>
      </div>

      <div class="package-card">
        <div class="package-header">
          <div class="package-title">
            <span class="package-name">运动套餐 (个人版) 128元</span>
            <van-icon name="info-o" />
          </div>
          <div class="package-price">(128.00元)</div>
        </div>
        <div class="package-actions">
          <van-button plain size="small" round class="action-btn"
            >余量查询</van-button
          >
          <van-button plain size="small" round class="action-btn"
            >换套餐</van-button
          >
        </div>
      </div>

      <div class="tab-container">
        <van-tabs v-model="activeTab" animated swipeable sticky>
          <van-tab title="套餐资源">
            <div class="resource-list">
              <!-- 课时流量 -->
              <div class="resource-item">
                <div class="resource-title">
                  <div class="dot blue"></div>
                  <span>课时</span>
                </div>

                <div class="resource-detail">
                  <div class="resource-name">
                    <span>- 30节国内通用课时</span>
                    <span class="resource-total">总计：30节</span>
                  </div>
                </div>
              </div>

              <!-- 教练通话 -->
              <div class="resource-item">
                <div class="resource-title">
                  <div class="dot orange"></div>
                  <span>通话</span>
                </div>

                <div class="resource-detail">
                  <div class="resource-name">
                    <span>- 500分钟教练咨询</span>
                    <span class="resource-total">总计：500分钟</span>
                  </div>
                  <div class="resource-name">
                    <span>- 100分钟国际教练咨询</span>
                    <span class="resource-total">总计：100分钟</span>
                  </div>
                </div>
              </div>

              <!-- 额外功能 -->
              <div class="resource-item">
                <div class="resource-title">
                  <div class="dot red"></div>
                  <span>通信功能</span>
                </div>

                <div class="resource-detail">
                  <div class="resource-name">
                    <span>- 全国健身网络</span>
                    <span class="resource-total">0/18</span>
                  </div>
                </div>
              </div>
            </div>
          </van-tab>
          <van-tab title="融合资源">
            <div class="empty-content">
              <p>暂无融合资源数据</p>
            </div>
          </van-tab>
        </van-tabs>
      </div>
    </div>

    <div class="footer">
      <div class="service-btn">
        <van-icon name="service-o" size="24" />
        <span>在线客服</span>
      </div>
      <div class="action-tabs">
        <div class="action-tab">套餐专区</div>
        <div class="action-tab special">5折课时</div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref } from "vue";

export default {
  name: "MyCourses",
  setup() {
    const activeTab = ref(0);

    return {
      activeTab,
    };
  },
};
</script>

<style scoped>
.my-courses-page {
  background-color: #f5f7fa;
  min-height: 100vh;
  padding-top: 46px;
  padding-bottom: 60px;
}

.custom-nav {
  background-color: #f0f7ff !important;
}

:deep(.van-nav-bar__title) {
  color: #000;
  font-weight: 600;
  font-size: 18px;
}

.service-number {
  margin: 0 8px;
  font-size: 15px;
  font-weight: 500;
}

.content {
  padding: 15px;
}

.user-info {
  margin-bottom: 15px;
}

.user-info h2 {
  font-size: 20px;
  margin: 0 0 10px 0;
  font-weight: 500;
}

.user-tags {
  display: flex;
}

.package-card {
  background-color: #3b82f6;
  border-radius: 10px;
  padding: 15px;
  color: #fff;
  margin-bottom: 15px;
}

.package-header {
  display: flex;
  flex-direction: column;
  margin-bottom: 15px;
}

.package-title {
  display: flex;
  align-items: center;
}

.package-name {
  font-size: 18px;
  font-weight: 500;
  margin-right: 5px;
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
  color: #fff !important;
  border-color: #fff !important;
}

.tab-container {
  background: #fff;
  border-radius: 10px;
  overflow: hidden;
}

:deep(.van-tabs__wrap) {
  padding: 0 10px;
}

:deep(.van-tabs__line) {
  background-color: #3b82f6;
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

.resource-name {
  display: flex;
  justify-content: space-between;
  margin-bottom: 15px;
  color: #666;
}

.resource-total {
  font-weight: 500;
  color: #333;
}

.empty-content {
  padding: 30px 0;
  text-align: center;
  color: #999;
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
  width: 70px;
  border-right: 1px solid #eee;
  font-size: 12px;
  color: #666;
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

.action-tab.special {
  background-color: #1890ff;
}
</style>
