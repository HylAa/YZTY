/**
 * 微信相关工具函数
 */
import axios from "axios";

// 声明全局wx对象，来自微信JSSDK
/* global wx */

const wxUtils = {
  /**
   * 初始化微信JS-SDK配置
   * @param {Object} config 配置信息，从后端获取
   * @returns {Promise}
   */
  initJssdkConfig(config) {
    return new Promise((resolve, reject) => {
      wx.config({
        debug: process.env.NODE_ENV === "development", // 开发环境开启debug模式
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
    return axios.post("/api/wechat/jssdkConfig", { url }).then((res) => {
      if (res.data.code === 0) {
        return res.data.data;
      }
      throw new Error(res.data.message || "获取微信配置失败");
    });
  },

  /**
   * 微信授权登录，引导用户跳转到微信授权页
   * @param {String} redirectUrl 授权后的回调地址
   */
  oauthLogin(redirectUrl) {
    // 构建授权链接，redirectUrl需要urlencode
    const encodedUrl = encodeURIComponent(redirectUrl || window.location.href);
    const appId = process.env.VUE_APP_WECHAT_APPID; // 从环境变量获取

    // 构建授权URL，scope=snsapi_userinfo 表示获取用户基本信息
    const authUrl = `https://open.weixin.qq.com/connect/oauth2/authorize?appid=${appId}&redirect_uri=${encodedUrl}&response_type=code&scope=snsapi_userinfo&state=STATE#wechat_redirect`;

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
    return axios.post("/api/wechat/getUserInfo", { code }).then((res) => {
      if (res.data.code === 0) {
        return res.data.data;
      }
      throw new Error(res.data.message || "获取用户信息失败");
    });
  },

  /**
   * 获取微信手机号
   * 注意：这需要用户已关注公众号，且公众号为认证服务号
   * @returns {Promise}
   */
  getPhoneNumber() {
    return new Promise((resolve, reject) => {
      // 此处需要企业微信或认证服务号才能调用
      wx.invoke("getPhoneNumber", {}, function (res) {
        if (res.err_msg === "getPhoneNumber:ok") {
          // 提交到后台解密
          axios
            .post("/api/wechat/decryptPhoneNumber", {
              encryptedData: res.encryptedData,
              iv: res.iv,
            })
            .then((res) => {
              if (res.data.code === 0) {
                resolve(res.data.data.phoneNumber);
              } else {
                reject(new Error(res.data.message || "获取手机号失败"));
              }
            })
            .catch(reject);
        } else {
          reject(new Error(res.err_msg || "获取手机号失败"));
        }
      });
    });
  },
};

export default wxUtils;
