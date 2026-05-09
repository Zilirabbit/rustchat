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

## 当前记录

### Backend

- [v0.1.0 - Phase 1 - 项目初始化](./backend/v0.1.0-phase1-project-init.md)
- [v0.1.0 - Phase 1 - common 基础模块](./backend/v0.1.0-phase1-common-module.md)
- [v0.1.0 - Phase 1 - storage 数据库模块](./backend/v0.1.0-phase1-storage-module.md)
- [v0.1.0 - Phase 1 - 数据库表设计](./backend/v0.1.0-phase1-db-schema.md)
- [v0.1.0 - Phase 1 - middleware 中间件](./backend/v0.1.0-phase1-middleware.md)
- [v0.1.0 - Phase 1 - connection WebSocket 基础](./backend/v0.1.0-phase1-connection.md)
- [v0.1.0 - Phase 1 - 私聊功能（message + session）](./backend/v0.1.0-phase1-private-message-session.md)
- [v0.1.0 - Phase 1 - 会话列表（conversation）](./backend/v0.1.0-phase1-conversation-list.md)
- [v0.1.0 - Phase 1 - 历史消息](./backend/v0.1.0-phase1-history-messages.md)
- [v0.1.0 - Phase 1 - 客户端联调前置接口收口](./backend/v0.1.0-phase1-client-preflight-apis.md)
- [v0.1.0 - Phase 1 - 真实数据库集成测试](./backend/v0.1.0-phase1-real-db-integration-tests.md)

### Frontend

- 暂无独立前端 summary。后续前端页面、状态管理、API 封装类总结放入 `frontend/`。

### Integration

- [v0.1.0 - Phase 1 - HTTP 接口联调](./integration/v0.1.0-phase1-http-api-integration.md)
