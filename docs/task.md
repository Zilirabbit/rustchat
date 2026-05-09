# RustChat - 开发任务清单（refined task.md）

## 1. 使用说明

本任务清单用于管理 RustChat 项目的开发进度。

任务状态约定：

- `[ ]` 未开始
- `[~]` 进行中
- `[x]` 已完成
- `[-]` 暂不做 / 延后

每个任务建议补充：

- 输出物（代码 / 文档 / migration / 测试）
- 完成标准
- 依赖项

---

## 2. 里程碑规划

### Milestone 1：项目可启动

目标：服务可运行，具备基础工程能力。

### Milestone 2：用户系统闭环

目标：用户可注册、登录，并获取 JWT。

### Milestone 3：私聊主链路闭环

目标：用户可建立连接并完成私聊发送与接收。

### Milestone 4：会话与历史消息闭环

目标：可展示会话列表并查询历史消息。

### Milestone 5：群聊与基础前端演示

目标：可完成群聊与前端演示。

---

## 3. Phase 1：基础能力（必须完成）

## 3.1 项目初始化

### 任务

- [x] 初始化 Rust 项目（cargo workspace 或单项目）
- [x] 建立基础目录结构（common / user / session / message / connection / storage）
- [x] 引入核心依赖（axum / tokio / sqlx / tracing / serde / jsonwebtoken）
- [x] 配置 `.env` 与示例配置文件
- [x] 建立基础启动入口 `main.rs`
- [x] 添加健康检查接口 `/health`

### 输出物

- 可编译运行的项目骨架
- `Cargo.toml`
- 基础目录结构
- 环境变量模板

### 完成标准

- 项目可成功启动
- 健康检查接口可访问
- 本地开发环境配置清晰可复现

---

## 3.2 common 基础模块

### 任务

- [x] 日志系统（tracing）
- [x] 配置管理（env / config）
- [x] 错误类型封装（thiserror / anyhow）
- [x] 统一响应结构定义
- [x] 通用 Result / AppError 定义

### 输出物

- `common` 模块代码
- 错误处理约定
- 统一响应格式
- 基础验证方式说明

### 完成标准

- 服务启动日志正常输出
- 错误可统一转换为接口响应
- 配置项可从环境变量读取

### 验证建议

- 启动服务后手动验证 `GET /health`
- 未配置 `DATABASE_URL` 时验证 `GET /db/ping` 返回统一错误响应
- 执行 `cargo check`
- 执行 `cargo test`
- 后续补充 `config`、`AppError -> HTTP 响应`、统一响应结构的单元测试

### 后续扩展约定

- 当前响应中的 `code` 可先用于基础返回，后续建议从 HTTP 状态码语义逐步细化为稳定的业务错误码
- 后续建议拆分 `HTTP status` 与 `business code`，避免前端仅靠 `400 / 401 / 500` 区分具体业务错误
- 业务错误码建议按模块命名，例如：`COMMON_CONFIG_INVALID`、`AUTH_INVALID_TOKEN`、`USER_ALREADY_EXISTS`
- `AppError` 后续建议统一提供 `status_code()`、`error_code()`、`user_message()` 等方法，保证可扩展性
- 统一响应结构后续可预留 `request_id` 字段，便于日志追踪与问题定位

---

## 3.3 数据库模块（storage）

### 任务

- [x] 建立 PostgreSQL 连接池
- [x] 接入 sqlx
- [x] 初始化 migration 体系
- [x] 编写基础数据库连接测试
- [x] 约定 Repository 层结构

### 输出物

- DB 连接模块
- migration 初始化文件
- storage 模块结构

### 完成标准

- 服务启动后可成功连接数据库
- migration 可执行
- 本地数据库初始化流程清晰

---

## 3.4 数据库表设计

### 任务

- [x] 设计 `users` 表
- [x] 设计 `sessions` 表
- [x] 设计 `session_members` 表
- [x] 设计 `messages` 表
- [x] 设计未读状态表（如 `user_session_read_state`）
- [x] 编写对应 migration 文件

### 输出物

- SQL migration 文件
- `db.md` 或表结构说明

### 完成标准

- 所有核心表可成功创建
- 表关系满足用户 / 会话 / 消息需求
- 为后续会话列表与未读统计预留字段

---

## 3.5 用户系统（user）

### 任务

- [x] 定义用户实体与 DTO
- [x] 实现用户注册接口
- [x] 实现用户登录接口
- [x] 实现密码加密与校验
- [x] 实现 JWT 生成与校验
- [x] 实现 `GET /me` 接口
- [x] 编写注册 / 登录接口测试

### 输出物

- user 模块代码
- users CRUD
- 用户认证接口
- 基础测试

### 完成标准

- 用户可注册
- 用户可登录并获取 token
- token 可用于访问受保护接口
- 错误登录会返回正确错误

### 真实数据库测试补充时机（建议）

当前 `3.5` 可先用以下两层测试完成第一轮闭环：

- service 层使用内存 / fake repository，先锁定注册、登录、密码校验、JWT 逻辑
- route / handler 层使用 stub service，先锁定接口路径、请求参数、响应结构、鉴权入口

这样可以先保证：

- 用户系统代码可快速迭代
- 不依赖本地 PostgreSQL 状态即可完成基础自测
- 不会误污染开发库中的真实用户数据

但这不代表真实数据库测试可以长期缺失。

如果本阶段暂时不补，建议在以下时间点补上：

- `3.6 中间件（middleware）` 完成后
- `3.7 WebSocket 基础` 开始前

原因：

- 到 `3.6` 完成时，`JWT 校验`、`当前登录用户提取`、`受保护接口访问` 这条 HTTP 认证链路已经基本稳定，适合一次性验证“真实库 + 真实 token + 真实接口”
- 在 `3.7 / 3.8` 之前补上，可以尽早确认 `users` 表、登录态、token 解析没有隐藏问题，避免把基础认证问题带到 WebSocket 和私聊链路里放大
- 如果等到私聊、会话、消息功能都写完再补，排查失败时很难区分问题究竟出在 `user`、`middleware`、`session` 还是 `message`

建议将这项工作视为：

- `3.5` 的增强收尾
- 同时也是 `3.6` 的联调前置校验

建议补充内容：

- 新增 `TEST_DATABASE_URL`
- 测试启动前自动执行 migration
- 每条测试前后清理 `users` 及其相关依赖数据，或使用事务回滚保证隔离
- 明确区分“开发库”和“测试库”，禁止直接对日常开发库跑破坏性测试

真实数据库测试至少应覆盖：

- 注册成功后，`users` 表中确实存在新用户记录
- 重复用户名注册时，数据库唯一约束与接口返回保持一致
- 登录时可从真实数据库读取用户并完成密码校验
- `GET /me` 可通过真实登录获得的 token 访问成功
- 错误密码访问被拒绝
- 缺失 / 非法 token 访问被拒绝

完成标志建议：

- 能在独立测试库中重复执行
- 多次运行结果一致，不依赖人工手动清表
- 不污染开发环境已有数据
- 出错时可快速定位是 migration、repo、service 还是 middleware 问题

---

## 3.6 中间件（middleware）

### 任务

- [x] JWT 鉴权中间件
- [x] 请求日志中间件
- [x] 错误日志记录
- [x] 提取当前登录用户上下文

### 输出物

- middleware 模块代码

### 完成标准

- 受保护接口可识别当前用户
- 未登录请求会被拒绝
- 请求日志可追踪

---

## 3.7 WebSocket 基础（connection）

### 任务

- [x] 建立 WebSocket 连接入口
- [x] WebSocket 握手时 JWT 鉴权
- [x] 维护 `user_id <-> connection` 映射
- [x] 实现断线清理
- [x] 定义基础 WS 消息协议
- [x] 可选：实现 heartbeat/ping-pong

### 输出物

- connection 模块代码
- 基础 WS 协议说明

### 完成标准

- 合法 token 可建立连接
- 非法 token 会被拒绝
- 在线用户连接状态可管理
- 断开连接后可正确清理

---

## 3.8 私聊功能（message + session）

### 任务

- [x] 创建私聊 session 接口
- [x] 实现私聊成员校验
- [x] 实现发送文本消息
- [x] 消息入库
- [x] 单点推送给接收方
- [x] 对发送方返回确认
- [x] 编写最小链路测试

### 输出物

- session 模块基础能力
- message 模块基础能力
- 私聊发送链路

### 完成标准

- 用户可创建私聊会话
- 用户可发送私聊消息
- 接收方在线时可实时收到
- 消息可在数据库中查询到

---

## 4. Phase 2：核心功能

## 4.1 群聊系统

### 任务

- [ ] 创建群聊
- [ ] 添加群成员
- [ ] 退出群聊
- [ ] 群聊成员权限校验
- [ ] 群消息广播

### 完成标准

- 可创建群聊
- 成员可加入 / 退出
- 群成员在线时可收到群消息

---

## 4.2 会话列表（conversation）

### 任务

- [x] 获取用户会话列表
- [x] 查询最近消息
- [x] 未读数统计
- [x] 按最近消息时间排序
- [x] 定义 conversation VO

### 完成标准

- 用户可看到自己的会话列表
- 会显示最近消息和时间
- 未读数正确

---

## 4.3 历史消息

### 任务

- [x] 按 session 查询历史消息
- [x] 支持分页参数
- [x] 增加成员权限校验
- [x] 定义消息列表返回结构
- [x] 编写历史消息接口测试

### 完成标准

- 用户可分页查询历史消息
- 非成员不可查询
- 消息顺序稳定

---

## 4.4 客户端联调前置接口收口

目标：在正式开始前端工作前，先补齐后续客户端演示和联调会依赖的最小 HTTP API，避免到前端阶段才发现创建私聊必须手填 `target_user_id`，以及 `unread_count` 只能展示但无法清零。

### 任务

- [x] 实现用户搜索接口，例如 `GET /api/users?keyword=`
- [x] 用户搜索结果返回 `user_id`、`username` 等创建私聊所需字段
- [x] 用户搜索接口接入 JWT 鉴权，只允许登录用户查询
- [x] 实现会话已读接口，例如 `POST /api/sessions/:id/read`
- [x] 已读接口校验当前用户必须是该 session 成员
- [x] 已读接口更新 `user_session_read_state.last_read_message_id / last_read_at`
- [x] 已读后再次查询会话列表时，当前用户该会话的 `unread_count` 可正常归零
- [x] 补充 route / service / repo 层测试
- [x] 补充真实数据库集成测试
- [x] 更新 `docs/api.md` 与 Postman collection

### 输出物

- user 模块用户搜索能力
- session 或 conversation 模块已读写回能力
- 对应 DTO / handler / service / repo / routes 代码
- API 文档与 Postman 示例
- 单元测试与集成测试

### 完成标准

- 后续客户端可通过用户名关键字搜索目标用户并拿到 `target_user_id`
- 搜索结果不泄露密码哈希等敏感字段
- 当前用户不能通过已读接口操作自己未加入的会话
- 已读接口对空会话、无新消息会话、重复调用保持稳定
- 标记已读后，会话列表中的 `unread_count` 与数据库 `user_session_read_state` 一致

### 本阶段最小范围

- 用户搜索：
  - 支持 `keyword` 按用户名模糊匹配
  - 返回创建私聊所需的最小用户信息：`user_id`、`username`
  - 接口需要 JWT 鉴权
- 会话已读：
  - 只做“当前用户将某会话标记到最新消息”的会话级已读
  - `POST /api/sessions/:id/read` 不要求请求体
  - 校验当前用户必须是该 session 成员
  - 标记后再次查询会话列表，该会话 `unread_count` 应可归零

---

## 5. Phase 3：完善体验

## 5.1 图片消息（可选）

### 任务

- [ ] 文件上传接口
- [ ] 本地存储或静态资源访问
- [ ] 图片消息结构定义
- [ ] 图片消息展示

---

## 5.2 前端 UI

### 前端环境搭建任务

- [x] 确认前端技术栈：`Vue 3 + TypeScript + Vite`
- [x] 确认前端基础依赖：`vue-router`、`pinia`、`axios`
- [x] 在项目根目录创建 `frontend/` 应用目录
- [x] 初始化 Vite Vue TypeScript 项目
- [x] 配置前端 `.env.example`
- [x] 配置后端 HTTP API 地址，例如 `VITE_API_BASE_URL`
- [x] 配置 WebSocket 地址，例如 `VITE_WS_BASE_URL`
- [x] 配置开发服务器监听 `0.0.0.0`，便于 Windows 宿主机访问虚拟机内前端服务
- [x] 补充前端启动说明，例如 `npm install`、`npm run dev -- --host 0.0.0.0`
- [x] 验证前端 dev server 可从宿主机浏览器访问（本机 `curl` 已返回 `200`，宿主机浏览器已确认可访问）

安装依赖与启动命令：

```bash
cd frontend
npm install
npm run dev -- --host 0.0.0.0
```

如需从零手动补齐依赖：

```bash
cd frontend
npm install vue vue-router pinia axios
npm install -D vite typescript @vitejs/plugin-vue vue-tsc
```

### 前端环境完成标准

- `frontend/` 目录存在且可独立启动
- `npm install` 可成功安装依赖
- `npm run dev -- --host 0.0.0.0` 可启动 Vite 开发服务器
- Windows 宿主机可通过 `http://<虚拟机IP>:5173` 访问前端页面
- 前端环境变量可区分 HTTP API 与 WebSocket 地址
- 暂不要求完成具体聊天 UI 和接口联调

### 任务

- [ ] 登录页
- [ ] 聊天主页面
- [ ] 会话列表组件
- [ ] 消息列表组件
- [ ] 消息输入框
- [ ] 登录态管理
- [ ] 与后端接口联调
- [ ] 与 WebSocket 联调

### 完成标准

- 用户可登录进入聊天页面
- 可看到会话列表
- 可发送并接收消息
- 页面刷新后可恢复基础状态

---

## 5.3 客户端扩展范围评估

### 任务

- [ ] 用户搜索与会话已读扩展范围后置到前端开始后再评估

### 前端开始后再评估的扩展范围

- 用户搜索：
  - 支持 `limit`，默认 20，设置最大上限避免一次返回过多
  - 默认排除当前登录用户，避免客户端误选自己创建私聊
  - 头像、昵称、在线状态、好友关系、分页游标
- 会话已读：
  - 如果会话暂无消息，则可只更新 `last_read_at`，`last_read_message_id` 保持为空
  - 逐条已读回执
  - 通过 WebSocket 广播已读状态，影响对方客户端界面

---

## 5.4 用户体验优化（可选）

### 任务

- [ ] 自动重连
- [ ] 消息状态（发送中 / 已发送 / 失败）
- [ ] loading 状态
- [ ] 错误提示优化

---

## 6. 测试与验收任务

## 6.1 接口测试

- [x] 注册接口测试
- [x] 登录接口测试
- [x] 注册接口真实数据库集成测试
- [x] 登录接口真实数据库集成测试
- [x] `GET /me` 真实数据库集成测试
- [x] 创建私聊接口测试
- [x] 会话列表接口测试
- [x] 历史消息接口测试

## 6.2 WebSocket 测试

- [x] WS 连接建立测试
- [x] 非法 token 拒绝测试
- [x] 私聊消息收发测试
- [ ] 群聊消息广播测试
- [x] 断线清理测试

## 6.3 集成验证

- [x] 从注册到登录完整链路验证
- [x] 基于真实数据库的“注册 -> 登录 -> /me”完整链路验证
- [x] 从登录到聊天完整链路验证
- [ ] 刷新页面后历史消息验证

---

## 7. 文档任务

- [ ] 更新 `spec.md`
- [ ] 更新 `task.md`
- [ ] 编写 `summary.md`
- [ ] 编写 `api.md`
- [ ] 编写 `db.md`
- [ ] 编写 `README.md`

---

## 8. Git 与开发管理任务

- [ ] 建立 `main / dev / feature/*` 分支策略
- [ ] 约定 commit message 规范
- [ ] 每完成一个 milestone 写阶段总结
- [ ] 合并前完成自测

---

## 9. 可选增强（后期）

- [-] Redis（在线状态 / 缓存）
- [-] Nginx（部署）
- [-] Docker 化部署
- [-] 消息队列（解耦）
- [-] 已读回执
- [-] 用户在线状态展示

---

## 10. 当前推荐执行顺序

建议优先按如下顺序推进：

1. 项目初始化
2. common 基础模块
3. storage + migration
4. users 表与用户系统
5. middleware
6. WebSocket 基础
7. 私聊主链路
8. 会话列表
9. 历史消息
10. 客户端联调前置接口收口（用户搜索 + 会话已读）
11. 前端环境搭建
12. 前端私聊 UI 与联调
13. 群聊
14. 可选优化
