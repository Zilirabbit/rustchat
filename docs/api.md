# RustChat API 文档

## 1. 说明

- 文档范围：当前仓库已实现并可用的 HTTP API
- 服务默认地址：`http://127.0.0.1:3000`
- 统一成功响应格式：

```json
{
  "code": 200,
  "message": "success message",
  "data": {}
}
```

- 统一错误响应格式：

```json
{
  "code": 401,
  "message": "error message"
}
```

## 2. Postman 导入

`api.md` 不能直接导入 Postman。

如需导入 Postman，请使用：

- `docs/postman/rustchat.postman_collection.json`
- `docs/postman/rustchat.websocket.postman_collection.json`

导入方式：

1. 打开 Postman
2. 点击 `Import`
3. 先导入 `docs/postman/rustchat.postman_collection.json`
4. 再导入 `docs/postman/rustchat.websocket.postman_collection.json`
5. 导入后修改 collection 变量：
   - HTTP collection：`base_url`
   - WebSocket collection：`ws_base_url`
6. 先执行 `POST /api/login`
7. `Login` 会自动把 JWT 写入 Postman 全局变量 `rustchat_token`

建议：

- `base_url = http://127.0.0.1:3000`
- `ws_base_url = 127.0.0.1:3000`
- 远程虚拟机联调时改为 `http://<vm-ip>:3000`
  - WebSocket 同时改为 `<vm-ip>:3000`

说明：

- 如果导入 WebSocket collection 后，请求页仍显示普通 HTTP 的 `Send` 按钮，而不是 WebSocket 的 `Connect`
- 说明该请求在当前 Postman 中没有被识别为真正的 WebSocket request
- 此时建议直接在 Postman 中通过 `New -> WebSocket` 手动创建连接请求，再保存到本地 collection

## 3. 认证说明

当前受保护接口使用 Bearer Token：

```text
Authorization: Bearer <token>
```

获取 token 的方式：

1. 先调用 `POST /api/register`
2. 再调用 `POST /api/login`
3. 从返回结果中的 `data.token` 取值
4. 填入 Postman collection 变量 `token`

## 4. 接口列表

### 4.1 健康检查

- 方法：`GET`
- 路径：`/health`
- 鉴权：否

示例请求：

```bash
curl http://127.0.0.1:3000/health
```

成功响应示例：

```json
{
  "code": 200,
  "message": "service is healthy",
  "data": {
    "status": "ok"
  }
}
```

### 4.2 数据库连通性检查

- 方法：`GET`
- 路径：`/db/ping`
- 鉴权：否

说明：

- 当 `DATABASE_URL` 已配置且数据库可用时返回 `200`
- 当未配置数据库时返回 `503`

示例请求：

```bash
curl http://127.0.0.1:3000/db/ping
```

成功响应示例：

```json
{
  "code": 200,
  "message": "database connected",
  "data": {
    "message": "database connected",
    "value": 1
  }
}
```

未配置数据库响应示例：

```json
{
  "code": 503,
  "message": "database is not configured"
}
```

### 4.3 用户注册

- 方法：`POST`
- 路径：`/api/register`
- 鉴权：否
- `Content-Type`：`application/json`

请求体：

```json
{
  "username": "alice",
  "password": "secret123"
}
```

字段约束：

- `username`：去除首尾空格后长度必须在 `3~32`
- `password`：长度必须在 `6~32`

示例请求：

```bash
curl -X POST http://127.0.0.1:3000/api/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

成功响应示例：

```json
{
  "code": 200,
  "message": "user registered",
  "data": {
    "user_id": 1,
    "username": "alice"
  }
}
```

可能错误：

- `400`：用户名或密码格式不合法
- `409`：用户名已存在
- `503`：数据库未配置

### 4.4 用户登录

- 方法：`POST`
- 路径：`/api/login`
- 鉴权：否
- `Content-Type`：`application/json`

请求体：

```json
{
  "username": "alice",
  "password": "secret123"
}
```

示例请求：

```bash
curl -X POST http://127.0.0.1:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

成功响应示例：

```json
{
  "code": 200,
  "message": "login succeeded",
  "data": {
    "token": "<jwt-token>",
    "user": {
      "user_id": 1,
      "username": "alice"
    }
  }
}
```

可能错误：

- `401`：用户名或密码错误
- `503`：数据库未配置

### 4.5 获取当前登录用户

- 方法：`GET`
- 路径：`/api/me`
- 鉴权：是

请求头：

```text
Authorization: Bearer <token>
```

示例请求：

```bash
curl http://127.0.0.1:3000/api/me \
  -H "Authorization: Bearer <token>"
```

成功响应示例：

```json
{
  "code": 200,
  "message": "current user fetched",
  "data": {
    "user_id": 1,
    "username": "alice"
  }
}
```

可能错误：

- `401`：缺少 token
- `401`：token 非法或已过期
- `401`：鉴权通过但用户不存在
- `503`：数据库未配置

### 4.6 搜索用户

- 方法：`GET`
- 路径：`/api/users`
- 鉴权：是

请求头：

```text
Authorization: Bearer <token>
```

查询参数：

- `keyword`：必填，按用户名模糊匹配，去除首尾空格后长度必须在 `1~32`

说明：

- 仅返回创建私聊所需的最小公开字段：`user_id`、`username`
- 不返回 `password_hash` 等敏感字段
- 当前实现会排除当前登录用户，避免前端误选自己创建私聊
- 单次最多返回 `20` 条

示例请求：

```bash
curl "http://127.0.0.1:3000/api/users?keyword=bo" \
  -H "Authorization: Bearer <token>"
```

成功响应示例：

```json
{
  "code": 200,
  "message": "users searched",
  "data": [
    {
      "user_id": 2,
      "username": "bob"
    }
  ]
}
```

可能错误：

- `400`：缺少 `keyword` 或查询参数不合法
- `401`：缺少 token 或 token 非法
- `503`：数据库未配置

### 4.7 创建私聊会话

- 方法：`POST`
- 路径：`/api/sessions/private`
- 鉴权：是
- `Content-Type`：`application/json`

请求头：

```text
Authorization: Bearer <token>
```

请求体：

```json
{
  "target_user_id": 2
}
```

说明：

- 如果当前用户和目标用户之间已存在私聊，则直接返回已有会话
- 如果不存在，则新建一条 `private` 会话和两条成员关系

示例请求：

```bash
curl -X POST http://127.0.0.1:3000/api/sessions/private \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"target_user_id":2}'
```

成功响应示例：

```json
{
  "code": 200,
  "message": "private session ready",
  "data": {
    "session_id": 12,
    "session_type": "private",
    "peer_user_id": 2,
    "created_at": "2026-05-03 12:00:00+00",
    "created": true
  }
}
```

可能错误：

- `400`：目标用户不存在
- `400`：不能给自己创建私聊
- `401`：缺少 token 或 token 非法
- `503`：数据库未配置

### 4.8 标记会话已读

- 方法：`POST`
- 路径：`/api/sessions/{session_id}/read`
- 鉴权：是
- 请求体：无

请求头：

```text
Authorization: Bearer <token>
```

说明：

- 仅会话成员可标记该会话已读
- 当前实现会将 `user_session_read_state.last_read_message_id` 更新为该会话当前的 `sessions.last_message_id`
- 空会话会写入 `last_read_message_id = null`，并更新 `last_read_at`
- 标记后再次查询 `GET /api/conversations`，该会话对当前用户的 `unread_count` 应为 `0`

示例请求：

```bash
curl -X POST http://127.0.0.1:3000/api/sessions/12/read \
  -H "Authorization: Bearer <token>"
```

成功响应示例：

```json
{
  "code": 200,
  "message": "session marked as read",
  "data": {
    "session_id": 12,
    "last_read_message_id": 90,
    "last_read_at": "2026-05-07 12:00:00+00"
  }
}
```

可能错误：

- `400`：`session_id` 不合法
- `401`：缺少 token 或 token 非法
- `403`：当前用户不是该会话成员
- `503`：数据库未配置

### 4.9 获取会话列表

- 方法：`GET`
- 路径：`/api/conversations`
- 鉴权：是

请求头：

```text
Authorization: Bearer <token>
```

说明：

- 仅返回当前用户已加入的会话
- 按最近消息时间倒序排列，无消息的会话排在最后
- `private` 会话的 `session_name` 为“对方用户名”
- `unread_count` 只统计“当前用户未读且不是自己发送”的消息

示例请求：

```bash
curl http://127.0.0.1:3000/api/conversations \
  -H "Authorization: Bearer <token>"
```

成功响应示例：

```json
{
  "code": 200,
  "message": "conversations fetched",
  "data": [
    {
      "session_id": 12,
      "session_type": "private",
      "session_name": "bob",
      "last_message": "hello",
      "last_message_time": "2026-05-03 12:00:00+00",
      "unread_count": 3
    },
    {
      "session_id": 7,
      "session_type": "group",
      "session_name": "team",
      "last_message": null,
      "last_message_time": null,
      "unread_count": 0
    }
  ]
}
```

可能错误：

- `401`：缺少 token 或 token 非法
- `503`：数据库未配置

### 4.10 查询历史消息

- 方法：`GET`
- 路径：`/api/messages`
- 鉴权：是

请求头：

```text
Authorization: Bearer <token>
```

查询参数：

- `session_id`：必填，会话 ID，必须为正整数
- `limit`：必填，单次返回数量，范围 `1~50`
- `before_message_id`：选填，游标；传入后只返回 `message_id < before_message_id` 的更早消息

说明：

- 仅会话成员可查询该会话历史消息
- 返回顺序固定为 `message_id` 倒序，即最新消息在前
- 若 `has_more = true`，下一页可继续传 `next_before_message_id`

示例请求：

```bash
curl "http://127.0.0.1:3000/api/messages?session_id=12&limit=20" \
  -H "Authorization: Bearer <token>"
```

继续查询更早消息：

```bash
curl "http://127.0.0.1:3000/api/messages?session_id=12&limit=20&before_message_id=88" \
  -H "Authorization: Bearer <token>"
```

成功响应示例：

```json
{
  "code": 200,
  "message": "history messages fetched",
  "data": {
    "session_id": 12,
    "limit": 20,
    "before_message_id": null,
    "next_before_message_id": 88,
    "has_more": true,
    "messages": [
      {
        "message_id": 90,
        "session_id": 12,
        "sender_id": 1,
        "sender_username": "alice",
        "message_type": "text",
        "content": "hello",
        "created_at": "2026-05-07 12:00:00+00"
      },
      {
        "message_id": 88,
        "session_id": 12,
        "sender_id": 2,
        "sender_username": "bob",
        "message_type": "text",
        "content": "hi",
        "created_at": "2026-05-07 11:59:00+00"
      }
    ]
  }
}
```

可能错误：

- `400`：缺少 `session_id / limit` 或分页参数不合法
- `401`：缺少 token 或 token 非法
- `403`：当前用户不是该会话成员
- `503`：数据库未配置

### 4.11 WebSocket 连接

- 方法：`GET`
- 路径：`/ws`
- 协议：`WebSocket`
- 鉴权：是

握手鉴权支持两种方式：

- `Authorization: Bearer <token>`
- 查询参数：`/ws?token=<token>`

说明：

- 若请求头中存在 `Authorization`，则优先按 Bearer Token 解析
- 若请求头中没有 `Authorization`，则回退到 `token` 查询参数
- 握手成功后，服务端会先主动推送一条 `connected` 事件

浏览器示例：

```js
const socket = new WebSocket("ws://127.0.0.1:3000/ws?token=<jwt-token>");
```

非浏览器客户端示例：

```text
GET /ws
Authorization: Bearer <jwt-token>
```

服务端首条消息示例：

```json
{
  "type": "connected",
  "user_id": 1,
  "username": "alice",
  "connection_id": 1
}
```

客户端可发送的协议：

```json
{
  "type": "ping"
}
```

```json
{
  "type": "send_message",
  "session_id": 12,
  "content": "hello"
}
```

服务端响应：

```json
{
  "type": "pong"
}
```

发送成功后，发送方会收到确认：

```json
{
  "type": "message_sent",
  "message": {
    "message_id": 1,
    "session_id": 12,
    "sender_id": 1,
    "sender_username": "alice",
    "content": "hello",
    "created_at": "2026-05-03 12:00:00+00"
  }
}
```

若接收方在线，接收方会收到实时推送：

```json
{
  "type": "receive_message",
  "message": {
    "message_id": 1,
    "session_id": 12,
    "sender_id": 1,
    "sender_username": "alice",
    "content": "hello",
    "created_at": "2026-05-03 12:00:00+00"
  }
}
```

非法消息响应示例：

```json
{
  "type": "error",
  "message": "invalid websocket message"
}
```

可能错误：

- `401`：缺少 token
- `401`：token 非法或已过期
- `403`：发送者不是该私聊成员
- `400` / 握手失败：请求头不满足 WebSocket upgrade 要求

## 5. 建议测试顺序

建议按下面顺序测试：

1. `GET /health`
2. `GET /db/ping`
3. `POST /api/register`
4. `POST /api/login`
5. 将 `data.token` 写入 `token` 变量
6. `GET /api/me`
7. `GET /api/users?keyword=<username>`
8. `POST /api/sessions/private`
9. 连接 `GET /ws?token=<jwt>`
10. 发送 `{"type":"send_message","session_id":<id>,"content":"hello"}`
11. `GET /api/conversations`
12. `GET /api/messages?session_id=<id>&limit=20`
13. `POST /api/sessions/<id>/read`
14. 再次 `GET /api/conversations` 验证 `unread_count`
15. 删除 `Authorization` 再测一次 `/api/me`
16. 将 token 改成非法值再测一次 `/api/me`

## 6. 维护约定

后续新增 API 时，建议同步维护两份文件：

- 人读文档：`docs/api.md`
- Postman 导入文件：`docs/postman/rustchat.postman_collection.json`

每次新增接口至少同步以下信息：

- 方法
- 路径
- 是否鉴权
- 请求参数
- 成功响应示例
- 典型错误响应
