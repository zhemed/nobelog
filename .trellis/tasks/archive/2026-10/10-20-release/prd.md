# 发布 v0.0.22

## Goal

按项目现有发布方式（annotated tag + `tools/package.sh` 产出的 tar.gz + 推送 tag）发布 v0.0.22，
并顺手修掉"版本三源不一致"（Cargo.toml / package.json / tag）。

## 背景

- 现有 tag：`v0.0.21`（annotated，message 形如 `0.0.21：简述`），dist 里有 0.0.7–0.0.21 的包。
- 代码里 `rust/Cargo.toml` 与 `package.json` 仍是 `0.1.0`，与 tag 不一致。
- 本版本包含：审查后的 P1（路径穿越/头注入/RSS 时区）、P2（CSS 确定性、热更新、内容校验）、P3（符号链接、.mdx、保留名、updated）。

## Requirements

- R1 `rust/Cargo.toml`、`package.json`、`rust/Cargo.lock` 的 nobelog 版本统一为 `0.0.22`。
- R2 `VERSION=0.0.22 tools/package.sh` 产出 `dist/nobelog-0.0.22-linux-x86_64.tar.gz` 与 `dist/SHA256SUMS`。
- R3 包内容核对：`nobelog` 二进制（`--version` = 0.0.22）、`content/`、`vendor/`、`public/`、
  `rust/client/main.js`、`README.md`、`LICENSE`、`VERSION`（内容为 `0.0.22`）。
- R4 创建 annotated tag `v0.0.22`（message：`0.0.22：审查修复（安全/渲染/构建）`），推送提交与 tag。
- R5 不部署（线上服务是否重启另行确认）；不改其它行为。

## Acceptance Criteria

- [ ] 版本三源一致，`cargo build --release --locked` 通过。
- [ ] 发布包生成且 SHA256SUMS 校验通过；包内 `./nobelog --version` 输出含 0.0.22。
- [ ] 提交 + tag 已推送，`git status` 干净，远程可见 `v0.0.22`。
- [ ] Trellis 记录完整（PRD/directive、journal、归档）。
