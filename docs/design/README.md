# Rust Chat UI 生成说明

本目录保存 Rust Chat 前端 MongoDB 风格 UI 的视觉生成资料。

## 文件用途

- `PROMPT.md`：用于生成高保真桌面聊天 UI 的主提示词，英文部分可直接作为 image prompt。
- `DESIGN.md`：MongoDB 风格设计 token，包含颜色、字体、圆角、间距和组件规则。
- `../img/UI-preview.png`：参考图，用于约束三栏布局、白绿主题、消息区密度和整体视觉质感。

## 生成步骤

1. 读取 `PROMPT.md` 的英文部分作为基础生成提示。
2. 读取 `DESIGN.md`，将其中的 `canvas`、`surface`、`hairline`、`brand-green`、`brand-green-dark`、`rounded.md` 等 token 作为视觉约束。
3. 将参考图 `../img/UI-preview.png` 作为 image reference。
4. 追加项目约束：
   - 品牌名使用 `Rust Chat`。
   - 当前前端实现只展示已有接口能支撑的数据。
   - 不生成或实现缺少后端支撑的假频道、在线状态、reaction、link preview、typing indicator、固定房间统计。
   - 左侧展示真实 group/private 会话；中间展示真实消息；右侧仅在真实群聊选中时展示真实群成员。

## 推荐生成提示补充

```text
Use Rust Chat as the product name. Keep the MongoDB-inspired white and green light theme from DESIGN.md. Use the provided reference image only for layout and visual density. The final product UI must be grounded in real backend data: real group/private conversations, real messages, and real group members. Do not include fake community rooms, online members, reactions, link previews, typing indicators, or room statistics unless those APIs exist.
```

## 实现说明

当前代码实现不直接复刻 `PROMPT.md` 中所有静态 mock 内容，而是以可联调、可运行、真实数据最小展示为准。后续若补齐频道、在线状态、消息反应、链接预览等接口，再按 `PROMPT.md` 和参考图扩展 UI。
