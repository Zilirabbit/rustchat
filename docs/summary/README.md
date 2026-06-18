# Summary 记录规范

`docs/summary` 用于记录阶段性开发结果、关键变更和验证结果。

## 命名规则

建议使用下面的文件名格式：

```text
v<版本号>-<阶段>-<主题>.md
```

例如：

```text
v0.1.0-phase1-project-init.md
v0.2.0-phase1-common.md
```

## 分类规则

- `backend/`：后端模块、数据库、接口、服务端 WebSocket、后端测试相关总结
- `frontend/`：前端页面、状态管理、API 封装、UI 行为相关总结
- `integration/`：前后端联调、浏览器真实链路、Postman / curl 验证、双端 WebSocket 验收总结

## 如何查找

- 按能力演进查：优先看下面的里程碑阶段索引。
- 按代码域查：进入 `backend/`、`frontend/`、`integration/` 对应目录。
- 按提交查：每个阶段都标注了当前 `dev` 主线相关 commit；旧 backup / refs/original 中的重复提交不纳入主索引。

## 当前记录

### Phase 0 - 项目启动与环境准备

覆盖提交：`d81212c`、`8e0bfa1`、`50c8d3c`、`f998f22`

- `d81212c` 初始化仓库。
- `8e0bfa1` 完成后端工程初始化、基础模块骨架和 PostgreSQL 连通验证。
- `50c8d3c` 补充前端环境准备计划。
- `f998f22` 落地前端 Vite / Vue 工程、API 封装、状态管理和基础页面结构。

### Phase 1 - 后端基础设施

覆盖提交：`8e0bfa1`、`976c0c4`、`df69705`

- [v0.1.0 - Phase 1 - 项目初始化](./backend/v0.1.0-phase1-project-init.md)
- [v0.1.0 - Phase 1 - common 基础模块](./backend/v0.1.0-phase1-common-module.md)
- [v0.1.0 - Phase 1 - storage 数据库模块](./backend/v0.1.0-phase1-storage-module.md)
- [v0.1.0 - Phase 1 - 数据库表设计](./backend/v0.1.0-phase1-db-schema.md)
- [v0.1.0 - Phase 1 - middleware 认证与日志中间件](./backend/v0.1.0-phase1-middleware.md)

### Phase 2 - 实时私聊核心链路

覆盖提交：`6025990`、`192fb04`、`da1f154`

- [v0.1.0 - Phase 2 - connection WebSocket 基础](./backend/v0.1.0-phase2-connection.md)
- [v0.1.0 - Phase 2 - 私聊功能（message + session）](./backend/v0.1.0-phase2-private-message-session.md)
- [v0.1.0 - Phase 2 - 会话列表（conversation）](./backend/v0.1.0-phase2-conversation-list.md)
- [v0.1.0 - Phase 2 - 历史消息](./backend/v0.1.0-phase2-history-messages.md)

### Phase 3 - 会话体验与客户端前置接口

覆盖提交：`051fedc`、`df7f56c`、`1ba7155`

- [v0.1.0 - Phase 3 - 真实数据库私聊集成测试](./backend/v0.1.0-phase3-real-db-integration-tests.md)
- [v0.1.0 - Phase 3 - 客户端联调前置接口收口](./backend/v0.1.0-phase3-client-preflight-apis.md)
- [v0.1.0 - Phase 3 - 私聊唯一性约束](./backend/v0.1.0-phase3-private-session-uniqueness.md)

### Phase 4 - 前端 UI 与本地联调

覆盖提交：`90814bf`、`0496c3e`、`8383d7f`、`21fc585`、`3af1736`、`1ba7155`

- `90814bf` 补充环境与项目概览细节。
- `0496c3e` 实现前端聊天 UI，相关设计记录见 [frontend-ui-plan](../product/frontend-ui-plan.md)。
- `8383d7f` 记录大文件清理问题与处理方式。
- `21fc585` 将 summary 记录按 `backend/`、`frontend/`、`integration/` 目录归档。
- [v0.1.0 - Phase 4 - HTTP 接口联调](./integration/v0.1.0-phase4-http-api-integration.md)
- [v0.1.0 - Phase 4 - 刷新后恢复当前打开的会话](./frontend/v0.1.0-phase4-active-session-restore.md)

### Phase 5 - 集成修复、群聊与一致性补强

覆盖提交：`3af1736`、`6d1259f`、`1ba7155`

- `3af1736` 修复浏览器 HTTP API 联调链路中的 CORS、空会话已读和 5xx 排障能力。
- [v0.1.0 - Phase 5 - 群聊系统](./backend/v0.1.0-phase5-group-chat.md)
- [v0.1.0 - Phase 5 - 群聊与前端对齐](./frontend/v0.1.0-phase5-group-chat-frontend-alignment.md)
- [v0.1.1 - phase5.5.1/2 - 群聊成员列表与退出一致性](./frontend/v0.1.1-phase5.5.1-2-group-member-list-and-leave-consistency.md)
- [v0.1.1 - phase5.5.3 - WebSocket 自动重连与连接状态提示](./frontend/v0.1.1-phase5.5.3-websocket-reconnect.md)
- [v0.1.1 - phase5.5.4 - 消息发送体验](./frontend/v0.1.1-phase5.5.4-message-send-experience.md)
- 私聊一致性补强记录见 Phase 3 的“私聊唯一性约束”。
- 后续可继续放：双端 WebSocket 验收、未读/已读增强、部署记录。

### Phase 6 - 前端真实数据展示与设计资产

覆盖提交：`c662f65`

- [v0.1.0 - phase6 - 前端真实数据最小展示、CSS 重构与设计资产入库](./frontend/v0.1.0-phase6-real-data-ui-and-design-assets.md)
