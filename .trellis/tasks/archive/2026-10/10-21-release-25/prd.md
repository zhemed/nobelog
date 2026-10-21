# 发布 v0.0.25（站点图标）

## Goal

用户验收通过，按既定发版流程发布 v0.0.25：版本三源同步、打包、tag、GitHub Release（默认），
并重启本机使版本号一致。

## 本版变更（v0.0.24 → v0.0.25）

- 站点图标：熊脸线稿（`tools/assets/bear-icon-source.jpg`）→ `site-icon.svg`（明暗自适应）+
  32/192 透明 PNG + 32/192 暗色变体 + apple-touch 180 + og:image 512；白底 favicon 已移除。
- 图标生成脚本 `tools/make-icons.py` 入库（可传源图切换设计）。
- 模板 head 五处链接；spec `directory-structure.md` 补 tools 说明。

## 流程

1. 版本三源同步 `0.0.25`（Cargo.toml / package.json / Cargo.lock）。
2. 门禁 `cargo fmt --check`、`clippy --all-targets`、`test`（92 项）。
3. `SITE_URL=https://z.miio.cc VERSION=0.0.25 tools/package.sh`；校验包内 VERSION/`--version`/robots/sha256。
4. commit + annotated tag `v0.0.25`（message `0.0.25：站点图标`）+ push。
5. GitHub Release（默认动作）：标题 `v0.0.25：站点图标`，notes 写变更与 sha256，附 tar.gz + SHA256SUMS。
6. 本机 `nobelog-local`：rebuild + 重启一次，确认 `--version` 0.0.25、图标与回归通过。

## Acceptance

- [ ] 版本三源一致；包校验通过；tag 与 Release 已推送。
- [ ] 本机跑 v0.0.25；7 个图标资源 200、head 五处、9 路由回归通过。
- [ ] Trellis journal + 归档。
