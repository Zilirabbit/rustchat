# Postman Collections

当前目录拆成两份 collection：

- `rustchat.postman_collection.json`
  - 仅包含 HTTP 请求
  - 已按 `System / Auth / Dual User Auth / Session` 分组
- `rustchat.websocket.postman_collection.json`
  - 仅包含 WebSocket 请求
  - 已按 `Connection / Private Chat` 分组

这样拆分是为了兼容 Postman 对 WebSocket collection 的单独限制，避免把 HTTP 与 WebSocket 混在同一个 collection 后出现导入或使用异常。

注意：

- 如果直接导入 `rustchat.websocket.postman_collection.json` 后，界面仍显示普通 HTTP 的 `GET` 和 `Send`，而不是 WebSocket 的 `Connect`
- 说明当前 Postman 没有把这条请求识别成真正的 WebSocket request
- 这时不要继续用导入结果验证，请改为在 Postman 里手动新建 WebSocket request

## 推荐导入方式

1. 先导入 `rustchat.postman_collection.json`
2. 如果只验证单用户链路，先跑通：
   - `Register`
   - `Login`
3. 确认 `Login` 已把 JWT 自动写入 Postman 全局变量 `rustchat_token`
4. 如果要验证私聊，推荐直接使用双用户分组：
   - `Register Alice`
   - `Register Bob`
   - `Login Alice`
   - `Login Bob`
5. 再导入 `rustchat.websocket.postman_collection.json`
6. 根据环境修改：
   - `base_url = http://127.0.0.1:3000` 或 `http://<vm-ip>:3000`
   - `ws_base_url = 127.0.0.1:3000` 或 `<vm-ip>:3000`

如果你没有先跑 `Login`，也可以手动设置一个 Postman 全局变量：

- `rustchat_token = <jwt-token>`

如果你使用双用户私聊联调，请确认以下全局变量已存在：

- `rustchat_alice_token`
- `rustchat_bob_token`
- `rustchat_alice_user_id`
- `rustchat_bob_user_id`
- `rustchat_private_session_id`

## WebSocket 验证建议

- 推荐方式：手动新建 WebSocket request，而不是依赖 JSON 导入结果
- 优先使用 `Connect With Query Token`
- 连接成功后应先收到：

```json
{
  "type": "connected",
  "user_id": 1,
  "username": "alice",
  "connection_id": 1
}
```

- 然后发送：

```json
{
  "type": "ping"
}
```

- 期望收到：

```json
{
  "type": "pong"
}
```

## 私聊双用户验证建议

推荐使用 collection 中已经准备好的请求顺序：

1. `Dual User Auth -> Register Alice`
2. `Dual User Auth -> Register Bob`
3. `Dual User Auth -> Login Alice`
4. `Dual User Auth -> Login Bob`
5. `Session -> Create Private Session Alice -> Bob`
   这一步会自动写入 `rustchat_private_session_id`
6. `RustChat WebSocket -> Private Chat -> Alice`
7. `RustChat WebSocket -> Private Chat -> Bob`
8. 在 `Alice` 连接中发送：

```json
{
  "type": "send_message",
  "session_id": {{rustchat_private_session_id}},
  "content": "hello"
}
```

期望结果：

- `Alice` 收到 `message_sent`
- `Bob` 收到 `receive_message`

## 手动新建 WebSocket Request

根据 Postman 官方文档，最稳的方式是：

1. 点击 `New`
2. 选择 `WebSocket`
3. 输入：

```text
ws://<vm-ip>:3000/ws?token=<jwt-token>
```

4. 点击 `Connect`

如果你看到的是 `Send`，不是 `Connect`，说明当前标签页仍然不是 WebSocket request。
