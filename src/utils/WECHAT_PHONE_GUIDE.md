# 微信获取手机号功能说明

## 环境差异

### 1. 微信公众号网页（当前环境）

**不支持直接获取手机号**，原因：
- 微信JSSDK没有提供获取手机号的API
- `wx.invoke("getPhoneNumber")` 不存在
- 需要用户手动输入手机号

**解决方案**：
```javascript
// 显示手机号绑定弹窗，让用户手动输入
showBindPhone.value = true;
```

### 2. 微信小程序

**支持获取手机号**，但需要：
- 使用特定的 button 组件
- 设置 `open-type="getPhoneNumber"`
- 用户主动授权

**实现方式**：
```html
<!-- 小程序 wxml -->
<button open-type="getPhoneNumber" @getphonenumber="onGetPhoneNumber">
  获取手机号
</button>
```

```javascript
// 小程序 js
onGetPhoneNumber(e) {
  if (e.detail.encryptedData) {
    // 发送到后端解密
    wx.request({
      url: '/wechat/decryptPhoneNumber',
      data: {
        encryptedData: e.detail.encryptedData,
        iv: e.detail.iv,
        code: e.detail.code
      }
    })
  }
}
```

## 后端解密流程

1. **获取 session_key**
   - 通过 `wx.login()` 获取 code
   - 调用微信接口换取 session_key

2. **解密手机号**
   - 使用 AES-128-CBC 算法
   - 密钥：session_key（Base64解码）
   - IV：前端传来的 iv（Base64解码）

3. **返回手机号**
   ```json
   {
     "phoneNumber": "13800138000",
     "purePhoneNumber": "13800138000",
     "countryCode": "86"
   }
   ```

## 当前项目实现

1. **前端（Home.vue）**：
   - 点击"获取手机号"按钮
   - 显示手机号绑定弹窗
   - 用户手动输入并验证

2. **组件（BindPhoneDialog.vue）**：
   - 手机号输入框
   - 验证码发送
   - 绑定确认

3. **后端（已实现）**：
   - `/wechat/decryptPhoneNumber` 接口
   - 支持小程序加密数据解密
   - 但公众号网页用不到此接口

## 注意事项

1. **公众号网页** 只能通过用户手动输入获取手机号
2. **小程序** 可以直接获取，但需要用户授权
3. **企业微信** 有自己的API体系
4. 不同环境需要不同的实现策略

## 推荐做法

对于公众号网页项目：
1. 使用手机号+验证码方式
2. 提供良好的用户体验（自动聚焦、格式化等）
3. 做好手机号格式验证
4. 考虑缓存用户信息减少重复输入