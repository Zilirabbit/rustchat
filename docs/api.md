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

`docs/postman/rustchat.postman_collection.json`

导入方式：

1. 打开 Postman
2. 点击 `Import`
3. 选择 `docs/postman/rustchat.postman_collection.json`
4. 导入后修改 collection 变量：
   - `base_url`
   - `token`

建议：

- `base_url = http://127.0.0.1:3000`
- 远程虚拟机联调时改为 `http://<vm-ip>:3000`

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

## 5. 建议测试顺序

建议按下面顺序测试：

1. `GET /health`
2. `GET /db/ping`
3. `POST /api/register`
4. `POST /api/login`
5. 将 `data.token` 写入 `token` 变量
6. `GET /api/me`
7. 删除 `Authorization` 再测一次 `/api/me`
8. 将 token 改成非法值再测一次 `/api/me`

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
