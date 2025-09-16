/**
 * 微信相关工具函数
 */
import api from "../api";

// 声明全局wx对象，来自微信JSSDK
/* global wx */

const wxUtils = {
  /** 判断是否在微信内 */
  isWechat() {
    return /MicroMessenger/i.test(navigator.userAgent || "");
  },

  /**
   * 初始化微信JS-SDK配置
   * @param {Object} config 配置信息，从后端获取
   * @returns {Promise}
   */
  initJssdkConfig(config) {
    return new Promise((resolve, reject) => {
      wx.config({
        debug: import.meta.env.DEV, // Vite 环境：开发模式开启 debug
        appId: config.appId,
        timestamp: config.timestamp,
        nonceStr: config.nonceStr,
        signature: config.signature,
        jsApiList: [
          "updateAppMessageShareData",
          "updateTimelineShareData",
          "getLocation",
          "openLocation",
          "scanQRCode",
          "chooseImage",
          "getLocalImgData",
          "uploadImage",
          "downloadImage",
          "chooseWXPay",
          "getPhoneNumber",
        ],
      });

      wx.ready(() => {
        console.log("微信JS-SDK初始化成功");
        resolve();
      });

      wx.error((err) => {
        console.error("微信JS-SDK初始化失败", err);
        reject(err);
      });
    });
  },

  /**
   * 获取微信JS-SDK配置
   * @param {String} url 当前页面URL
   * @returns {Promise}
   */
  getJssdkConfig(url) {
    // 若非微信内，直接跳过并返回空配置，避免本地开发 403
    if (!this.isWechat()) {
      return Promise.resolve({ skip: true });
    }
    return api.wechat.getJssdkConfig(url).then((res) => {
      if (res.code === 0) {
        return res.data;
      }
      throw new Error(res.message || "获取微信配置失败");
    });
  },

  /**
   * 微信授权登录，引导用户跳转到微信授权页
   * @param {String} redirectUrl 授权后的回调地址
   */
  oauthLogin(redirectUrl) {
    // 从 Vite 环境变量获取 AppID 与重定向域名
    const appId = import.meta.env.VITE_WECHAT_APPID;
    const origin = import.meta.env.VITE_WECHAT_REDIRECT_ORIGIN || window.location.origin;
    if (!appId) {
      console.error("VITE_WECHAT_APPID 未配置，无法发起微信授权");
      alert("微信 AppID 未配置，请联系管理员");
      return;
    }
    // 使用配置域名拼接当前路径，去掉 hash 片段
    const pathAndQuery = window.location.pathname + window.location.search;
    const finalRedirect = (redirectUrl || origin + pathAndQuery).split("#")[0];
    const encodedUrl = encodeURIComponent(finalRedirect);
    const scope = "snsapi_userinfo"; // 或根据需求使用 snsapi_base

    const authUrl = `https://open.weixin.qq.com/connect/oauth2/authorize?appid=${appId}&redirect_uri=${encodedUrl}&response_type=code&scope=${scope}&state=STATE#wechat_redirect`;

    // 跳转到微信授权页
    window.location.href = authUrl;
  },

  /**
   * 从URL中获取微信授权code
   * @returns {String|null} 授权code
   */
  getAuthCodeFromUrl() {
    const url = window.location.search;
    const match = url.match(/[?&]code=([^&]+)/);
    return match ? match[1] : null;
  },

  /**
   * 通过授权code获取用户信息
   * @param {String} code 授权code
   * @returns {Promise}
   */
  getUserInfoByCode(code) {
    return api.wechat.getUserInfoByCode(code).then((res) => {
      if (res.code === 0) {
        return res.data;
      }
      throw new Error(res.message || "获取用户信息失败");
    });
  },

  /**
   * 获取微信手机号（仅小程序支持）
   * 注意：公众号网页无法直接获取手机号，需要用户手动输入
   * 此方法仅在小程序环境下有效
   * @param {Object} e - 小程序按钮事件对象
   * @returns {Promise}
   */
  getPhoneNumber(e) {
    return new Promise((resolve, reject) => {
      // 检查是否在小程序环境
      if (!window.__wxjs_environment || window.__wxjs_environment !== 'miniprogram') {
        reject(new Error("获取手机号仅在小程序环境支持，公众号网页请使用手动绑定"));
        return;
      }

      // 小程序环境：从事件对象中获取加密数据
      if (e && e.detail && e.detail.encryptedData) {
        // 提交到后台解密
        api.wechat
          .decryptPhoneNumber({
            encryptedData: e.detail.encryptedData,
            iv: e.detail.iv,
            // 需要提供 code 或 session_key
            code: e.detail.code,
          })
          .then((res) => {
            if (res.code === 0) {
              resolve(res.data.phoneNumber);
            } else {
              reject(new Error(res.message || "获取手机号失败"));
            }
          })
          .catch(reject);
      } else {
        reject(new Error("未获取到手机号授权"));
      }
    });
  },
};

export default wxUtils;
