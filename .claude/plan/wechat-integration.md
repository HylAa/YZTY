# 微信API集成实现计划

## 任务描述
实现后端真实的微信用户信息获取和手机号解密功能，替换现有的模拟数据。

## 技术方案
采用轻量级内存缓存方案，使用 tokio::sync::RwLock 管理 Access Token 和 Jsapi Ticket 缓存。

## 项目结构
```
server/
├── src/
│   ├── main.rs              # 主入口
│   ├── wechat/              # 微信模块
│   │   ├── mod.rs           # 模块定义
│   │   ├── client.rs        # 微信API客户端
│   │   ├── config.rs        # 配置管理
│   │   ├── cache.rs         # 缓存管理
│   │   └── crypto.rs        # 加解密工具
│   └── handlers/            # 处理器
│       ├── mod.rs
│       └── wechat.rs        # 微信相关处理器
├── Cargo.toml               # 依赖配置
└── .env                     # 环境变量
```

## 核心功能
1. **JSSDK签名**: 实现 jsapi_ticket 获取和签名计算
2. **用户信息获取**: OAuth2.0 授权流程，通过 code 换取用户信息
3. **手机号解密**: AES-128-CBC 解密算法实现

## 执行步骤

### ✅ Step 1: 添加依赖
- dotenvy: 环境变量管理
- base64: Base64 编解码
- hex: 十六进制转换
- aes, block-modes: AES 加解密

### Step 2: 创建微信配置模块
- 从环境变量加载 AppID 和 AppSecret
- 配置验证和管理

### Step 3: 实现缓存管理
- Access Token 缓存（2小时有效）
- Jsapi Ticket 缓存（2小时有效）
- 自动刷新机制

### Step 4: 创建微信API客户端
- get_access_token(): 获取访问令牌
- get_jsapi_ticket(): 获取 JS 票据
- get_user_info(): 获取用户信息
- 统一错误处理

### Step 5: 实现加密算法
- SHA1 签名算法
- 随机字符串生成
- AES-128-CBC 解密

### Step 6: 更新处理器
- 真实 JSSDK 签名计算
- OAuth 用户信息获取
- 手机号解密实现

### Step 7: 集成主应用
- 初始化微信客户端
- 注入 AppState
- 更新路由

### Step 8: 环境配置
- WECHAT_APPID
- WECHAT_APPSECRET
- PORT

## 技术要点
- **并发安全**: Arc<RwLock> 保证缓存线程安全
- **错误处理**: 统一错误响应格式
- **性能优化**: 缓存机制避免频繁调用微信API
- **安全性**: 敏感信息环境变量管理

## 测试要点
1. JSSDK 配置签名验证
2. 用户信息获取流程测试
3. 手机号解密功能测试（需要真实数据）
4. 缓存过期和刷新测试

## 注意事项
- 需要真实的 WECHAT_APPSECRET
- URL 必须与前端完全一致
- 所有 API 调用需要日志记录
- 手机号解密仅在小程序环境有效