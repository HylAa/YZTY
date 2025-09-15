<template>
  <div class="package-detail-page">
    <div class="nav-bar">
      <div class="nav-left" />
      <div class="nav-title">套餐详情</div>
      <div class="nav-right">
        <van-icon name="question-o" size="20" @click="goToCustomerService" />
        <span class="service-number" @click="goToCustomerService">客服</span>
      </div>
    </div>

    <div class="package-header">
      <div class="package-title">篮球/羽毛球套餐</div>
      <div class="package-price">1280.00元/月</div>
      <div class="package-valid">生效时间：2025-06-01</div>
    </div>

    <div class="resource-card">
      <div class="card-header">
        <div class="header-title">套餐课程</div>
        <div class="header-action">
          <span class="action-text">查看记录</span>
          <van-icon name="arrow" />
        </div>
      </div>

      <div class="resource-list">
        <!-- 篮球课程 -->
        <div class="resource-item">
          <div class="resource-left">
            <div class="resource-icon blue">
              <van-icon name="medal-o" size="24" color="#fff" />
            </div>
            <div class="resource-info">
              <div class="resource-name">专业篮球课程</div>
              <div class="resource-progress-text">
                <span class="resource-used">已上 10节</span>
                <span class="resource-total">总量 30节</span>
              </div>
              <div class="resource-progress">
                <div class="progress-bar" style="width: 33%" />
              </div>
            </div>
          </div>
          <div class="resource-actions">
            <van-button plain type="primary" size="small" round>
              预约
            </van-button>
          </div>
        </div>

        <!-- 足球基础训练 -->
        <div class="resource-item">
          <div class="resource-left">
            <div class="resource-icon orange">
              <van-icon name="medal-o" size="24" color="#fff" />
            </div>
            <div class="resource-info">
              <div class="resource-name">足球基础训练</div>
              <div class="resource-progress-text">
                <span class="resource-used">已上 5节</span>
                <span class="resource-total">总量 20节</span>
              </div>
              <div class="resource-progress">
                <div class="progress-bar" style="width: 25%" />
              </div>
            </div>
          </div>
          <div class="resource-actions">
            <van-button plain type="primary" size="small" round>
              预约
            </van-button>
          </div>
        </div>

        <!-- 足球战术训练 -->
        <div class="resource-item">
          <div class="resource-left">
            <div class="resource-icon green">
              <van-icon name="medal-o" size="24" color="#fff" />
            </div>
            <div class="resource-info">
              <div class="resource-name">足球战术训练</div>
              <div class="resource-progress-text">
                <span class="resource-used">已上 2节</span>
                <span class="resource-total">总量 10节</span>
              </div>
              <div class="resource-progress">
                <div class="progress-bar" style="width: 20%" />
              </div>
            </div>
          </div>
          <div class="resource-actions">
            <van-button plain type="primary" size="small" round>
              预约
            </van-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 羽毛球课程 -->
    <div class="resource-card">
      <div class="card-header">
        <div class="header-title">已完成课程</div>
        <div class="header-action">
          <span class="action-text">查看全部</span>
          <van-icon name="arrow" />
        </div>
      </div>

      <div class="course-list">
        <div class="course-item">
          <div class="course-left">
            <div class="course-icon red">
              <van-icon name="certificate" size="24" color="#fff" />
            </div>
            <div class="course-info">
              <div class="course-name">羽毛球初级课程</div>
              <div class="course-progress-text">
                <span class="course-used">已完成</span>
                <span class="course-total">总量 18节</span>
              </div>
              <div class="course-progress">
                <div class="progress-bar" style="width: 100%" />
              </div>
            </div>
          </div>
          <div class="course-actions">
            <van-button type="primary" size="small" round> 评价 </van-button>
          </div>
        </div>

        <div class="course-item">
          <div class="course-left">
            <div class="course-icon pink">
              <van-icon name="certificate" size="24" color="#fff" />
            </div>
            <div class="course-info">
              <div class="course-name">瑜伽基础训练班</div>
              <div class="course-progress-text">
                <span class="course-used">已完成</span>
                <span class="course-total">总量 15节</span>
              </div>
              <div class="course-progress">
                <div class="progress-bar" style="width: 100%" />
              </div>
            </div>
          </div>
          <div class="course-actions">
            <van-button type="primary" size="small" round> 评价 </van-button>
          </div>
        </div>
      </div>
    </div>

    <div class="footer-buttons">
      <van-button block type="primary" @click="$router.push('/')">
        返回课程首页
      </van-button>
      <!-- <van-button block> 续费套餐 </van-button> -->
    </div>
  </div>
</template>

<script>
export default {
  name: "PackageDetail",
  props: {
    id: {
      type: String,
      required: false,
    },
  },
  setup() {
    // 跳转到企业微信客服页面
    const goToCustomerService = () => {
      window.location.href =
        "https://work.weixin.qq.com/kfid/kfc8cb013cc4c466389";
    };

    // 套餐详情数据可以在这里添加
    return {
      goToCustomerService,
    };
  },
};
</script>

<style scoped>
.package-detail-page {
  min-height: 100vh;
  background-color: #f5f7fa;
  position: relative;
  padding-bottom: 100px;
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

.package-header {
  background-color: #3b82f6;
  color: white;
  padding: 20px 15px;
}

.package-title {
  font-size: 18px;
  font-weight: 500;
  margin-bottom: 10px;
}

.package-price {
  font-size: 24px;
  font-weight: bold;
  margin-bottom: 5px;
}

.package-valid {
  font-size: 14px;
  opacity: 0.8;
}

.resource-card {
  background-color: white;
  border-radius: 10px;
  margin: 15px;
  overflow: hidden;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  border-bottom: 1px solid #eee;
}

.header-title {
  font-size: 16px;
  font-weight: 500;
}

.header-action {
  display: flex;
  align-items: center;
  color: #666;
  font-size: 14px;
}

.action-text {
  margin-right: 5px;
}

.resource-list,
.course-list {
  padding: 0 15px;
}

.resource-item,
.course-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 0;
  border-bottom: 1px solid #f5f5f5;
}

.resource-item:last-child,
.course-item:last-child {
  border-bottom: none;
}

.resource-left,
.course-left {
  display: flex;
  flex: 1;
}

.resource-icon,
.course-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background-color: #3b82f6;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 10px;
}

.orange {
  background-color: #f59e0b;
}

.green {
  background-color: #10b981;
}

.red {
  background-color: #ef4444;
}

.pink {
  background-color: #ec4899;
}

.resource-info,
.course-info {
  flex: 1;
}

.resource-name,
.course-name {
  font-size: 15px;
  font-weight: 500;
  margin-bottom: 8px;
}

.resource-progress-text,
.course-progress-text {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #666;
  margin-bottom: 5px;
}

.resource-progress,
.course-progress {
  height: 4px;
  background-color: #eee;
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background-color: #3b82f6;
  border-radius: 2px;
}

.resource-actions,
.course-actions {
  margin-left: 10px;
}

.footer-buttons {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  padding: 15px;
  background-color: white;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.05);
}
</style>
