# Git 推送大文件失败问题记录

## 问题现象

执行推送 `dev` 分支时，GitHub 拒绝接收远程更新：

```text
remote: error: File backend/target/debug/backend is 102.40 MB; this exceeds GitHub's file size limit of 100.00 MB
remote: error: GH001: Large files detected.
! [remote rejected] dev -> dev (pre-receive hook declined)
error: failed to push some refs to 'github.com:Zilirabbit/rustchat.git'
```

## 具体原因

`backend/target/debug/backend` 是 Rust 本地编译生成的二进制产物，大小为 `102.40 MB`，超过 GitHub 普通 Git 仓库单文件 `100 MB` 的限制。

虽然后续提交中已经通过 `.gitignore` 和 `git rm --cached` 停止跟踪 `backend/target/` 下的产物，但问题文件仍存在于尚未推送的历史提交中。GitHub 检查的是整段待推送历史，不只是当前最新提交，所以仍然会拒绝推送。

本次待推送状态中，本地 `dev` 分支比 `origin/dev` 多多个提交，大文件出现在这些未推送提交的历史里。

## 排查过程

确认当前分支和远程跟踪关系：

```bash
git status --short --branch
git branch -vv
git remote -v
```

确认 SSH 认证正常：

```bash
ssh -T git@github.com
```

确认 dry-run 推送阶段能连接远程，但正式推送被 GitHub pre-receive hook 拒绝：

```bash
git push --dry-run origin dev
git push origin dev
```

查看大文件是否仍在当前索引中：

```bash
git ls-files backend/target/debug/backend backend/target/debug/backend.d
```

当前最新版本已经不再跟踪这些文件，但历史里仍包含它们。

查看相关历史：

```bash
git log --oneline -- backend/target/debug/backend backend/target/debug/backend.d
```

## 解决方式

先创建本地备份分支，保留清理前状态：

```bash
git branch backup_before_large_file_cleanup_20260509 dev
```

然后只重写尚未推送的 `origin/dev..dev` 这段历史，从这些提交中移除本地构建产物：

```bash
git filter-branch --force \
  --index-filter 'git rm -r --cached --ignore-unmatch backend/target/debug/backend backend/target/debug/backend.d' \
  --prune-empty \
  origin/dev..dev
```

清理后确认待推送历史中不再包含问题文件：

```bash
git rev-list --objects origin/dev..dev | rg 'backend/target/debug/backend(\.d)?$'
```

确认 dry-run 可推送：

```bash
git push --dry-run origin dev
```

最后推送成功：

```bash
git push origin dev
```

推送完成后，本地 `dev` 与远程 `origin/dev` 已同步：

```text
90814bf (HEAD -> dev, origin/dev) docs: add some details
```

## 注意事项

- `backend/target/`、`frontend/node_modules/`、`frontend/dist/` 这类本地生成目录不应提交到 Git。
- 其他通常不应提交的内容：
  - 本地环境变量：`.env`、`.env.local`、`.env.*`，但 `.env.example` 应提交，用于说明需要哪些配置项。
  - 编译和打包产物：`target/`、`dist/`、`.vite/`、`coverage/`。
  - 依赖安装目录：`node_modules/`。
  - 日志文件：`*.log`、`npm-debug.log*`、`yarn-debug.log*`、`pnpm-debug.log*`。
  - 编辑器和系统文件：`.vscode/`、`.idea/`、`.DS_Store`、`Thumbs.db`。
- 如果大文件已经进入尚未推送的提交历史，单纯在后续提交中删除它还不够，必须重写未推送历史。
- 如果大文件已经推送到远程，再重写历史会影响其他协作者，需要先沟通。
- 本次重写的是尚未推送到远程的本地提交，因此可以直接清理后正常推送。

## 关于 `.gitignore`

项目中的 `.gitignore` 已补充以下规则：

```text
target/
backend/target/
node_modules/
frontend/node_modules/
dist/
frontend/dist/
coverage/
.vite/
.env
.env.*
!.env.example
*.log
.vscode/
.idea/
.DS_Store
Thumbs.db
```

需要注意：`.gitignore` 只对“还没有被 Git 跟踪”的文件生效。

如果某个文件已经被提交过，即使后来把它写进 `.gitignore`，Git 仍然会继续跟踪它。此时需要手动从 Git 索引中移除：

```bash
git rm -r --cached backend/target
```

这个命令只会把文件从 Git 跟踪列表中移除，不会删除本地磁盘上的实际文件。
