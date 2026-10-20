# 默认维护 GitHub Releases 并补 v0.0.22 发布单

## Goal

按用户长期授权，把“发版默认更新 GitHub Release”固化为项目约定，并为 v0.0.22 补建 release
（notes + 二进制包 + SHA256SUMS 附件）。

## 背景

- 用户 2026-10-20 明确授权：「Releases 以后也默认更新吧，我授权了」→ 后续每次发版默认创建/更新 GitHub Release。
- 仓库 remote：`github.com/zhemed/nobelog`；`gh` 已登录（账号 zhemed，具备 repo 权限）。
- v0.0.22 已推 tag，但尚未建 GitHub Release。

## Requirements

- R1 为 `v0.0.22` 创建 GitHub Release：标题 `v0.0.22`，notes 覆盖本版 P1/P2/P3 修复与包 sha256，
  附件上传 `dist/nobelog-0.0.22-linux-x86_64.tar.gz` 与 `dist/SHA256SUMS`。
- R2 发布后用 `gh release view v0.0.22 --json ...` 核对 tag、assets、url。
- R3 把流程写进 `.trellis/spec/frontend/build-serve-deploy.md`：发版固定动作 = 版本同步 → `--locked` 构建 →
  `tools/package.sh` → 校验 sha256 → commit/tag push → **`gh release create`（默认）** → 部署。
- R4 不重新打包、不改代码、不重启服务（本轮已在线 v0.0.22）。

## Acceptance Criteria

- [ ] GitHub Release v0.0.22 存在，含 2 个附件与 notes；URL 可访问。
- [ ] spec 写明“默认更新 Release”，含 gh 命令模板。
- [ ] Trellis 记录完整（PRD/directive、journal、归档）。
