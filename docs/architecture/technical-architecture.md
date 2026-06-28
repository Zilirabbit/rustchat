# RustChat 技术架构图

本文档用于展示 RustChat 当前项目的技术架构。图示仿照常见系统技术架构图的表达方式，将访问终端、前端服务、接入服务、后端服务、存储服务、实时连接、认证安全、配置日志和开发联调工具统一放在一张图中。

![RustChat 技术架构图](../img/rustchat-technical-architecture-v2.drawio.svg)

可编辑源文件：

- [draw.io SVG 版本](../img/rustchat-technical-architecture-v2.drawio.svg)

## 架构说明

- 访问终端：用户通过电脑浏览器或移动浏览器访问前端页面。
- 前端服务：Vue 3 SPA 负责界面展示，Pinia 管理登录态、会话、消息和连接状态，Axios 与 WebSocket client 分别对接 HTTP API 和实时连接。
- 接入服务：当前开发阶段以 Vite dev server 或静态资源托管提供前端资源；前端通过 HTTP 和 WebSocket 访问后端。
- 后端服务：Rust + axum + tokio 提供 HTTP API 与 WebSocket 服务，核心模块包括 user、session、conversation、message、file、connection、common 和 storage。
- 存储服务：PostgreSQL 保存用户、会话、成员、消息、已读状态和文件元信息；本地 `UPLOAD_DIR` 保存上传文件内容。
- 支撑能力：JWT / Argon2 提供认证与密码安全，Connection Manager 维护在线连接映射，`.env` 和 tracing 支撑配置与日志。
- 开发联调：Cargo test、SQLx migration、Postman、tracing 日志、summary 文档和浏览器开发者工具用于测试、联调、排障和过程沉淀。

当前版本以单后端服务、PostgreSQL 和本地文件存储完成基础聊天闭环。Redis、MQ、Nginx、对象存储、分布式锁和独立监控平台属于后续可扩展方向，不作为当前 v1 必要组件。
