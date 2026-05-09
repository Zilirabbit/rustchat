# RustChat 前端 UI 实现计划

## 1. 文档目的

本文用于指导第一版前端聊天 UI 的实现。

当前阶段不优先制作 Figma 高保真稿，也不优先用图片生成工具生成视觉预览。第一版目标是尽快做出可运行、可联调、可演示的 Web UI，验证注册 / 登录、私聊、会话列表、历史消息和 WebSocket 实时消息主链路。

后续如果需要展示效果、统一视觉规范或做移动端适配，再补 Figma 或视觉稿。

## 2. 当前结论

推荐路径：

```text
UI 实现细节文档
-> 直接实现 Vue 基础 UI
-> 本机运行验证
-> 与后端 HTTP API 联调
-> 与 WebSocket 联调
-> 根据真实页面再做视觉微调
```

暂不推荐一开始就做：

- 高保真 Figma 设计稿
- GPT-image 生成静态预览图
- 复杂动效、主题系统、响应式大改造

原因：

- 当前项目是功能演示优先，静态图不能验证接口、状态和实时通信。
- Figma 会增加从设计稿到代码的转换成本。
- 第一版 UI 的核心风险在登录态、接口字段、WebSocket 消息流和刷新恢复，而不是视觉细节。

## 3. 第一版 UI 范围

第一版只完成私聊演示闭环：

- 注册 / 登录
- 登录态保存与恢复
- 搜索用户
- 创建或进入私聊会话
- 展示会话列表
- 展示历史消息
- 通过 WebSocket 发送文本消息
- 接收 WebSocket 实时消息
- 标记会话已读
- 页面刷新后可恢复基础状态

暂不做：

- 群聊 UI
- 图片消息 UI
- 消息发送状态
- 在线状态展示
- 复杂自动重连策略
- 头像上传与个性化资料
- 移动端专项适配

## 4. 页面结构

### 4.1 登录页

文件位置：

```text
frontend/src/views/LoginView.vue
```

页面内容：

- 项目名：RustChat
- 登录表单：用户名、密码、登录按钮
- 注册入口：复用同一表单，提供注册按钮
- 错误提示：账号不存在、密码错误、用户名重复、网络错误
- 加载状态：登录中 / 注册中避免重复提交

交互规则：

- 登录成功后保存 token 和当前用户信息。
- 登录成功后跳转到 `/chat`。
- 已登录用户访问 `/login` 时，可自动跳转到 `/chat`。

### 4.2 聊天页

文件位置：

```text
frontend/src/views/ChatView.vue
```

页面布局：

```text
┌──────────────────────────────────────────────┐
│ 顶部栏：当前用户 / WebSocket 状态 / 退出       │
├───────────────┬──────────────────────────────┤
│ 左侧栏         │ 右侧聊天区                    │
│ - 用户搜索     │ - 会话标题                    │
│ - 会话列表     │ - 历史消息                    │
│               │ - 消息输入框                  │
└───────────────┴──────────────────────────────┘
```

左侧栏：

- 顶部用户搜索框
- 搜索结果列表
- 点击搜索结果创建私聊会话
- 会话列表展示会话名称、最近消息、时间、未读数

右侧聊天区：

- 无会话时显示空状态
- 有会话时展示当前会话名称
- 消息列表按时间展示
- 当前用户消息靠右，对方消息靠左
- 底部输入框发送文本消息

## 5. 组件拆分

按现有目录约定拆分：

```text
frontend/src/components/ConversationList.vue
frontend/src/components/UserSearch.vue
frontend/src/components/MessageList.vue
frontend/src/components/MessageInput.vue
```

组件职责：

- `UserSearch.vue`：输入关键字、调用用户搜索接口、展示结果、触发创建私聊。
- `ConversationList.vue`：展示会话列表、当前选中会话、未读数、最近消息。
- `MessageList.vue`：展示历史消息和实时消息，处理空状态。
- `MessageInput.vue`：输入文本消息，处理发送按钮、回车发送、空内容禁用。

页面级职责：

- `LoginView.vue`：注册、登录、跳转。
- `ChatView.vue`：组合聊天页组件，协调会话、消息和 WebSocket 状态。

## 6. Pinia 状态设计

### 6.1 auth store

文件位置：

```text
frontend/src/stores/auth.ts
```

建议状态：

```text
token
user
isAuthenticated
```

职责：

- 保存登录 token。
- 保存当前用户信息。
- 从 `localStorage` 恢复登录态。
- 退出登录时清理本地状态。

### 6.2 chat store

文件位置：

```text
frontend/src/stores/chat.ts
```

建议状态：

```text
conversations
activeSessionId
messagesBySessionId
loadingConversations
loadingMessages
```

职责：

- 加载会话列表。
- 切换当前会话。
- 加载历史消息。
- 接收新消息后更新当前会话消息列表。
- 标记会话已读后更新未读数。

### 6.3 connection store

文件位置：

```text
frontend/src/stores/connection.ts
```

建议状态：

```text
connected
connecting
lastError
```

职责：

- 管理 WebSocket 连接状态。
- 记录连接错误。
- 提供页面顶部连接状态展示。

## 7. API 接入顺序

第一轮按以下顺序接入，避免同时排查太多问题：

1. `POST /api/register`
2. `POST /api/login`
3. `GET /api/me`
4. `GET /api/users?keyword=<username>`
5. `POST /api/sessions/private`
6. `GET /api/conversations`
7. `GET /api/messages?session_id=<id>&limit=20`
8. `POST /api/sessions/<id>/read`
9. `GET /ws?token=<jwt>`

HTTP 请求统一放在：

```text
frontend/src/api/
```

WebSocket 连接统一放在：

```text
frontend/src/ws/client.ts
```

## 8. WebSocket 消息流

连接地址：

```text
ws://<backend-host>:3000/ws?token=<jwt>
```

发送文本消息：

```json
{
  "type": "send_message",
  "session_id": 12,
  "content": "hello"
}
```

客户端需要处理的事件：

- `message_sent`：当前用户发送成功后，追加到消息列表。
- `receive_message`：收到对方消息后，追加到对应会话消息列表，并更新会话列表最近消息。
- 错误事件：显示轻量错误提示。

第一版可先不做复杂重连。断开时只显示连接状态，并允许刷新页面恢复。

## 9. 样式方向

第一版视觉风格：

- 清爽、实用、偏桌面聊天工具。
- 左侧窄栏，右侧主聊天区域。
- 使用中性色背景，不做复杂装饰。
- 按钮、输入框和列表项保持明确 hover / active 状态。
- 未读数使用小徽标。
- 错误信息使用轻量提示，不做弹窗堆叠。

建议布局尺寸：

```text
页面最小高度：100vh
左侧栏宽度：280px - 320px
顶部栏高度：48px - 56px
消息输入区高度：64px - 88px
```

## 10. 实现步骤

### Step 1：登录页 UI

- 完成登录 / 注册表单。
- 接入 `register` 和 `login`。
- 登录成功保存 token。
- 跳转 `/chat`。

### Step 2：基础路由守卫

- 未登录访问 `/chat` 时跳转 `/login`。
- 已登录访问 `/login` 时跳转 `/chat`。
- 刷新页面时从 `localStorage` 恢复 token。

### Step 3：聊天页静态结构

- 完成顶部栏、左侧栏、右侧聊天区。
- 完成空状态、加载状态、错误状态。

### Step 4：会话与历史消息

- 接入用户搜索。
- 接入创建私聊。
- 接入会话列表。
- 切换会话时加载历史消息。
- 进入会话后调用已读接口。

### Step 5：WebSocket 联调

- 登录后建立 WebSocket 连接。
- 发送文本消息。
- 处理 `message_sent` 和 `receive_message`。
- 同步更新消息列表与会话列表。

### Step 6：体验微调

- 补充输入框禁用状态。
- 补充加载中状态。
- 补充错误提示。
- 补充滚动到底部。
- 补充基本响应式规则。

## 11. 第一版完成标准

- 用户可注册并登录。
- 登录成功后进入聊天页。
- 页面刷新后可恢复登录态。
- 可搜索用户并创建私聊。
- 可查看会话列表。
- 可进入会话查看历史消息。
- 可通过 WebSocket 发送文本消息。
- 对方在线时可实时接收文本消息。
- 进入会话后未读数可归零。
- 前端页面可在宿主机浏览器访问。

## 12. 后续可选增强

- Figma 高保真设计稿。
- 移动端布局。
- 群聊页面。
- 图片消息。
- 自动重连。
- 消息发送中 / 失败 / 重试状态。
- 在线状态。
- 头像与用户资料。

## 13. 当前交付状态与下一步

### 13.1 当前交付状态

前端第一版 UI 已落地：

- `LoginView.vue`：登录 / 注册表单、错误提示、加载状态、demo 跳过验证入口。
- `ChatView.vue`：顶部栏、连接状态、退出、左侧搜索与会话列表、右侧聊天区。
- `UserSearch.vue`：用户搜索与选择。
- `ConversationList.vue`：会话名称、最近消息、时间、未读数。
- `MessageList.vue`：历史消息与实时消息展示，当前用户消息靠右。
- `MessageInput.vue`：文本输入、回车发送、禁用状态。
- `auth / chat / connection` store：登录态、本地恢复、会话消息状态、WebSocket 状态。
- `frontend/src/api/`：已按后端真实路径接入 `/api/register`、`/api/login`、`/api/me`、`/api/users`、`/api/sessions/private`、`/api/conversations`、`/api/messages`、`/api/sessions/<id>/read`。
- `frontend/src/ws/client.ts`：已按 `GET /ws?token=<jwt>` 建立浏览器 WebSocket 客户端。

当前为了方便先查看页面，登录页提供了“跳过验证进入”按钮。

注意：

- 跳过验证只用于开发预览，不代表认证链路完成。
- 跳过模式不会自动请求受保护接口，也不会建立 WebSocket 连接。
- 后续真实联调完成后，应将该入口限制为开发环境可见，或直接移除。

### 13.2 下一步优先级

下一步不应该继续扩展 UI 功能，而应该先修通真实后端联调。

优先级顺序：

1. 修通注册 / 登录
2. 修通登录态恢复
3. 修通用户搜索和创建私聊
4. 修通会话列表与历史消息
5. 修通 WebSocket 发送和接收
6. 最后再做体验微调

原因：

- 当前 UI 结构已经足够验证第一版私聊闭环。
- 如果注册 / 登录失败，后续会话、消息、WebSocket 都无法稳定验证。
- 继续做视觉优化会掩盖真正的风险：接口地址、数据库状态、鉴权、CORS、WebSocket token。

### 13.3 认证链路排查顺序

建议先不用浏览器，从后端和 HTTP 接口开始排查。

1. 确认后端服务启动

```bash
curl http://127.0.0.1:3000/health
curl http://127.0.0.1:3000/db/ping
```

期望：

- `/health` 返回服务健康。
- `/db/ping` 返回数据库已连接。

2. 确认数据库 migration 已执行

重点确认以下表存在：

- `users`
- `sessions`
- `session_members`
- `messages`
- `user_session_read_state`

3. 直接验证注册接口

```bash
curl -X POST http://127.0.0.1:3000/api/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

4. 直接验证登录接口

```bash
curl -X POST http://127.0.0.1:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

5. 用登录返回的 token 验证 `/api/me`

```bash
curl http://127.0.0.1:3000/api/me \
  -H "Authorization: Bearer <token>"
```

如果以上 curl 都成功，再回到浏览器联调前端。

### 13.4 前端联调检查项

浏览器联调时优先检查：

- `frontend/.env` 中 `VITE_API_BASE_URL` 是否为真实后端地址。
- `frontend/.env` 中 `VITE_WS_BASE_URL` 是否为真实 WebSocket 地址。
- 如果前端运行在虚拟机，宿主机浏览器访问时不能使用虚拟机内部的 `127.0.0.1` 指向后端，应改为虚拟机 IP。
- 浏览器 DevTools Network 中注册 / 登录请求是否发到 `/api/register`、`/api/login`。
- 失败时响应内容是业务错误、网络错误、CORS 错误，还是连接失败。

常见判断：

- `401`：多半是 token 缺失、过期、解析失败，或请求没有带 `Authorization`。
- `409`：用户名已存在，换一个用户名即可。
- `503`：后端数据库未配置或不可连接。
- 浏览器 CORS 报错：需要后端允许前端源访问。
- `ERR_CONNECTION_REFUSED`：前端配置的后端地址不对，或后端没有启动。

### 13.5 WebSocket 验收顺序

认证链路修通后，再做 WebSocket：

1. 注册并登录两个用户，例如 `alice` 和 `bob`。
2. 用 `alice` 搜索 `bob` 并创建私聊。
3. 两个浏览器窗口分别登录两个用户。
4. 确认顶部连接状态为“实时已连接”。
5. `alice` 发送消息，确认自己窗口追加消息。
6. `bob` 窗口确认实时收到消息。
7. 刷新页面后进入同一会话，确认历史消息存在。
8. 进入会话后重新拉会话列表，确认未读数归零。

### 13.6 完成判定

完成下一步后，应满足：

- 不使用“跳过验证进入”也能注册 / 登录。
- 登录成功后进入 `/chat`。
- 刷新后仍保持登录态。
- 可搜索用户并创建私聊。
- 可查看会话列表和历史消息。
- 双浏览器窗口可实时互发文本消息。
- 进入会话后未读数归零。

以上完成后，才进入 `5.4 用户体验优化`，例如自动重连、发送状态、错误提示细化和移动端布局。
