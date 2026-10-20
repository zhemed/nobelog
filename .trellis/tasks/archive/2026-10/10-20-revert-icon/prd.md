# 回退站点图标（含源图与脚本）

## Goal

按用户决定，把站点图标改动完全回退（含设计源图与生成脚本），并让本机恢复为"无自定义图标"状态。

## 内容

- 回退 `1d8e38d`：`rust/src/templates.rs` 的 head 图标链接、`content/images/site-icon-*.png`、
  `tools/make-icons.py`、`tools/assets/site-icon-source.png`、spec 里的 tools 说明行。
- 保留站点原有资源（`content/images/test-cover.svg` 等）不动。
- 重建 release 并重启 `nobelog-local` 一次，验证 head 不再有图标链接、图标端点 404、其余功能回归。

## Acceptance

- [ ] `git revert` 后 `git status` 干净，无图标相关文件残留。
- [ ] `cargo fmt --check` / `clippy` / `test`（92 项）全绿。
- [ ] 本机重启后：head 无 `rel="icon"`/`apple-touch-icon`/`og:image`；`/images/site-icon-*.png` 404；
      9 路由 + 404、feed pubDate、CSS 哈希、`/images` no-cache 均正常。
- [ ] Trellis journal + 归档；提交与推送。
