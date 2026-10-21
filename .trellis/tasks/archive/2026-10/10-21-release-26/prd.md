# 发布 v0.0.26（粉笔少女图标）

## Goal

按既定发版流程发布 v0.0.26：版本三源同步、打包、tag、GitHub Release（默认），并重启本机使版本号一致。

## 本版变更（v0.0.25 → v0.0.26）

- 站点图标更换为粉笔风少女涂鸦：`site-icon.svg`（内嵌亮/暗两版，`prefers-color-scheme` 切换）+
  `site-icon-32-transparent.png`、`-192-transparent.png` + 暗色变体 `-32-dark.png`、`-192-dark.png` +
  `site-icon-180.png`（apple-touch）、`site-icon-512.png`（og:image）。
- 暗色变体改为**饱和度分流**：近中性色才做深浅反转，蓝眼/粉腮红/黄星/红心等彩色保留。
- `tools/make-icons.py` 扩展：支持传入源图、`lineart` 与 `bleed`（满幅插画）两种模式；
  源图集中在 `tools/assets/`（猫、熊、插画、粉笔少女）。

## 流程

1. 版本三源同步 `0.0.26`（Cargo.toml / package.json / Cargo.lock）。
2. 门禁 `cargo fmt --check`、`clippy --all-targets`、`test`（92 项）。
3. `SITE_URL=https://z.miio.cc VERSION=0.0.26 tools/package.sh`；校验包内 VERSION / `--version` / robots /
   `public/images/` 图标清单 / `sha256sum -c`。
4. commit + annotated tag `v0.0.26`（message `0.0.26：粉笔少女图标`）+ push。
5. GitHub Release（默认动作）：标题 `v0.0.26：粉笔少女图标`，notes 写变更与 sha256，附 tar.gz + SHA256SUMS。
6. 本机 `nobelog-local`：rebuild + 重启一次，确认 `--version` 0.0.26、图标与回归通过。

## Acceptance

- [ ] 版本三源一致；包校验通过；tag 与 Release 已推送。
- [ ] 本机跑 v0.0.26；图标资源 200、head 五处、9 路由回归通过。
- [ ] Trellis journal + 归档。
