# 发版 0.0.38

## 内容

工程加固版，无内容与样式变化（站点 HTML 产物字节不变）：

- 新增 `.github/workflows/ci.yml`：push main / PR / 手动触发，跑 fmt、clippy `-D warnings`、test、release build `--locked`、站点构建冒烟；仅官方 action。
- 新增 `tools/fetch-baseline.sh`：从 GitHub Release 还原 `public/` 冻结基线（sha256 + 包内 `VERSION` 校验，只写 `public/`），全新 clone 也能复现 `nobelog diff`。
- release profile 移除 `panic = "abort"`，启用 `tower-http` catch-panic：单请求 panic 返回 500 而不退出进程；动态重建 panic 由 `JoinError` 捕获并退避；二进制 +7.03%（6,222,264 → 6,659,792 字节），保留 `strip`。
- CI actions 升级：`actions/checkout` v7、`actions/cache` v6（脱离 Node20 弃用提示）。
- 本机按用户决定不配域名（`SITE_URL` 事项关闭），发布包使用默认占位域名 `https://example.com`。

## 流程与验证

1. 版本同步：`rust/Cargo.toml` 与 `rust/Cargo.lock`（`name = "nobelog"` 项）0.0.37 → 0.0.38；
2. 门禁：`cd rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`（98 通过）`&& cargo build --release --locked`；
3. 打包：仓库根 `VERSION=0.0.38 tools/package.sh`（不传 `SITE_URL`，默认 `https://example.com`）→ `dist/nobelog-0.0.38-linux-x86_64.tar.gz` + `SHA256SUMS`；
4. 包校验：`(cd dist && sha256sum -c SHA256SUMS)`；解包确认 `./nobelog --version` = 0.0.38、`VERSION` 文件一致、条目数约 110；
5. 提交与 tag：版本提交带锚点 `[task:11-10-release-0038]`；annotated tag `v0.0.38`，message 用 `0.0.38：<一句话主题>`；`git push origin main && git push origin v0.0.38`；
6. GitHub Release（仓库默认动作）：`gh release create v0.0.38 --title "v0.0.38" --notes-file <notes> dist/nobelog-0.0.38-linux-x86_64.tar.gz dist/SHA256SUMS`；
7. 上线：`cargo build --release --locked` 后 `systemctl restart nobelog-local`；冒烟 9 条路由 + 404、CSS 哈希、feed pubDate；
8. 留痕：Trellis 归档 + journal。

## Acceptance Criteria

- [x] `rust/Cargo.toml`、`rust/Cargo.lock`、`rust/target/release/nobelog --version`、包内 `VERSION` 均为 0.0.38。
- [x] fmt / clippy `-D warnings` / test（98 passed）/ build `--release --locked` 全绿。
- [x] `dist/nobelog-0.0.38-linux-x86_64.tar.gz` 与 `SHA256SUMS` 生成，`sha256sum -c` 通过；解包后 `./nobelog --version` = 0.0.38。
- [x] 版本提交带任务锚点；annotated tag `v0.0.38` 指向该提交；main 与 tag 均已推送。
- [x] GitHub Release `v0.0.38` 存在，附 tarball + SHA256SUMS，notes 含本版要点与 sha256。
- [x] 本机 `nobelog-local` 重启后 9 条路由 + 404 冒烟通过，页面正常（版本 0.0.38）。
- [x] 两个任务（CI 维护、发版）归档并记录 journal。

## Out of Scope

- 内容/样式/模板变更；依赖版本升级；`SITE_URL` 配置（用户决定本地不需要域名）。

## 验证结果（2026-11-10，verifier 独立复核 + Lead 执行）

- verifier 复核 7 项全 PASS：仅 3 个 tracked 文件改动（+4/-4）；版本一致性（Cargo.toml / Cargo.lock / 二进制 / 包内 VERSION / 包内二进制）全为 0.0.38；门禁 fmt/clippy `-D warnings`/test 98/build `--locked` 全绿；发布包 sha256 与大小复算一致；解包 110 条目齐全；`diff public rust/dist` = 42 同 / 0 不同。
- 发布包：`dist/nobelog-0.0.38-linux-x86_64.tar.gz` 5,148,336 字节，sha256 `f1366b2daa4b9d08112d18ab13f99557061704480b3fafe00f27a5a3f45c3b2e`；`sha256sum -c` 通过；解包后 `./nobelog --version` = `nobelog 0.0.38`。
- 提交与 tag：版本提交 `9bb7061 chore: 发版 0.0.38 [task:11-10-release-0038]`；annotated tag `v0.0.38` 指向该提交；main 与 tag 均已推送。
- GitHub Release：[v0.0.38](https://github.com/zhemed/nobelog/releases/tag/v0.0.38)，附 `nobelog-0.0.38-linux-x86_64.tar.gz`（5,148,336 字节）与 `SHA256SUMS`，notes 含本版要点与 sha256。
- CI：[run #42860166864](https://github.com/zhemed/nobelog/actions/runs/42860166864) `success`（2m25s，新 actions 版本）。
- 上线冒烟（本机 `nobelog-local`）：`cargo build --release --locked` 后重启，PID 220164，`/proc/220164/exe` sha256 = `6e5bf048…`（0.0.38 构建产物）；9 条路由（`/`、`/about/`、`/archives/`、`/tags/`、`/categories/`、两篇文章、`/feed.xml`、`/sitemap.xml`）全 200、未知路径 404；CSS `/assets/main.8f4a0d54.css` 200；feed pubDate 正常。
- 本机按用户决定不配域名：服务保持 `SITE_URL` 未设置，canonical 按请求 Host 推断；发布包使用默认占位域名 `https://example.com`。
