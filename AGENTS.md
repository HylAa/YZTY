# Repository Guidelines

## 项目结构与模块组织
- 技术栈：Vue 3、Vue Router 4、Vuex 4、Vant、ECharts、Axios、Vue CLI 5。
- 主要目录：
  - `src/views/` 页面视图（如 `Login.vue`、`admin/Dashboard.vue`）
  - `src/components/` 公共组件
  - `src/router/index.js` 路由配置
  - `src/store/` Vuex 模块（如 `modules/user.js`、`courses.js`）
  - `src/api/index.js` 接口封装与拦截器
  - `src/utils/` 工具方法（如 `wxUtils.js`）
  - `src/assets/css/global.css` 全局样式；`public/` 静态资源；`dist/` 构建产物

## 构建、测试与本地开发命令
- `npm ci`/`npm install`：安装依赖（CI/本地）。
- `npm run serve`：启动开发服务器（HMR，默认本地端口）。
- `npm run build`：生产构建输出到 `dist/`。
- `npm run lint`：运行 ESLint 检查并尝试自动修复。

## 代码风格与命名规范
- 缩进 2 空格；优先使用 `const`/`let`；避免未使用变量与魔法数。
- 组件/视图文件使用 PascalCase（如 `UserManagement.vue`）。
- JS 与 Vuex 模块使用 lowerCamelCase（如 `courses.js`）。
- 样式使用 Less/CSS，建议 BEM；全局样式集中于 `assets/css`。
- Lint 遵循 `.eslintrc.js` 与 `eslint-plugin-vue` 规则，提交前需通过。

## 测试指南
- 当前未配置自动化测试；推荐后续引入 Vitest/Jest + Vue Test Utils。
- 命名建议：与源码同目录 `*.spec.js` 或使用 `__tests__/` 目录。
- 覆盖优先：API 封装、路由守卫、关键组件交互逻辑；建议覆盖率 ≥ 70%。

## 提交与 Pull Request 规范
- 提交信息建议采用 Conventional Commits：
  - 示例：`feat: 新增课程筛选`、`fix: 修复登录重定向`、`chore: 升级依赖`。
- 分支命名：`feat/xxx`、`fix/xxx`、`chore/xxx`。
- PR 要求：
  - 说明变更动机与范围，关联 Issue（如 `#123`）。
  - UI 变更附前后截图；列出验证步骤与影响面。
  - 合并前确保 `npm run lint` 与 `npm run build` 通过，控制台无明显报错。

## 安全与配置提示（前端）
- 环境变量使用 `VUE_APP_*` 前缀；勿提交 `.env.*` 中的敏感信息。
- 在 `src/api/index.js` 设置 `baseURL`、超时与错误拦截，避免泄露后端细节。
- 仅提交必要静态资源与脱敏数据，避免在仓库中存放密钥与私有证书。

