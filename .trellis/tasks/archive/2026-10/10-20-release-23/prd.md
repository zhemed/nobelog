# 发布 v0.0.23（含 robots.txt）

## Goal

按 `.trellis/spec/frontend/build-serve-deploy.md` 的既定发版流程发布 v0.0.23：
版本号同步 → 门禁 → `--locked` 构建 → `package.sh` 打包 → sha256 校验 → commit + annotated tag push →
**默认创建 GitHub Release（2 附件）**。

## 本版变更

- 构建期生成 `robots.txt`（`User-agent: *` / `Allow: /` / `Sitemap: <SITE_URL>/sitemap.xml`）；
  自动产物带首行标记，`SITE_URL` 变化会覆盖；手写文件（无标记）保留。
- spec 增加静态部署（nginx）对 `/images/**` 缓存的建议。

## Requirements

- R1 版本三源同步 `0.0.23`（Cargo.toml / package.json / Cargo.lock）。
- R2 `cargo fmt --check`、`clippy --all-targets`、`test`（90 项）全绿。
- R3 `VERSION=0.0.23 tools/package.sh`（**构建时带 `SITE_URL=https://z.miio.cc`**，让包内 robots/sitemap 用真实域名）；
  校验包内 `robots.txt` 的 Sitemap 行、`--version`=0.0.23、sha256sum -c。
- R4 提交消息带 `[task:10-20-release-23]`；annotated tag `v0.0.23`，message `0.0.23：robots.txt 与静态部署说明`；推送 main 与 tag。
- R5 GitHub Release：标题 `v0.0.23：robots.txt 生成`，notes 写本版变更 + sha256，附 tar.gz 与 SHA256SUMS。
- R6 不部署远程（仅只读授权）；本地 systemd 是否重启由用户另行决定。

## Acceptance Criteria

- [ ] 版本三源一致，锁文件仅 nobelog 一行变化。
- [ ] 包生成并校验通过，包内 `public/robots.txt` 的 Sitemap = `https://z.miio.cc/sitemap.xml`。
- [ ] main 与 tag 已推送；GitHub Release 存在且 assets=2、Latest。
- [ ] Trellis journal + 归档。
