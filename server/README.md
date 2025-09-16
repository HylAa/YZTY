# 微信API集成服务器

## 功能说明

本服务器实现了微信公众号/小程序的真实API集成，包括：
- JSSDK 签名配置
- OAuth2.0 用户信息获取
- 手机号解密（小程序专用）

## 配置步骤

### 1. 获取微信凭证

登录[微信公众平台](https://mp.weixin.qq.com/)或[微信开放平台](https://open.weixin.qq.com/)：
- **AppID**: 在"开发 > 基本配置"中查看
- **AppSecret**: 在"开发 > 基本配置"中查看（需要管理员权限）

### 2. 配置环境变量

编辑 `.env` 文件：
```env
WECHAT_APPID=你的AppID
WECHAT_APPSECRET=你的32位AppSecret
PORT=8018

# MySQL 配置（可通过 DATABASE_URL 覆盖）
MYSQL_HOST=127.0.0.1
MYSQL_PORT=3306
MYSQL_USER=root
MYSQL_PASSWORD=123456
MYSQL_DATABASE=yzty
```

### 3. 初始化数据库

服务器启动时会自动连接 MySQL，请提前创建数据库与数据表：

```sql
CREATE DATABASE IF NOT EXISTS `yzty` DEFAULT CHARACTER SET utf8mb4;
USE `yzty`;

CREATE TABLE IF NOT EXISTS `wechat_users` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  `openid` VARCHAR(64) NOT NULL,
  `nickname` VARCHAR(128) NULL,
  `avatar` VARCHAR(512) NULL,
  `phone` VARCHAR(20) NULL,
  `created_at` DATETIME NOT NULL,
  `updated_at` DATETIME NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `idx_openid` (`openid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

若希望重用现有连接字符串，可通过 `DATABASE_URL`（例如 `mysql://root:123456@127.0.0.1:3306/yzty`）覆盖上述逐项配置。

### 4. 配置IP白名单

**重要**：在微信公众平台后台添加服务器IP到白名单：
1. 登录微信公众平台
2. 进入"开发 > 基本配置"
3. 在"IP白名单"中添加你的服务器IP地址
4. 保存配置

### 5. 运行服务器

```bash
cd server
cargo run
```

## API接口

### 1. 获取JSSDK配置
```bash
POST /wechat/jssdkConfig
Content-Type: application/json

{
  "url": "https://your-domain.com/page"
}
```

### 2. 获取用户信息
```bash
POST /wechat/getUserInfo
Content-Type: application/json

{
  "code": "微信授权回调的code"
}
```

### 3. 解密手机号（小程序）
```bash
POST /wechat/decryptPhoneNumber
Content-Type: application/json

{
  "encryptedData": "加密数据",
  "iv": "初始向量",
  "code": "登录code或session_key"
}
```

### 4. 手动绑定手机号（公众号 H5）
```bash
POST /wechat/bindPhone
Content-Type: application/json

{
  "openid": "用户唯一标识",
  "phone": "11位手机号"
}
```

- 成功返回 `data` 中的用户信息（包含 `phone` 字段）
- 若 openid 不存在，返回 404
- 建议在前端引导用户手动输入验证码后再调用

## 常见问题

### Q: 报错 "invalid appid"
A: 检查 AppID 是否正确配置在 `.env` 文件中

### Q: 报错 "invalid ip, not in whitelist"
A: 需要在微信公众平台后台添加服务器IP到白名单

### Q: 报错 "invalid code"
A: 微信授权code只能使用一次，且有效期很短（5分钟）

### Q: AppSecret 验证失败
A: 确保 AppSecret 是32位字符串，从微信后台完整复制

## 安全建议

1. **不要将 `.env` 文件提交到版本控制系统**
2. **生产环境使用环境变量而非 `.env` 文件**
3. **定期更换 AppSecret**
4. **使用 HTTPS 部署生产环境**

## 技术架构

- **框架**: Axum (Rust Web框架)
- **缓存**: 内存缓存（RwLock + HashMap）
- **加密**: AES-128-CBC（手机号解密）
- **签名**: SHA1（JSSDK签名）

## 缓存策略

- Access Token: 7000秒（约2小时），提前5分钟刷新
- Jsapi Ticket: 7000秒（约2小时），提前5分钟刷新

## 开发调试

启用调试模式查看详细日志：
```bash
RUST_LOG=debug cargo run
```

## 许可证

本项目仅供学习使用，使用前请确保已获得微信相关API的使用权限。