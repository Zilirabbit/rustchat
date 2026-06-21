# RustChat 数据库设计

本文档对应 `task.md` 中的 `3.4 数据库表设计`，当前覆盖 Phase 1 所需的核心聊天表结构。

注意：

- 当前以 `backend/migrations/20260423010000_create_chat_core_tables.sql` 和后续增量 migration 作为正式来源
- 如果手动执行 SQL 创建过表，还需要保证 `_sqlx_migrations` 中存在对应版本记录，否则后续应用启动执行 migration 时会因对象已存在而失败

## 1. 设计目标

- 支持用户注册 / 登录
- 支持私聊与群聊会话
- 支持消息持久化与历史消息分页
- 支持会话列表的最近消息展示
- 支持未读数统计的基础状态表

## 2. 核心表

### 2.1 `users`

用途：

- 存储用户账号信息

关键字段：

- `id`：主键，`BIGSERIAL`
- `username`：用户名，大小写不敏感唯一
- `password_hash`：密码哈希
- `avatar_url`：头像地址，预留给前端展示
- `created_at / updated_at`：创建与更新时间

约束说明：

- 用户名长度限制为 `3 ~ 32`
- 用户名必须是裁剪后的值，避免首尾空格
- 使用 `LOWER(username)` 唯一索引，避免 `Alice` / `alice` 重复

### 2.2 `sessions`

用途：

- 表示聊天会话，兼容私聊与群聊

关键字段：

- `id`：主键
- `session_type`：`private` 或 `group`
- `name`：群聊名称；私聊为空
- `created_by`：创建者用户 ID
- `last_message_id / last_message_at`：最近一条消息信息，用于会话列表
- `created_at / updated_at`：创建与更新时间

约束说明：

- 私聊会话 `name` 必须为 `NULL`
- 群聊会话必须提供合法名称
- `last_message_id` 与 `last_message_at` 必须同时为空或同时存在
- `last_message_id` 通过外键保证属于当前会话

### 2.3 `session_members`

用途：

- 维护会话成员关系

关键字段：

- `session_id`：所属会话
- `user_id`：成员用户
- `role`：成员角色，当前支持 `owner / member`
- `joined_at`：加入时间

约束说明：

- `(session_id, user_id)` 唯一，防止重复入群/入会话
- 删除会话时级联删除成员关系

说明：

- `role` 虽然当前版本权限简单，但已为“基础群主模型”预留

### 2.4 `private_session_pairs`

用途：

- 为私聊建立“无序用户对 -> session”的唯一映射
- 防止同一对用户在并发创建时产生多条 private session

关键字段：

- `session_id`：私聊会话 ID，主键并关联 `sessions(id)`
- `user_low_id / user_high_id`：按大小排序后的两端用户 ID
- `created_at`：映射创建时间

约束说明：

- `(user_low_id, user_high_id)` 唯一，保证同一对用户只有一个私聊会话
- `user_low_id < user_high_id`，避免自己和自己建立私聊 pair
- 删除私聊 session 时级联删除对应 pair

迁移说明：

- `20260510010000_enforce_private_session_uniqueness.sql` 会先删除所有现有 `private` 会话
- 删除私聊时会级联清理对应私聊消息、成员关系与已读状态
- 群聊会话和群聊相关数据不受影响

### 2.5 `messages`

用途：

- 存储会话消息

关键字段：

- `session_id`：所属会话
- `sender_id`：发送者
- `message_type`：当前支持 `text / image / system`
- `content`：消息内容
- `created_at`：发送时间

约束说明：

- 内容不能为空白字符串
- `(session_id, id)` 额外唯一，用于和其他表建立“消息必须属于该会话”的复合外键

索引说明：

- `(session_id, id DESC)`：支持历史消息倒序分页
- `(sender_id, created_at DESC)`：支持发送者维度排查与扩展查询

### 2.6 `user_session_read_state`

用途：

- 记录用户在某会话中的已读进度

关键字段：

- `user_id + session_id`：联合主键，每个用户在每个会话只有一条状态
- `last_read_message_id`：最后已读消息
- `last_read_at`：最后已读时间
- `created_at / updated_at`：记录创建与更新时间

约束说明：

- `last_read_message_id` 通过复合外键保证属于当前会话

说明：

- 后续未读数可通过 `messages.id > last_read_message_id` 配合会话过滤统计
- 空会话或首次进入会话时，`last_read_message_id` 可以为空

## 3. 关键关系

- `sessions.created_by -> users.id`
- `private_session_pairs.session_id -> sessions.id`
- `private_session_pairs.user_low_id -> users.id`
- `private_session_pairs.user_high_id -> users.id`
- `session_members.session_id -> sessions.id`
- `session_members.user_id -> users.id`
- `messages.session_id -> sessions.id`
- `messages.sender_id -> users.id`
- `sessions.(id, last_message_id) -> messages.(session_id, id)`
- `user_session_read_state.(session_id, last_read_message_id) -> messages.(session_id, id)`

## 4. 索引与查询意图

- 用户登录：`users_username_lower_uidx`
- 私聊唯一性：`private_session_pairs_user_pair_uidx`
- 查询某用户会话列表：`session_members_user_id_idx`
- 查询最近活跃会话：`sessions_last_message_at_idx`
- 分页查询历史消息：`messages_session_id_id_desc_idx`
- 统计某会话成员已读状态：`user_session_read_state_session_id_idx`

## 5. 当前暂未在表层强约束的规则

- 一个私聊会话只能包含 2 名成员
- 消息发送者必须是会话成员

这些规则更适合在 `session / message service` 中结合业务流程校验，避免本阶段 migration 复杂度过高。
