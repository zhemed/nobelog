# 回退 favicon 清晰化（回到粉笔少女细节版）

## Goal

用户决定废除清晰化版本，回退到清晰化**之前**的粉笔少女细节版。

## 内容

- `git revert` 两笔提交（新→旧）：`235f071`（四角透明修复）与 `48f9504`（16/32/48 简化版 + SVG 内嵌 48px +
  small 模式 + head 七处）。
- 回退后应恢复：
  - `content/images/site-icon-32-transparent.png`（透明细节版）、`site-icon-192-transparent.png`、`-192-dark.png`、
    `site-icon.svg`（内嵌 192 细节版）、`site-icon-180.png`、`site-icon-512.png`
  - 删除 `site-icon-16/32/48.png` 与 `-16/32/48-dark.png`
  - `rust/src/templates.rs` head 回到五处（svg + 32-transparent + 192-transparent + apple-touch 180 + og:image 512）
  - `tools/make-icons.py` 去掉 small 模式与自检
- 重建 release + 重启 `nobelog-local` 一次（用户已选「回退并重启」）。

## Acceptance

- [ ] `git status` 干净，无 16/32/48 残留。
- [ ] 门禁 fmt / clippy / test（92 项）全绿。
- [ ] 本机重启后：head 五处；`site-icon.svg`、`-32-transparent`、`-192-transparent`、`-192-dark`、`-180`、`-512` 均 200；
      `site-icon-16.png`、`-32.png`、`-48.png` 与 dark 版本全部 404；回归（9 路由、feed、CSS、images no-cache）通过。
- [ ] 提交、journal、归档。
