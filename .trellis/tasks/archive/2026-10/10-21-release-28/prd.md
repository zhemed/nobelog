# 发布 v0.0.28（首页卡片留白）

## Goal

按既定发版流程发布 v0.0.28：版本三源同步、打包、tag、GitHub Release（默认），并重启本机使版本一致。

## 本版变更（v0.0.27 → v0.0.28）

- 首页文章卡片补留白（`vendor/patches/patch.css`）：
  - 卡片间距恢复上游设计值 `1.25rem`（上游选择器 `#post-list a.card-wrapper` 与本站
    `<article class="card-wrapper">` 结构不匹配，实际间距为 0）；
  - 卡片正文 `padding-bottom` 与 `padding-top` 对齐为 `1.75rem`（原为 1.25rem，日期/分类行贴底）。
- 预期 CSS 产物哈希：`main.1f82b0f7.css` → `main.1ca5c955.css`。

## 流程

1. 版本三源同步 `0.0.28`（Cargo.toml / package.json / Cargo.lock）。
2. 门禁 `cargo fmt --check`、`clippy --all-targets`、`test`（92 项）。
3. `SITE_URL=https://z.miio.cc VERSION=0.0.28 tools/package.sh`；校验包内 VERSION / `--version` / robots /
   CSS 哈希为 `main.1ca5c955.css` / `sha256sum -c`。
4. commit + annotated tag `v0.0.28`（message `0.0.28：首页卡片留白`）+ push。
5. GitHub Release（默认动作）：标题 `v0.0.28：首页卡片留白`，notes 写变更与 sha256，附 tar.gz + SHA256SUMS。
6. 本机 `nobelog-local`：rebuild + 重启一次，确认 `--version` 0.0.28、CSS 哈希切换、
   卡片间距 20px / 底部内边距 28px、9 路由回归通过。

## Acceptance

- [ ] 版本三源一致；包校验通过；tag 与 Release 已推送。
- [ ] 本机跑 v0.0.28；CSS 哈希 = `main.1ca5c955.css`；卡片间距与底部留白实测与预览一致；回归通过。
- [ ] Trellis journal + 归档。
