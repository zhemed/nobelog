# 加固：CI 门禁、基线还原与 panic 韧性

## Goal

按用户 2026-11-11「就听你的」授权，实施 Lead 评估建议中优先级最高的三项工程加固：

1. **CI 门禁**：新增 GitHub Actions workflow，把 fmt / clippy / test / release build / 站点构建冒烟固化到 push 与 PR 上，防止 agent 或人工漏跑门禁。
2. **基线可还原**：新增 `tools/fetch-baseline.sh`，从 GitHub Release 包还原 `public/` 冻结基线，使全新 clone 也能复现 `nobelog diff` 这个最强验收手段。
3. **panic 韧性**：release profile 去掉 `panic="abort"`，启用 `tower-http` 的 catch-panic，在 serve 路由边界兜底：单请求 panic 返回 500 而不是整个站点进程 abort；动态重建 panic 由 `JoinError` 捕获并退避。

## Requirements

- R1 CI：新增 `.github/workflows/ci.yml`，触发 `push`（main）、`pull_request`、`workflow_dispatch`；步骤覆盖：
  - `cargo fmt --manifest-path rust/Cargo.toml --check`
  - `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`
  - `cargo test --manifest-path rust/Cargo.toml --locked`
  - `cargo build --manifest-path rust/Cargo.toml --release --locked`
  - `SITE_URL=https://example.com ./rust/target/release/nobelog build --self-assets --out rust/dist`（站点构建冒烟）
  - `tools/check-rust.sh`（工具链版本自检）
  - 只用官方 action（`actions/checkout`、`actions/cache`），不引入第三方 action；`permissions` 最小化。
- R2 基线脚本：新增可执行 `tools/fetch-baseline.sh`：
  - 用法 `tools/fetch-baseline.sh [tag]`；不传 tag 时取 GitHub 最新 release；`NOBELOG_REPO` 可覆盖 owner/repo。
  - 下载 `nobelog-<version>-linux-x86_64.tar.gz` 与 `SHA256SUMS`，`sha256sum -c` 校验通过后解包，用包内 `public/` 覆盖仓库根的 `public/`。
  - 失败路径清晰报错（缺 curl/tar/sha256sum、release 不存在、校验失败）；只写 `public/`，不碰源码、不执行 git 操作。
  - 结束时打印还原文件数与下一步 `nobelog diff public rust/dist` 命令。
- R3 panic 韧性：
  - `rust/Cargo.toml` 的 `[profile.release]` 删除 `panic = "abort"`（回到默认 unwind），保留 `lto`、`codegen-units`、`strip`；加注释说明原因，防止未来被加回。
  - `tower-http` features 增加 `catch-panic`；`rust/src/serve.rs` 对静态/动态两种模式的路由统一挂 `CatchPanicLayer`（最外层），panic 返回 500。
  - 动态模式重建 panic 仍由现有 `spawn_blocking(...).await` 的 `JoinError` 分支捕获、记录日志并退避，进程继续用旧快照服务。
  - 生产代码不得新增 `unwrap()` / `expect()` / `panic!`。
- R4 文档同步：`.trellis/spec/frontend/quality-guidelines.md`（删除「panic 会 abort」的旧描述，补 CI 门禁说明）、`.trellis/spec/frontend/build-serve-deploy.md`（验证清单补基线还原）、`README.md`（基线与 CI 说明）、必要时 `.trellis/spec/frontend/directory-structure.md`。
- R5 不改动：模板输出、内容、`vendor/`、依赖版本（除 `tower-http` feature）、版本号、`public/` 基准内容、`.trellis` 任务结构；不拆 `serve.rs`；不引入 fixture / HTML 校验 / cargo-audit（留待后续）。

## Acceptance Criteria

- [x] `.github/workflows/ci.yml` 存在且 YAML 可被 `python3 -c "import yaml; yaml.safe_load(...)"` 解析；步骤与 R1 一一对应；不含第三方 action。
- [x] `tools/fetch-baseline.sh` 可执行；实测能从 Release 还原 `public/`（42 个文件，SHA256 校验通过）；`./rust/target/release/nobelog diff public rust/dist` 仅首页相对日期差异（与 2026-11-10 接手时结论一致）。
- [x] `rust/Cargo.toml` 的 release profile 不再有 `panic = "abort"`；`tower-http` features 含 `catch-panic`；`rust/src/serve.rs` 中两种模式的 Router 都挂了 `CatchPanicLayer`。
- [x] `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`（98 通过）、`cargo build --release --locked` 全绿。
- [x] 静态构建（`SITE_URL=https://example.com`）与 v0.0.37 基线 `diff` 仍只有相对日期差异；HTML 产物字节未因本次改动变化。
- [x] 文档不再出现「release panic=abort / panic 会 abort」的矛盾表述；README 与 spec 说明基线还原方式与 CI 门禁。
- [x] 任务归档并记录 journal，提交带任务锚点 `[task:11-10-hardening]`；不推送远端。

## Out of Scope

- 拆 `serve.rs`、HTML 校验、黄金 fixture、拆 `.trellis`（评估结论为低收益/不成立）。
- `cargo-audit`、dependabot 等依赖安全审计（可另立任务）。
- 版本号变更与发版、推送远端、部署方式变更。
- 为 `strip=true` 改回带符号（本次决定保留 strip，发布包体积优先；完整 backtrace 用本地 debug 构建）。

## Notes

- 技术设计与取舍见 `design.md`；执行清单与验证命令见 `implement.md`。
- 用户原话与边界见 `directive.md`。

## 验证结果（2026-11-10，verifier 独立复核）

- 验收结论：7 项全部 PASS，无 FAIL、无越界改动。完整报告见 `research/verification.md`。
- 改动范围：7 个已跟踪文件 +69/-5，新增 `.github/workflows/ci.yml`（66 行）与 `tools/fetch-baseline.sh`（107 行，755）；未改模板输出、内容、vendor、版本号。
- CI：PyYAML 解析通过；仅 `actions/checkout@v4` / `actions/cache@v4`；7 条步骤与 R1 一一对应；无 diff 步骤。
- 基线脚本：`v0.0.37` 与无参 latest 均还原 42 文件、sha256 通过；6 条失败分支均非零退出且 public/ 不变；只写 `public/`、无 git 操作。
- 门禁：fmt / clippy `-D warnings` / test（98 passed）/ build `--release --locked` / 站点构建（15 页 155509 字节）全绿；`diff public rust/dist` 41 同 / 1 不同（仅首页相对日期）。
- panic：release 二进制探针实测——请求 panic 返回 500、进程存活、后续请求 200；动态重建 panic 走 JoinError、日志退避、旧快照继续服务。
- 二进制：6,222,264 → 6,659,792 字节（+7.03%），sha256 `4d1eb5d3…`；`--locked` 重建后一致。
- 未推送远端、未发版、未改版本号（按授权边界）。
