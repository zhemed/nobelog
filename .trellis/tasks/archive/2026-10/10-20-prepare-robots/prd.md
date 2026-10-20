# 备好 robots.txt 生成与 nginx /images 缓存建议

## Goal

本地预备两处与静态部署相关的小改动（**不动远程**，改动随下一次发版上线）：
① 构建产物自动生成 `robots.txt`（含 Sitemap 绝对地址）；② 在 spec 记录静态部署下 nginx 对 `/images/` 的缓存建议。

## 背景

远程 `https://z.miio.cc` 是 nginx + nobelog 静态模式（反代到静态服务），实测：
- `robots.txt` → 404（站点有 sitemap，缺 robots 文件）；
- `/images/test-cover.svg` → `public, s-maxage=3600, stale-while-revalidate=86400`（PAGE_CACHE），
  v0.0.22 的 `no-cache` 只作用于 `--dynamic`，静态部署下换图可能旧图最多 1 小时（SWR 1 天）。

## Requirements

- R1 `rust/src/build.rs` 在构建时写 `robots.txt`：
  ```
  User-agent: *
  Allow: /

  Sitemap: <base_url>/sitemap.xml
  ```
  `<base_url>` 与 404/feed 相同：`SITE_URL` 优先，否则 `SITE.url`。补单测（有/无 SITE_URL 两种）。
- R2 `PRESERVE_IN_PUBLIC` 已含 `robots.txt`，确认重建时不会被清掉/覆盖冲突；若 out 里已有手写 robots.txt，应保留而不是覆盖（决定并写明）。
- R3 `.trellis/spec/frontend/build-serve-deploy.md` 增加“静态部署（nginx）注意”：`/images/` 建议加
  `Cache-Control: no-cache`（或 `expires -1`）以做到换图即生效；给出最小 snippet，注明需用户授权后由运维执行。
- R4 不改远程、不部署、不重启；不改无关代码。

## Acceptance Criteria

- [ ] `cargo fmt --check` / `clippy --all-targets` / `test` 全绿。
- [ ] 构建产物包含 `robots.txt` 且 Sitemap 使用配置的绝对地址（SITE_URL 生效）。
- [ ] 现有 content 构建 15 页不变；`diff` 只多出 robots.txt。
- [ ] spec 有 nginx 建议与“需授权”标注。
- [ ] Trellis 记录与提交推送。
