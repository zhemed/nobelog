# 验收报告（verifier 独立复核）

日期：2026-11-10（UTC）/ 2026-11-11（Asia/Shanghai）
验收者：verifier（只读 + 跑测试/脚本，未改任何代码/文档/配置，未执行 git 写操作）
结论：**7 项全部 PASS，无 FAIL、无越界改动。**

## 1 改动范围 — PASS

- `git status --porcelain`：7 个 M + 3 个 ??，全部在预期内：
  - M：`.trellis/spec/frontend/{build-serve-deploy,directory-structure,quality-guidelines}.md`、`README.md`、`rust/Cargo.lock`、`rust/Cargo.toml`、`rust/src/serve.rs`
  - ??：`.github/`（仅 `workflows/ci.yml`）、`tools/fetch-baseline.sh`（755）、`.trellis/tasks/11-10-hardening/`（规划文档）
- `git diff --stat`：7 files, +69/-5；`git diff --check` rc=0。
- 探针残留：grep `__panic_probe|panic-marker|catch-panic probe`（排除 .git/target）无命中。
- 未改模板/内容/vendor/版本号：`git diff --name-only` 无 `content/`、`vendor/`、`templates.rs`；版本 0.0.37 未变；无新增 TODO/FIXME/dbg!。

## 2 CI YAML — PASS

- PyYAML 解析 OK；触发 push main / pull_request / workflow_dispatch（ci.yml:12-16）；`permissions: contents: read`（18-19）；concurrency 取消旧运行（21-23）。
- action 仅 `actions/checkout@v4`（35）与 `actions/cache@v4`（38）两个官方 action。
- cache key 含 `runner.os` + `hashFiles('rust-toolchain.toml','rust/Cargo.lock')`（40-45）。
- 步骤：rustup show(48) → `tools/check-rust.sh`(51) → fmt(54) → clippy `-D warnings`(57) → test `--locked`(60) → build `--release --locked`(63) → 站点构建冒烟(66)；无 diff 步骤（注释 8 行说明原因）；timeout 30min。

## 3 fetch-baseline.sh — PASS

- 静态：755；`bash -n` rc=0；`set -euo pipefail`；前置检查 curl/tar/sha256sum；python3 仅 latest 路径需要；mktemp + trap 清理；grep 无任何 git 命令。
- 实测：`v0.0.37` → sha256 OK、42 文件、rc=0；无参 latest → v0.0.37、42 文件；`NOBELOG_BASELINE_TAG` 生效；`NOBELOG_REPO` 覆盖生效；`v9.9.9`/多余参数/非法 tag 均 rc=1 且报错清晰；`--help` rc=0。
- 假 curl 验证 6 条失败分支（SHA256SUMS 无该资产、sha 不匹配、VERSION 不一致、缺 public/、API 非法 JSON、API 无 tag_name），每例 rc=1 且 public/ UNCHANGED、无残留临时目录；python 管道错误未被 pipefail 吞掉。
- 只写 public/ 实证：全仓快照 sha256 运行前后逐字节相同；git status 不变；从 /tmp 调用正常。

## 4 panic 改动 — PASS

- Cargo.toml：tower-http features 含 `catch-panic`（29）；`[profile.release]` 无 panic 键、保留 lto/codegen-units/strip，含防回退注释（32-39）。
- Cargo.lock：仅 +1 行 `"tracing"` 挂在 tower-http 依赖块（1281）；tracing 0.1.44 原已存在，无新包/无版本变动；依据 tower-http 0.6.11 `catch-panic = ["tracing"]`。
- serve.rs：:33 引入 CatchPanicLayer；:303-309 `CatchPanicLayer::new()` 为最后 `.layer` = 最外层；静态(:300)与动态(:286)共用同一 `app`；动态重建 JoinError 分支仍在（:214、:231-235）；`git diff -U0 -- rust/src rust/Cargo.toml` 无新增 unwrap/expect/panic。
- 运行时探针（/tmp 副本，仓库零改动）：release 二进制 × 静态/动态两模式，`/` 200 → 探针路径 **500** → 再 `/` 200，进程存活；日志含 `panicked at src/serve.rs:xxx`。动态重建注入 panic → 日志「动态模式重建任务异常，继续使用旧快照」+ 退避，后续请求 200、进程存活。

## 5 门禁复跑 — PASS

- `rustup show` / `tools/check-rust.sh`（1.99.1）/ fmt / clippy `-D warnings` / test `--locked`（**98 passed**）/ build `--release --locked` / 站点构建（15 页 155509 字节）/ `bash -n` 全绿。
- `nobelog diff public rust/dist` → **41 相同 / 1 不同 / 0 缺失 / 0 多出**，仅 `index.html:115` 相对日期 `19 天前` vs `24 天前`，rc=1 属预期；public 与 rust/dist 各 42 文件。
- 模拟全新 clone（无 public/、无 rust/dist）：站点构建 rc=0、输出 42 文件 → CI 冒烟步骤在 fresh clone 可用。

## 6 文档一致性 — PASS

- 全仓无「panic 会 abort」旧表述；命中仅规划文档（描述改前/决策）与防回退表述。
- README 徽章 URL 与脚本默认仓库一致；「验证与基线」与实测行为一致。
- quality-guidelines CI 小节与 ci.yml 步骤逐条对应；build-serve-deploy 补 fetch-baseline 与 unwind+strip；directory-structure 补两个新文件。
- 二进制体积：改前 6,222,264（记录于 design.md:9）→ 改后 6,659,792（+437,528，+7.03%，在预期 +5%~15% 内）。

## 7 额外 — PASS

- 二进制 sha256 改后 `4d1eb5d3b99465a39e6f669706ef5dee94bf867bbac5db804b05bc81b21aab00` / 6,659,792 字节 == 实测且 `--locked` 重建后不变；改前 sha 因覆盖无法复算，属预期。
- CI 命令与本地实跑语义等价（CI 用 `--manifest-path`，README 用 `cd rust`）。

## 风险/备注（非 FAIL）

1. CI 首跑无法本地验证：`rustup show` 在 ubuntu-latest 按 `rust-toolchain.toml` 自动装 1.99.1 属设计假设；若失败，最小补救是显式 `rustup toolchain install 1.99.1 --component rustfmt --component clippy`。
2. `actions/cache` 缓存整个 `~/.cargo`（含 bin），属常见做法；key 失效逻辑正确。
3. README CI 徽章在首次 push 前会 404（预期）。
4. `implement.md` 里「grep 应为空」的自检项被规划文档自身自指，非产品文档缺陷。
