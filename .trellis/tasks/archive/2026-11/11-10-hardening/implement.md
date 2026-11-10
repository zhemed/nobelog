# 执行计划：CI 门禁、基线还原与 panic 韧性

> 执行者：executor（实现，不提交）；验收者：verifier（只读复核 + 跑测试，不改代码）；Lead 终审并提交。
> 任务锚点：`[task:11-10-hardening]`。

## 步骤

### 1. 准备与基线记录

- [ ] `git status` 干净、`git log --oneline -1` 记录起点。
- [ ] 记录改动前二进制大小与 sha256：`ls -l rust/target/release/nobelog && sha256sum rust/target/release/nobelog`（预期 6,222,264 字节）。
- [ ] 确认 `public/` 基线存在（Lead 已于 2026-11-10 从 v0.0.37 Release 还原，42 个文件）；若缺失，先跑第 3 步脚本。

### 2. CI workflow

- [ ] 新建 `.github/workflows/ci.yml`，内容按 `design.md` §1：
  - 触发：push main / pull_request / workflow_dispatch；`permissions: contents: read`；concurrency 取消旧运行。
  - 步骤：checkout@v4 → cache（~/.rustup、~/.cargo）→ `rustup show` → `tools/check-rust.sh` → fmt → clippy `-D warnings` → test `--locked` → build `--release --locked` → 站点构建冒烟（`SITE_URL=https://example.com ... build --self-assets --out rust/dist`）。
  - 只用官方 action；不加 diff 步骤。
- [ ] 语法校验：`python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci.yml')); print('YAML OK')"`。
- [ ] 逐条在本地实跑 workflow 里的命令（与本地门禁相同），确认全部通过。

### 3. 基线还原脚本

- [ ] 新建 `tools/fetch-baseline.sh`（`chmod +x`），按 `design.md` §2：
  - `[tag]` 可选；默认取最新 release；`NOBELOG_REPO` / `NOBELOG_BASELINE_TAG` 可覆盖。
  - 下载 tarball + SHA256SUMS 到临时目录（trap 清理）→ 校验 sha256 → 解包 → 校验 VERSION → 覆盖 `public/` → 打印文件数与下一步 diff 命令。
  - 前置依赖检查与清晰错误信息；不执行 git 操作、不碰源码。
- [ ] 实测（需网络）：
  - `tools/fetch-baseline.sh v0.0.37` → 42 个文件，sha256 校验通过；
  - `SITE_URL=https://example.com ./rust/target/release/nobelog diff public rust/dist` → 41 相同 / 1 不同（首页 19→24 天前相对日期）；
  - 无参数调用（走 GitHub API 取 latest）也能工作；
  - 失败路径抽查：`tools/fetch-baseline.sh v9.9.9` 应非零退出且报错清晰。

### 4. panic 韧性

- [ ] `rust/Cargo.toml`：
  - `[profile.release]` 删除 `panic = "abort"`，加注释说明原因；保留 `lto`、`codegen-units`、`strip`。
  - `tower-http` features 追加 `"catch-panic"`。
- [ ] `rust/src/serve.rs`：
  - `use tower_http::catch_panic::CatchPanicLayer;`
  - 在 `run()` 里给统一 `app` 加最外层 `.layer(CatchPanicLayer::new())`（位置见 design §3）。
  - 不改其他逻辑；确认静态/动态两模式共用该 app。
- [ ] `cargo build --release --locked` 通过；记录新二进制大小与 sha256（与步骤 1 对比）。
- [ ] 手工验证 CatchPanicLayer 生效（不改仓库源码）：
  - 在 `/tmp` 复制一份仓库（或用独立小 demo）把某个 handler 改成 `panic!("probe")`，启动服务并请求，确认：请求返回 500、进程仍存活；验证后删除临时副本，仓库源码不得留下探针。
  - 若实现成本过高，可退化为：代码审查确认 layer 已挂 + `cargo test` 通过 + 在报告中说明未做运行时探针（由 verifier 复核）。
- [ ] 确认无新增 `unwrap()/expect()/panic!`（生产段）：`git diff -U0 | grep -E "^\+.*(unwrap\(\)|expect\(|panic!)"` 应无输出（测试/注释除外需说明）。

### 5. 文档

- [ ] `.trellis/spec/frontend/quality-guidelines.md`：
  - 修正「请求路径 panic 直接 500，且会 abort（release panic="abort"）」为 unwind + CatchPanicLayer 的真实行为；
  - 增加「CI 门禁」小节（workflow 跑哪些命令、本地仍需自跑）。
- [ ] `.trellis/spec/frontend/build-serve-deploy.md`：
  - 验证清单补「全新 clone 没有 public/ 时先 `tools/fetch-baseline.sh`」；
  - 发布/部署说明补 release 使用 unwind + strip（backtrace 用 debug 构建）。
- [ ] `README.md`：加 CI 徽章（`https://github.com/zhemed/nobelog/actions/workflows/ci.yml/badge.svg`）与「验证与基线」一段（fetch-baseline 用法 + CI 说明）。
- [ ] `.trellis/spec/frontend/directory-structure.md`：若列了根目录/工具，补 `.github/`、`tools/fetch-baseline.sh`。
- [ ] 全局复查：`grep -rn "panic=\"abort\"\|panic = \"abort\"" --include="*.md" . | grep -v archive` 应为空。

### 6. 全量门禁（实现完成前必跑）

```bash
cd rust && cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                     # 预期 98 passed
cargo build --release --locked
cd ..
SITE_URL=https://example.com ./rust/target/release/nobelog build --self-assets --out rust/dist
SITE_URL=https://example.com ./rust/target/release/nobelog diff public rust/dist   # 预期仅首页相对日期
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('YAML OK')"
bash -n tools/fetch-baseline.sh
```

- [ ] 以上全部通过，记录关键输出（测试数、diff 统计、二进制大小、sha256）。
- [ ] `git status --porcelain` 输出仅包含本任务预期文件；`git diff --stat` 记录改动范围。

## 审查门（verifier）

- [ ] 独立复核改动范围：仅 `.github/workflows/ci.yml`、`tools/fetch-baseline.sh`、`rust/Cargo.toml`、`rust/src/serve.rs`、README、`.trellis/spec/frontend/*`（+ 必要时 directory-structure）。
- [ ] 独立复跑第 6 步全部门禁命令。
- [ ] 检查 CI YAML：触发条件、权限、步骤顺序、无第三方 action、无 diff 步骤。
- [ ] 检查 fetch-baseline.sh：前置检查、sha256 校验、VERSION 校验、临时目录清理、只写 public/、失败路径。
- [ ] 检查 panic 改动：无 `panic="abort"`、feature 已加、CatchPanicLayer 在最外层且覆盖两模式、动态重建 JoinError 分支仍存在、无新增 unwrap/expect。
- [ ] 检查文档一致性：无「会 abort」旧表述；README/spec 描述与实际行为一致。
- [ ] 输出 PASS/FAIL 与证据（命令、输出摘要、行号引用）。

## 回退点

- CI/脚本：删除对应文件即可，无运行时影响。
- panic 改动：`git checkout -- rust/Cargo.toml rust/src/serve.rs` 回到 abort 行为（不推荐）。
- 文档：随对应提交 revert。
- 全程不推送远端、不发版、不改版本号。
