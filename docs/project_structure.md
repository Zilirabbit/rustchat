# RustChat 项目目录结构说明

## 1. 这份文件放在哪里

建议将“文件目录结构相关说明”放在：

```text
/docs/project-structure.md
```

原因：

- 它属于**项目级开发约定**，不是某一个业务模块（如 user / message / session）的功能说明。
- 它会同时影响后端目录、文档目录、模块拆分方式、命名规范、后续扩展方式。
- 这类内容适合作为团队或自己开发时的“统一参考约定”，应放在 `/docs` 根目录，而不是某个模块文档下。

如果后续文档继续增多，也可以改成：

```text
/docs/architecture/project-structure.md
```

但对你当前项目阶段，直接放在 `/docs/project-structure.md` 最实用。

---

## 2. 本文件的作用

本文件用于说明 RustChat 项目的：

- 项目根目录如何组织
- `/docs` 目录如何组织
- `backend/src` 如何分层与分模块
- 哪些内容放公共层，哪些内容放业务模块内
- 后续新增功能时如何扩展目录

它的定位不是产品需求文档，也不是任务清单，而是**代码与文档组织约定**。

---

## 3. 当前推荐的项目总目录

```text
rustchat/
├─ backend/
├─ frontend/
├─ docs/
│  ├─ spec.md
│  ├─ task.md
│  ├─ project-structure.md
│  ├─ modules/
│  │  ├─ user.md
│  │  ├─ session.md
│  │  ├─ message.md
│  │  ├─ connection.md
│  │  └─ conversation.md
│  └─ summary/
│     ├─ phase-1.md
│     └─ phase-2.md
└─ README.md
```

---

## 4. docs 目录说明

### 4.1 `spec.md`

用于描述：

- 项目目标
- 功能范围
- 核心业务规则
- 模块边界
- 验收标准

它回答的是：**项目要做什么，做到什么程度。**

### 4.2 `task.md`

用于描述：

- 里程碑
- 阶段任务
- 开发顺序
- 当前进度
- 测试与文档任务

它回答的是：**项目先做什么，后做什么，现在做到哪里。**

### 4.3 `project-structure.md`

用于描述：

- 目录结构
- 模块划分方式
- 代码分层约定
- 命名与组织原则

它回答的是：**代码和文档应该怎么放。**

### 4.4 `modules/*.md`

每个模块单独一个轻量文档，用于限制上下文。

例如：

- `modules/user.md`
- `modules/message.md`
- `modules/session.md`

这些文档只写该模块开发真正需要的信息，例如：

- 模块职责
- 涉及接口
- 涉及表
- 核心流程
- 业务规则
- 当前任务

这样每次开发一个功能时，只需读取：

- `spec.md`
- `task.md`
- 对应 `modules/*.md`

即可控制上下文范围。

---

## 5. backend 推荐目录结构

```text
backend/
├─ Cargo.toml
├─ .env
├─ .env.example
├─ migrations/
├─ src/
│  ├─ main.rs
│  ├─ app.rs
│  ├─ router.rs
│  │
│  ├─ common/
│  │  ├─ mod.rs
│  │  ├─ config.rs
│  │  ├─ error.rs
│  │  ├─ response.rs
│  │  ├─ pagination.rs
│  │  └─ utils.rs
│  │
│  ├─ middleware/
│  │  ├─ mod.rs
│  │  ├─ auth.rs
│  │  └─ logging.rs
│  │
│  ├─ storage/
│  │  ├─ mod.rs
│  │  ├─ db.rs
│  │  └─ transaction.rs
│  │
│  ├─ auth/
│  │  ├─ mod.rs
│  │  ├─ jwt.rs
│  │  ├─ password.rs
│  │  └─ types.rs
│  │
│  ├─ ws/
│  │  ├─ mod.rs
│  │  ├─ handler.rs
│  │  ├─ manager.rs
│  │  ├─ protocol.rs
│  │  └─ event.rs
│  │
│  ├─ user/
│  │  ├─ mod.rs
│  │  ├─ handler.rs
│  │  ├─ service.rs
│  │  ├─ repo.rs
│  │  ├─ dto.rs
│  │  ├─ model.rs
│  │  └─ routes.rs
│  │
│  ├─ session/
│  │  ├─ mod.rs
│  │  ├─ handler.rs
│  │  ├─ service.rs
│  │  ├─ repo.rs
│  │  ├─ dto.rs
│  │  ├─ model.rs
│  │  └─ routes.rs
│  │
│  ├─ message/
│  │  ├─ mod.rs
│  │  ├─ handler.rs
│  │  ├─ service.rs
│  │  ├─ repo.rs
│  │  ├─ dto.rs
│  │  ├─ model.rs
│  │  └─ routes.rs
│  │
│  └─ conversation/
│     ├─ mod.rs
│     ├─ handler.rs
│     ├─ service.rs
│     ├─ repo.rs
│     ├─ dto.rs
│     ├─ model.rs
│     └─ routes.rs
└─ tests/
```

---

## 6. frontend 推荐目录结构

前端计划先使用：

```text
Vue 3 + TypeScript + Vite
```

推荐目录结构：

```text
frontend/
├─ package.json
├─ index.html
├─ vite.config.ts
├─ tsconfig.json
├─ .env.example
├─ public/
└─ src/
   ├─ main.ts
   ├─ App.vue
   ├─ router/
   │  └─ index.ts
   ├─ stores/
   │  ├─ auth.ts
   │  ├─ chat.ts
   │  └─ connection.ts
   ├─ api/
   │  ├─ http.ts
   │  ├─ auth.ts
   │  ├─ users.ts
   │  ├─ sessions.ts
   │  ├─ conversations.ts
   │  └─ messages.ts
   ├─ ws/
   │  └─ client.ts
   ├─ views/
   │  ├─ LoginView.vue
   │  └─ ChatView.vue
   ├─ components/
   │  ├─ ConversationList.vue
   │  ├─ MessageList.vue
   │  ├─ MessageInput.vue
   │  └─ UserSearch.vue
   ├─ types/
   │  ├─ api.ts
   │  └─ chat.ts
   └─ styles/
      └─ main.css
```

### 6.1 前端目录职责

- `router/`：页面路由，例如登录页与聊天主页面
- `stores/`：Pinia 状态，例如登录态、当前会话、WebSocket 连接状态
- `api/`：HTTP API 封装，对应后端 `auth / users / sessions / conversations / messages`
- `ws/`：WebSocket 连接、事件解析、消息发送封装
- `views/`：页面级组件
- `components/`：可复用 UI 组件
- `types/`：前端请求、响应、聊天事件类型
- `styles/`：全局样式

### 6.2 前端环境变量约定

建议在 `frontend/.env.example` 中预留：

```text
VITE_API_BASE_URL=http://127.0.0.1:3000
VITE_WS_BASE_URL=ws://127.0.0.1:3000
```

在 Ubuntu 虚拟机 + Windows 宿主机浏览器访问场景中，应改为：

```text
VITE_API_BASE_URL=http://<虚拟机IP>:3000
VITE_WS_BASE_URL=ws://<虚拟机IP>:3000
```

### 6.3 当前阶段前端边界

第一版前端只要求完成私聊演示闭环：

- 注册 / 登录
- 搜索用户并创建私聊
- 会话列表
- 历史消息
- WebSocket 发送与接收文本消息
- 标记会话已读
- 刷新页面后恢复登录态并重新拉取会话与历史消息

群聊、图片消息、自动重连、消息发送状态、在线状态展示可以后置。

---

## 7. 为什么不用全局 `dto/`、`model/`、`service/` 目录

不建议这样：

```text
src/
├─ dto/
├─ model/
├─ service/
├─ repo/
└─ ...
```

原因：

### 7.1 不利于按功能阅读

如果做用户登录功能，你就要分别查看：

- `dto/user.rs`
- `model/user.rs`
- `service/user.rs`
- `repo/user.rs`
- `handler/user.rs`

文件分散，不利于“每次只读取一个功能的相关上下文”。

### 7.2 小项目容易越写越乱

随着功能增多，会出现大量：

- `dto/user.rs`
- `dto/message.rs`
- `dto/session.rs`

这些都堆在一起，管理成本会越来越高。

### 7.3 按模块聚合更适合当前项目

RustChat 当前更适合：

- 以业务模块为主
- 模块内再细分 handler / service / repo / dto / model

这样开发时上下文更集中。

---

## 8. 哪些东西应该全局统一

以下内容建议统一放公共层：

### 8.1 `common/`

放真正跨模块复用的内容：

- `config.rs`：配置读取
- `error.rs`：统一错误类型
- `response.rs`：统一返回结构
- `pagination.rs`：分页参数与分页返回
- `utils.rs`：真正通用的辅助函数

### 8.2 `auth/`

放认证相关公共能力：

- JWT
- 密码加密
- Claims / 当前用户上下文类型

### 8.3 `storage/`

放数据库基础设施：

- 数据库连接池
- 事务封装
- 共享 DB 类型

### 8.4 `ws/`

放 WebSocket 相关公共能力：

- WS 连接入口
- 连接管理
- 协议定义
- 事件分发

---

## 9. 哪些东西应该放模块内

以下内容更适合放在业务模块内部：

- `user/dto.rs`
- `user/model.rs`
- `user/repo.rs`
- `user/service.rs`
- `user/handler.rs`

其他模块同理。

原则是：

> 只服务某个业务模块的结构和逻辑，就放在该模块内部。

例如：

- `RegisterRequest` → 放 `user/dto.rs`
- `CreatePrivateSessionRequest` → 放 `session/dto.rs`
- `Message` → 放 `message/model.rs`
- `ConversationItem` → 放 `conversation/dto.rs` 或 `conversation/model.rs`

---

## 10. 关于 `utils` 的约定

`utils` 最容易变成杂物箱，因此要严格控制。

### 可以放进 `common/utils.rs` 的

- 时间格式化辅助
- 随机字符串生成
- 通用脱敏函数
- 通用校验辅助

### 不要放进 `utils` 的

- JWT 逻辑 → 放 `auth/jwt.rs`
- 密码加密 → 放 `auth/password.rs`
- WebSocket 消息协议 → 放 `ws/protocol.rs`
- 会话相关逻辑 → 放 `session/service.rs`
- 消息发送逻辑 → 放 `message/service.rs`

原则：

> 能归属到具体领域的代码，不放 `utils`。

---

## 11. 关于 `model` 和 `dto` 的约定

为了避免后续不统一，建议固定以下规则：

### 11.1 `model.rs`

用于放：

- 数据库实体
- 领域实体

例如：

- `User`
- `Session`
- `Message`

### 11.2 `dto.rs`

用于放：

- 请求参数
- 查询参数
- 返回数据结构

例如：

- `RegisterRequest`
- `LoginRequest`
- `CreateGroupSessionRequest`
- `MessageListQuery`

### 11.3 不要混用

不要把：

- HTTP 请求体
- API 返回结构
- 数据库实体

全部混在 `model.rs` 里。

推荐模式：

- 数据库存储对象 → `model.rs`
- API 输入输出对象 → `dto.rs`

---

## 12. 每个模块的统一结构建议

建议所有业务模块尽量使用同样的文件模式：

```text
xxx/
├─ mod.rs
├─ handler.rs
├─ service.rs
├─ repo.rs
├─ dto.rs
├─ model.rs
└─ routes.rs
```

职责约定：

- `handler.rs`：接收 HTTP 请求，调用 service，返回响应
- `service.rs`：编排业务逻辑
- `repo.rs`：数据库访问
- `dto.rs`：请求/响应结构
- `model.rs`：数据库实体/领域实体
- `routes.rs`：本模块路由注册

这样不是内容完全一致，而是**组织方式统一**。

---

## 13. 什么时候再拆更多文档

当前不建议一次性增加过多文档。

后续可以在需要时再新增：

- `docs/db.md`：当数据库设计开始稳定
- `docs/api.md`：当前后端接口开始联调
- `docs/websocket.md`：当 WS 协议开始固定
- `docs/architecture.md`：当项目结构复杂度上升

原则：

> 如果一份文档不能明显减少某次开发的上下文，就先不要建。

---

## 14. 当前阶段的推荐落地方案

### 文档层

现在先保留：

- `docs/spec.md`
- `docs/task.md`
- `docs/project-structure.md`
- `docs/modules/*.md`

### 后端层

采用：

- 按业务模块聚合目录
- 模块内统一拆分 handler / service / repo / dto / model
- 公共能力统一收敛到 `common/`、`auth/`、`storage/`、`ws/`

### 开发时读取建议

例如：

#### 做用户登录
读取：

- `docs/spec.md`
- `docs/task.md`
- `docs/modules/user.md`
- `docs/project-structure.md`

#### 做私聊发消息
读取：

- `docs/spec.md`
- `docs/task.md`
- `docs/modules/session.md`
- `docs/modules/message.md`
- `docs/modules/connection.md`
- `docs/project-structure.md`

这样既能限制上下文，又能保持目录和代码风格统一。

---

## 15. 一页约定（建议长期保持）

1. 按业务模块组织代码，不按全局 DTO/MODEL/SERVICE 横切。
2. 每个业务模块优先保持统一结构：`handler.rs / service.rs / repo.rs / dto.rs / model.rs / routes.rs`。
3. 全局通用能力放在 `common/`、`auth/`、`storage/`、`ws/`。
4. 请求/响应结构放 `dto.rs`，数据库实体放 `model.rs`。
5. 不允许创建含义模糊的 `misc.rs`、`helper.rs`、`temp.rs`。
6. 能归属于某个领域的逻辑，不要放进 `utils`。
7. 模块文档用于控制上下文，总文档用于保证全局一致性。

---

## 16. 最终建议

对于当前 RustChat 项目：

- “文件目录结构相关说明”最适合单独放在 `/docs/project-structure.md`
- 它应作为项目级约定文档长期维护
- 后端目录采用“按业务模块聚合”的方式，而不是全局按 DTO/MODEL/SERVICE 横切
- 真正的统一性来自**约定统一**，不是把所有文件堆到同一个目录
