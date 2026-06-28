# RustChat

RustChat 是一个基于 Rust 后端和 Vue 3 前端实现的轻量级 Web 即时聊天系统。项目定位为课程作业 / 练手项目 / 软件工程流程实践项目，目标是完成一个可运行、可演示、可验证的基础 IM 聊天闭环。

系统支持用户注册登录、JWT 鉴权、私聊、群聊、群成员管理、WebSocket 实时消息、会话列表、历史消息、未读状态、文件上传、图片/GIF/表情消息和语音消息。

## 技术架构图

![RustChat 技术架构图](docs/img/rustchat-technical-architecture-v2.drawio.svg)

架构图源文件见：[docs/img/rustchat-technical-architecture-v2.drawio.svg](docs/img/rustchat-technical-architecture-v2.drawio.svg)

## 项目技术总览

| 层次 | 技术 / 工具 | 说明 |
|---|---|---|
| 前端框架 | Vue 3 + TypeScript + Vite | 构建浏览器端聊天界面 |
| 前端状态 | Pinia | 管理登录态、会话、消息、连接状态 |
| 前端路由 | Vue Router | 管理登录页和聊天页路由 |
| HTTP 客户端 | axios | 请求后端 HTTP API |
| 实时通信 | WebSocket | 支持实时消息发送、接收、重连和会话刷新 |
| 后端语言 | Rust 2024 Edition | 提供类型安全和高可靠后端实现 |
| Web 框架 | axum + tokio | 提供 HTTP API 和 WebSocket 服务 |
| 数据库访问 | sqlx | PostgreSQL 访问和 migration 管理 |
| 数据库 | PostgreSQL | 保存用户、会话、成员、消息、已读状态和文件元信息 |
| 鉴权 | JWT + Argon2 | JWT 负责接口身份认证，Argon2 负责密码哈希 |
| 日志 | tracing | 服务端日志输出和联调排障 |
| 文件存储 | 本地 `UPLOAD_DIR` | 保存上传后的文件内容 |
| 接口验证 | Postman collection | 验证 HTTP API 和 WebSocket 链路 |

## 主要功能

### 用户与认证

- 用户注册、登录。
- 密码哈希存储。
- JWT 登录态认证。
- 获取当前登录用户信息。
- 搜索用户，用于创建私聊和添加群成员。

### 私聊与群聊

- 创建或复用私聊会话。
- 创建群聊。
- 群聊 owner 添加成员、移除普通成员。
- 群成员查看成员列表和主动退出群聊。
- 私聊唯一性约束，避免同一对用户产生重复私聊。

### 消息与实时通信

- WebSocket 建立实时连接。
- 文本消息实时发送与接收。
- 消息先持久化，再推送给在线成员。
- 发送方收到发送确认。
- 支持 WebSocket 自动重连。
- 支持重连后重新拉取会话列表和当前会话历史消息。
- 支持本地乐观消息、发送状态、pending 队列和失败重试。

### 会话列表与历史消息

- 展示当前用户参与的私聊和群聊。
- 展示最近消息、最近时间和未读数。
- 支持会话按最近活跃时间排序。
- 支持历史消息分页查询。
- 支持会话标记已读。

### 文件与媒体消息

- 文件分片上传、完成确认和下载。
- 上传文件生成文件消息。
- 支持图片 / GIF 内联预览。
- 支持将图片 / GIF 保存为浏览器本地表情。
- 支持语音录制、上传和播放器展示。
- 文件上传校验 owner、分片、大小和 SHA256。

### 前端体验

- 登录页和聊天页。
- 会话列表、消息列表、消息输入框。
- 群成员面板和用户搜索。
- 文件上传进度。
- WebSocket 在线、连接中、重连中、离线状态提示。
- 刷新后恢复登录态和当前会话。

## 项目目录结构

```text
rustchat-server/
├─ backend/                                  # Rust 后端服务
│  ├─ Cargo.toml                             # 后端依赖与包配置
│  ├─ .env.example                           # 后端环境变量示例
│  ├─ migrations/                            # sqlx 数据库迁移
│  └─ src/
│     ├─ main.rs                             # 后端启动入口
│     ├─ app.rs                              # 应用状态、服务组装、启动依赖
│     ├─ router.rs                           # 总路由挂载
│     ├─ auth/                               # JWT、密码哈希、认证类型
│     ├─ common/                             # 配置、错误、响应、日志通用能力
│     ├─ middleware/                         # 鉴权中间件、请求日志中间件
│     ├─ storage/                            # PostgreSQL 连接与 repository 上下文
│     ├─ user/                               # 注册、登录、当前用户、用户搜索
│     ├─ session/                            # 私聊、群聊、成员管理、已读状态
│     ├─ message/                            # 消息发送、消息持久化、历史消息
│     ├─ conversation/                       # 会话列表、最近消息、未读数
│     ├─ connection/                         # WebSocket 连接、协议、在线推送
│     ├─ file/                               # 文件上传、下载、文件消息
│     └─ integration_tests.rs                # 真实数据库与端到端测试
│
├─ frontend/                                 # Vue 3 前端应用
│  ├─ package.json                           # 前端依赖与脚本
│  ├─ .env.example                           # 前端环境变量示例
│  └─ src/
│     ├─ main.ts                             # 前端入口
│     ├─ App.vue                             # 根组件
│     ├─ router/                             # Vue Router 路由
│     ├─ api/                                # HTTP API 封装
│     ├─ ws/                                 # WebSocket client
│     ├─ stores/                             # Pinia 状态管理
│     ├─ views/                              # LoginView、ChatView 页面
│     ├─ components/                         # 会话、消息、输入框、用户搜索组件
│     ├─ types/                              # 前端类型定义
│     ├─ utils/                              # 表情、本地资源辅助逻辑
│     └─ styles/                             # 全局样式
│
├─ docs/                                     # 项目文档
│  ├─ product/                               # 产品规格、任务、需求分析、阶段路线
│  ├─ architecture/                          # 技术架构、数据库、目录结构
│  ├─ api/                                   # HTTP / WebSocket API 与 Postman
│  ├─ setup/                                 # 环境搭建
│  ├─ design/                                # 设计稿与设计说明
│  ├─ summary/                               # 阶段总结、验收记录
│  ├─ problems/                              # 已知问题与排障记录
│  └─ img/                                   # 文档图片与架构图源文件
│
└─ README.md                                 # 项目入口说明
```

## 主要功能文件分布

### 后端功能文件

| 功能 | 主要位置 | 说明 |
|---|---|---|
| 应用启动 | `backend/src/main.rs`、`backend/src/app.rs` | 读取配置、初始化服务、组装应用状态 |
| 路由聚合 | `backend/src/router.rs` | 挂载健康检查、用户、会话、消息、文件和 WebSocket 路由 |
| 配置与错误 | `backend/src/common/` | 环境变量、统一错误、统一响应、日志初始化 |
| 认证能力 | `backend/src/auth/`、`backend/src/middleware/auth.rs` | JWT 生成校验、密码哈希、受保护接口鉴权 |
| 用户系统 | `backend/src/user/` | 注册、登录、当前用户、用户搜索 |
| 会话系统 | `backend/src/session/` | 私聊、群聊、成员列表、添加成员、移除成员、退出群聊、标记已读 |
| 消息系统 | `backend/src/message/` | 消息写入、历史消息分页、消息业务校验 |
| 会话列表 | `backend/src/conversation/` | 当前用户会话列表、最近消息、未读数 |
| WebSocket | `backend/src/connection/` | WebSocket handler、在线连接管理、协议事件 |
| 文件消息 | `backend/src/file/` | 文件上传初始化、分片、完成、下载和文件消息生成 |
| 数据库 | `backend/src/storage/`、`backend/migrations/` | 数据库连接、迁移、表结构演进 |
| 集成测试 | `backend/src/integration_tests.rs` | 真实数据库私聊、群聊、文件上传等端到端测试 |

### 前端功能文件

| 功能 | 主要位置 | 说明 |
|---|---|---|
| 应用入口 | `frontend/src/main.ts`、`frontend/src/App.vue` | 创建 Vue 应用并挂载路由和 Pinia |
| 页面路由 | `frontend/src/router/index.ts` | 登录页、聊天页和路由守卫 |
| 登录页面 | `frontend/src/views/LoginView.vue` | 用户注册、登录、进入聊天页 |
| 聊天页面 | `frontend/src/views/ChatView.vue` | 聊天主界面，组合会话、消息、成员和输入区域 |
| HTTP API | `frontend/src/api/` | auth、users、sessions、messages、conversations、files 请求封装 |
| WebSocket | `frontend/src/ws/client.ts`、`frontend/src/stores/connection.ts` | 连接创建、状态维护、重连、消息发送 |
| 聊天状态 | `frontend/src/stores/chat.ts` | 会话、消息、成员、未读、pending 消息等状态 |
| 登录状态 | `frontend/src/stores/auth.ts` | token 和当前用户本地持久化 |
| 会话列表组件 | `frontend/src/components/ConversationList.vue` | 展示会话列表、最近消息、未读数 |
| 消息列表组件 | `frontend/src/components/MessageList.vue` | 展示文本、文件、图片/GIF、语音消息 |
| 输入框组件 | `frontend/src/components/MessageInput.vue` | 文本输入、文件选择、GIF 面板、录音 |
| 群成员面板 | `frontend/src/components/GroupChatPanel.vue` | 展示群成员、添加或移除成员 |
| 用户搜索 | `frontend/src/components/UserSearch.vue` | 搜索用户并发起私聊或添加群成员 |
| 表情工具 | `frontend/src/utils/stickers.ts` | 本地自定义表情保存和读取 |
| 类型定义 | `frontend/src/types/` | API 和聊天业务类型 |
| 全局样式 | `frontend/src/styles/main.css` | 页面布局和组件样式 |

### 文档与资料

| 内容 | 位置 |
|---|---|
| 文档总入口 | `docs/README.md` |
| 产品概述 | `docs/product/overview.md` |
| 产品规格 | `docs/product/spec.md` |
| 需求与可行性分析 | `docs/product/requirements-analysis.md` |
| Phase 时间轴 | `docs/product/phase-timeline.md` |
| Phase 明细 | `docs/product/phase-breakdown.md` |
| Git 提交记录导出 | `docs/product/git-commit-history.md` |
| 数据库设计 | `docs/architecture/db.md` |
| 技术架构说明 | `docs/architecture/technical-architecture.md` |
| API 文档 | `docs/api/http-api.md` |
| Postman 集合 | `docs/api/postman/` |
| 环境搭建 | `docs/setup/env-setup-vm.md` |
| 阶段总结 | `docs/summary/` |
| 已知问题 | `docs/problems/` |

## 本地运行

### 1. 后端

准备环境变量：

```bash
cd backend
cp .env.example .env
```

确认 `.env` 中的 `DATABASE_URL`、`JWT_SECRET`、`UPLOAD_DIR` 等配置可用，然后启动：

```bash
cargo run
```

默认后端地址：

```text
http://127.0.0.1:3000
```

健康检查：

```text
GET /health
```

### 2. 前端

准备环境变量和依赖：

```bash
cd frontend
cp .env.example .env
npm install
```

启动开发服务：

```bash
npm run dev
```

默认前端会读取：

```text
VITE_API_BASE_URL=http://127.0.0.1:3000
VITE_WS_BASE_URL=ws://127.0.0.1:3000
```

### 3. 常用验证

后端测试：

```bash
cd backend
cargo test
```

前端构建：

```bash
cd frontend
npm run build
```

真实数据库 ignored 测试需要配置独立 `TEST_DATABASE_URL`，不要指向日常开发库。

## 运行与生成目录

以下目录为生成物或运行时产物，不作为源码重点维护：

- `backend/target/`
- `frontend/node_modules/`
- `frontend/dist/`
- `backend/uploads/`

这些目录可以通过编译、安装依赖、构建前端或上传文件重新生成。

## 相关文档入口

- 文档总入口：[docs/README.md](docs/README.md)
- API 文档：[docs/api/http-api.md](docs/api/http-api.md)
- 数据库设计：[docs/architecture/db.md](docs/architecture/db.md)
- 技术架构：[docs/architecture/technical-architecture.md](docs/architecture/technical-architecture.md)
- 需求与可行性分析：[docs/product/requirements-analysis.md](docs/product/requirements-analysis.md)
- 阶段成果路线图：[docs/product/phase-timeline.md](docs/product/phase-timeline.md)
- Phase 阶段成果明细：[docs/product/phase-breakdown.md](docs/product/phase-breakdown.md)
- Git 提交记录：[docs/product/git-commit-history.md](docs/product/git-commit-history.md)
