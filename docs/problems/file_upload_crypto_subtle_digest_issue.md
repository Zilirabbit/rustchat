# 文件上传 Web Crypto digest 不可用问题记录

## 问题现象

在宿主机浏览器通过 VM IP 访问前端并上传文件时，页面可能报错：

```text
Cannot read properties of undefined (reading 'digest')
```

当前已确认的触发入口：

```text
http://192.168.221.131:5173
```

## 触发路径

前端文件上传会在请求后端完成上传前计算文件 SHA256：

```text
frontend/src/api/files.ts -> computeSHA256 -> crypto.subtle.digest("SHA-256", buffer)
```

当浏览器环境中 `crypto.subtle` 为 `undefined` 时，调用 `.digest` 会直接抛出上述异常。

## 根因

`crypto.subtle` 属于浏览器 Web Crypto API 的安全上下文能力。通常在以下环境可用：

- `https://...`
- `http://localhost`
- `http://127.0.0.1`

但通过 VM IP 使用普通 HTTP 访问时，例如：

```text
http://192.168.221.131:5173
```

该页面不一定被浏览器视为安全上下文，因此部分浏览器不会提供 `crypto.subtle`。

## 影响范围

- 后端文件上传稳定 v1 链路不受影响。
- 受影响的是前端上传前的 SHA256 计算。
- 在 `crypto.subtle` 不可用的浏览器环境中，文件上传会在发起 `/api/files/init` 前失败。
- 当前错误信息是原始 JS 异常，不够适合作为用户提示。

## 临时规避方式

- 使用 `http://127.0.0.1:5173` 或 `http://localhost:5173` 访问前端。
- 使用 HTTPS 方式访问前端。
- 等待下一阶段补齐前端 SHA256 fallback。

## 下一阶段建议

建议前端计算 SHA256 时采用两级策略：

1. 优先使用原生 `globalThis.crypto?.subtle?.digest`。
2. 不可用时使用纯 JS SHA256 fallback，例如 `@noble/hashes`。

同时需要把原始异常转换为用户可读错误，避免页面直接暴露：

```text
Cannot read properties of undefined (reading 'digest')
```

建议用户提示：

```text
当前浏览器环境不支持原生文件校验，已使用兼容模式或请改用 HTTPS / localhost。
```

## 验收建议

下一阶段修复后至少验证：

- `http://127.0.0.1:5173` 上传小文件成功。
- `http://192.168.221.131:5173` 上传小文件成功。
- 禁用或模拟 `crypto.subtle` 不可用时仍能上传。
- fallback 也失败时展示业务错误，不再展示原始 `digest` 异常。
