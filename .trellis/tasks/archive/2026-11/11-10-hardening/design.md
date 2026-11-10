# 技术设计：CI 门禁、基线还原与 panic 韧性

## 0. 现状与约束

- 工具链：`rust-toolchain.toml`（仓库根）固定 `1.99.1` + `rustfmt` + `clippy`；本地已按系统级 `/opt/rust` 布局装好。
- 质量基线：`cargo fmt --check` / `cargo clippy --all-targets` / `cargo test`（98 通过）/ `cargo build --release --locked`，2026-11-10 接手时全绿。
- `public/` 是 `diff` 用的冻结基准，被 `.gitignore` 忽略，从未入库；全新 clone 没有 `public/`。
- `package.sh` 每次打包会用当前构建覆盖 `public/`，因此「基线 = 最近一次发布产物」，可从 GitHub Release 包恢复（2026-11-10 实测 v0.0.37 包还原后 diff 仅首页相对日期差异）。
- 当前 `[profile.release]`：`lto = true`、`codegen-units = 1`、`panic = "abort"`、`strip = true`；二进制 6,222,264 字节。
- 请求路径生产代码 0 个 `unwrap/expect/panic`（`content.rs` 3 处、`markdown.rs` 17 处均为静态正则初始化或逻辑安全写法）。

## 1. CI 门禁（.github/workflows/ci.yml）

### 触发与权限

```yaml
on:
  push:
    branches: [main]
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```

### 步骤（只用官方 action）

1. `actions/checkout@v4`
2. `actions/cache@v4` 缓存 `~/.rustup` 与 `~/.cargo`；key 含 `runner.os`、`rust-toolchain.toml` 与 `rust/Cargo.lock` 的 hash（不缓存 `rust/target`，体积过大）。
3. `rustup show`：runner 自带 rustup，仓库根的 `rust-toolchain.toml` 会让它自动安装 1.99.1（含 rustfmt/clippy）。
4. `tools/check-rust.sh`：确认实际 rustc/cargo 与 `rust-toolchain.toml` 一致。
5. `cargo fmt --manifest-path rust/Cargo.toml --check`
6. `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`
7. `cargo test --manifest-path rust/Cargo.toml --locked`
8. `cargo build --manifest-path rust/Cargo.toml --release --locked`
9. `SITE_URL=https://example.com ./rust/target/release/nobelog build --self-assets --out rust/dist`

### 明确不做的

- **不跑 `diff`**：冻结基准里首页卡片的相对日期（`N 天前`）随构建时刻漂移，CI 每天都会红；fixture/日期归一化留待后续任务。
- **不用第三方 action**（setup-rust-toolchain、rust-cache 等）：减少供应链面；官方 checkout/cache 足够。
- **不装 `cargo-audit`**：另立任务评估。

## 2. 基线还原脚本（tools/fetch-baseline.sh）

### 接口

```text
tools/fetch-baseline.sh [tag]
```

- `tag` 省略时通过 GitHub API `repos/<repo>/releases/latest` 取 `tag_name`。
- 环境变量 `NOBELOG_REPO`（默认 `zhemed/nobelog`）可覆盖仓库。
- 环境变量 `NOBELOG_BASELINE_TAG` 可作为默认 tag（可选，便于离线/固定版本）。

### 流程

1. 前置检查：`curl`、`tar`、`sha256sum`、`python3`（解析 JSON）。
2. 解析 tag；版本号 `ver=${tag#v}`；资产名 `nobelog-${ver}-linux-x86_64.tar.gz`。
3. 下载资产与 `SHA256SUMS` 到 `mktemp -d`（`trap` 清理）。
4. `sha256sum -c SHA256SUMS`（只校验该资产行）；失败即退出。
5. 解包，校验包内 `VERSION` 与 `ver` 一致（额外保险）。
6. 用包内 `public/` 覆盖仓库根 `public/`（`rm -rf public && cp -a`）。
7. 打印：还原文件数、基线 tag、下一步命令
   `SITE_URL=https://example.com ./rust/target/release/nobelog diff public rust/dist`。

### 边界

- 只写 `public/`；不 git add/commit、不碰源码与 `.trellis`。
- 网络不可用时报错并提示「可用已有 Release 包手工解包到 public/」。
- 覆盖前打印目标路径，避免误用（`public/` 是 gitignored 构建产物，覆盖是预期行为）。

## 3. panic 韧性

### Cargo profile

```toml
[profile.release]
lto = true
codegen-units = 1
# 不用 panic = "abort"：动态模式下一次 handler panic 不该让整站进程退出。
# unwind 下由 tower-http CatchPanicLayer 返回 500；完整 backtrace 用 debug 构建。
strip = true
```

- 保留 `strip = true`：发布包体积优先；panic 消息本身带 `file:line`，足够定位。需要完整 backtrace 时本地 `cargo build`（debug）复现。
- 预期二进制体积 +5%~15%（unwind 表），实测记录在验收证据里。

### 依赖与路由

- `tower-http` features：`["fs", "set-header", "compression-gzip", "compression-br", "catch-panic"]`。
- `rust/src/serve.rs`：

```rust
use tower_http::catch_panic::CatchPanicLayer;
...
let app = app
    .layer(middleware::from_fn(add_accept_encoding_vary))
    .layer(CompressionLayer::new().compress_when(should_compress_response))
    .layer(CatchPanicLayer::new()); // 最外层：handler/中间件 panic → 500，进程不退出
```

- 两种模式（静态/动态）共用同一个 `app` 变量，因此一处挂载即覆盖两者。
- 动态重建：`spawn_blocking(build_runtime).await` 的 `Err(JoinError)` 分支在 unwind 下真正生效（abort 时进程已死），记录日志、退避、继续用旧快照。
- 启动期 panic/Err 仍会退出进程（无快照可服务），由 systemd `Restart=always` 兜底；这是可接受行为。

### 行为矩阵（改后）

| 场景 | 改前（panic=abort） | 改后（unwind + CatchPanicLayer） |
| --- | --- | --- |
| 请求 handler panic | 进程 abort，全部连接断开 | 该请求 500，其余请求正常 |
| 动态重建 panic | 进程 abort | `JoinError` 捕获、日志、退避、旧快照继续服务 |
| 启动构建 panic/Err | 进程退出（systemd 重启循环） | 同左（无快照可服务） |
| 普通错误（内容 YAML 非法等） | Err 处理，退避 | 同左 |

## 4. 文档

- `quality-guidelines.md`：修正「请求路径 panic 会 abort」的表述；补 CI 门禁小节（push/PR 跑与本地相同的命令，本地仍需自跑）。
- `build-serve-deploy.md`：验证清单补「全新 clone 先 `tools/fetch-baseline.sh` 还原 `public/`」；发布/部署段落注明 release 用 unwind + strip。
- `README.md`：加 CI 徽章与「验证/基线」一段（fetch-baseline 用法）。
- `directory-structure.md`：如列出根目录/工具清单，补 `.github/` 与 `tools/fetch-baseline.sh`。

## 5. 风险与回退

- 风险：`catch-panic` 增加依赖 feature 与少量体积；`panic=unwind` 让二进制变大、编译稍慢；正常响应路径无行为变化。
- 风险：CI 首次 push 后才发现 YAML/工具链问题——本地先用 PyYAML 解析 + 逐条命令实跑；推送不在本任务范围，由用户决定。
- 风险：`fetch-baseline.sh` 依赖 GitHub 可达与 release 资产命名约定；命名变化时脚本报「资产不存在」。
- 回退：单文件回退即可（CI/脚本可删；Cargo profile 与 serve 改动 `git revert` 对应提交）。

## 6. 备选方案与取舍

- **黄金 fixture + cargo test**：能进 CI 防模板回归，但 crate 是 bin-only（需 `CARGO_BIN_EXE` 子进程）、相对日期/页脚年份需归一化，成本约半天；本轮不做，留作后续任务。
- **`strip = "debuginfo"` 或 `debug = 1`**：能保留符号/行号换 backtrace，但发布包变大；本轮决定体积优先，保留 strip。
- **`serve` 单独 profile**：Cargo 的 `panic` 是 profile 级设置，同一二进制含 build/serve/diff 子命令，无法按子命令区分；不可行。
