# 发布 v0.0.24（XFH 默认安全）

## Goal

发布安全补丁版 v0.0.24：默认不再信任 `X-Forwarded-*`（`SITE_URL` 优先；需要时显式
`NOBELOG_TRUST_FORWARDED=1`），并重启本机使版本号一致。

## 流程（按 spec 发版清单）

- 版本三源同步 `0.0.24`（Cargo.toml / package.json / Cargo.lock）。
- 门禁 fmt/clippy/test（92 项）。
- `SITE_URL=https://<domain> VERSION=0.0.24 tools/package.sh`，校验包 sha256、`--version`、robots。
- commit + annotated tag `v0.0.24`（message `0.0.24：默认不信任 X-Forwarded-*`）+ push。
- GitHub Release（默认）：标题 `v0.0.24：XFH 默认安全`，附 tar.gz + SHA256SUMS。
- 本机 `nobelog-local`：rebuild + 重启一次，确认版本 0.0.24、XFH 回归仍 PASS。

## Constraints

- 只动本机；远程不负责。
- 不改其它行为。

## Acceptance

- [ ] 版本三源一致，包校验通过，tag/Release 已推送。
- [ ] 本机服务跑 v0.0.24，XFH 默认安全仍成立，路由/缓存回归通过。
- [ ] Trellis journal + 归档。
