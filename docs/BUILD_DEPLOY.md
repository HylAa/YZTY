# 构建与部署指南（前端 + 后端）

本项目包含：
- 前端：Vue 3 + Vite + Vant（H5），路径 `./`（根目录）
- 后端：Rust + Axum，路径 `./server`

目标：本地开发/构建、Linux 服务器部署、Nginx 同域反代、微信授权与手机号获取相关注意事项。

## 1. 前端（Vite）

### 1.1 本地开发
```bash
npm install
npm run dev
```

常用环境变量（.env 或 .env.local）：
```
VITE_API_BASE=https://yzty.nailinwanhe.com
VITE_WECHAT_APPID=你的公众号AppID
VITE_WECHAT_REDIRECT_ORIGIN=https://yzty.nailinwanhe.com
```

注意：Vite 只会注入以 `VITE_` 开头的变量；在代码中使用 `import.meta.env.*` 访问。

### 1.2 生产构建
```bash
npm run build
```
构建产物输出到 `dist/`，将其部署到 Nginx 指定的站点根目录（示例：`/www/wwwroot/yzty.nailinwanhe.com`）。

### 1.3 反向代理注意点（与后端对齐）
- 前端调用以 `/api/*`、`/wechat/*` 开头的接口。
- Nginx 将 `location /api/ { proxy_pass http://127.0.0.1:8018/; }`（带尾斜杠）用于“去掉 /api 前缀”转发。
  - 例：`/api/wechat/getUserInfo` → 后端收到 `/wechat/getUserInfo`。
- 若后端直接实现 `/api/*`，则 `proxy_pass http://127.0.0.1:8018;` 不带尾斜杠（不改前缀）。

### 1.4 常见问题（前端）
- 报错 `process is not defined`：Vite 环境请使用 `import.meta.env.*`，不要用 `process.env.*`。
- Vant 4 Toast：请使用 `showToast / showLoadingToast / closeToast` 函数式 API，且不需要 `app.use(Toast)`。
- 非微信环境调试：获取 JSSDK/授权建议跳过或 Mock，实际功能需在微信内置浏览器验证。

## 2. 后端（Rust + Axum）

### 2.1 本地运行
```bash
cd server
cargo run
# 或指定端口
PORT=8018 cargo run
```

必要环境变量：
- `WECHAT_APPID`：公众号 AppID（真实实现需要）
- `WECHAT_SECRET`：公众号 AppSecret（真实实现需要）
- `PORT`：监听端口（默认 8018）

### 2.2 交叉编译（推荐：cargo-zigbuild）
在 macOS（含 M1）上为 Linux 服务器构建二进制：

1) 安装 Zig 与 zigbuild
```bash
brew install zig
cargo install cargo-zigbuild
```

2) 添加编译目标（musl 静态链接更通用）
```bash
rustup target add x86_64-unknown-linux-musl    # 适用于 x86_64 服务器
rustup target add aarch64-unknown-linux-musl   # 适用于 ARM64 服务器
```

3) 构建
```bash
cd server
# x86_64 服务器
cargo zigbuild --release --target x86_64-unknown-linux-musl
# ARM64 服务器
cargo zigbuild --release --target aarch64-unknown-linux-musl
```

4) 产物与验证
```
server/target/<triple>/release/yzty-server
file server/target/<triple>/release/yzty-server  # 应为 ELF 64-bit
```

5) 上传与运行
```bash
scp server/target/<triple>/release/yzty-server user@server:/opt/yzty-server/
ssh user@server
chmod +x /opt/yzty-server/yzty-server
PORT=8018 WECHAT_APPID=xxx WECHAT_SECRET=yyy nohup /opt/yzty-server/yzty-server >/var/log/yzty-server.log 2>&1 &
```

（可选）使用 systemd 常驻：`/etc/systemd/system/yzty-server.service`
```
[Unit]
Description=YZTY Axum Server
After=network.target

[Service]
WorkingDirectory=/opt/yzty-server
Environment=PORT=8018
Environment=WECHAT_APPID=你的AppID
Environment=WECHAT_SECRET=你的AppSecret
ExecStart=/opt/yzty-server/yzty-server
Restart=always
RestartSec=3
User=www-data

[Install]
WantedBy=multi-user.target
```
启用：
```bash
systemctl daemon-reload
systemctl enable --now yzty-server
journalctl -u yzty-server -f
```

### 2.3 Nginx 同域反代（示例）
```
server {
  listen 80;
  server_name yzty.nailinwanhe.com;
  return 301 https://$host$request_uri;
}

server {
  listen 443 ssl http2;
  server_name yzty.nailinwanhe.com;

  ssl_certificate     /www/server/panel/vhost/cert/yzty.nailinwanhe.com/fullchain.pem;
  ssl_certificate_key /www/server/panel/vhost/cert/yzty.nailinwanhe.com/privkey.pem;

  root  /www/wwwroot/yzty.nailinwanhe.com;
  index index.html;

  location / { try_files $uri $uri/ /index.html; }

  # /api 去前缀转发
  location /api/ {
    proxy_pass http://127.0.0.1:8018/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
  }

  # 直转
  location /wechat/ { proxy_pass http://127.0.0.1:8018; }
  location /admin/  { proxy_pass http://127.0.0.1:8018; }
}
```

### 2.4 CORS 设置
- 同域部署时可关闭 CORS（由 Nginx 转发）。
- 若需跨域：不要将 `allow_credentials(true)` 与 `allow_origin(*)` 或 `allow_headers(*)` 同时使用；需明确指定允许的源与头。

## 3. 微信公众平台配置
- 网页授权回调域名：`yzty.nailinwanhe.com`（仅填域名，不带协议/端口/路径）。
- JS 接口安全域名：`yzty.nailinwanhe.com`。
- 前端授权跳转建议使用环境变量统一域名：`VITE_WECHAT_REDIRECT_ORIGIN=https://yzty.nailinwanhe.com`。
- JSSDK 签名 URL：使用“当前页面 URL（不含 hash）”；iOS 建议用首次进入页面 URL 参与签名。

## 4. 获取用户信息与手机号（实现建议）
- 用户信息（公众号网页授权）：`code -> access_token -> userinfo`。
- 手机号：公众号 H5 直取手机号能力受限，建议采用 H5 短信绑定（前端已提供 `BindPhoneDialog`），后端实现 `/api/sms/send` 与 `/api/sms/verify`。
- 若有小程序，可通过 `<wx-open-launch-weapp>` 唤起小程序中获取手机号，再回传 H5。

## 5. 常见问题速查
- 404：Nginx `location /api/` 与 `proxy_pass` 尾斜杠用法不当，导致前缀未去除或路径错位。
- 10003（redirect_uri 域名不一致）：公众号后台未配置域名或与你访问的域名不一致。
- 10012（appid 不能为空）：未设置 `VITE_WECHAT_APPID`/`WECHAT_APPID`，或环境变量未生效。
- CORS panic：`allow-credentials` 与 `*` 组合导致，需改为显式 Origin/Headers。
- 端口占用：`Address already in use`，用 `lsof -iTCP:<port> -sTCP:LISTEN -n -P` 排查并释放。
- `cannot execute binary file`：在 Linux 上运行 macOS 二进制，需在目标平台编译或交叉编译为 Linux ELF。
- cross 运行缺库：macOS 可改用 `cargo-zigbuild`（依赖 zig）更稳。

