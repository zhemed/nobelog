# 修复置顶图钉图标渲染成方块

## Goal

首页/文章列表卡片右上角的置顶图钉（`<i class="fas fa-thumbtack">`）渲染成「方块里一个 ×」，恢复为正常图钉。

## 背景

Font Awesome 6 的 `fas`（solid）依赖 `font-weight: 900` 命中实心字体；Regular 免费字体不含图钉字形。
样式里为统一 meta 文字颜色/字重写了：

```css
.post-meta i,
#post-list .card .card-body .post-meta i { font-weight: 400 !important; }
```

这两条把图标字重压到 400，浏览器改用 Regular 字体，缺字形 → 方块。

## Requirements

- R1 只改 `vendor/patches/patch.css`。
- R2 为 `.post-meta` 内的 `i.fas` / `i.fa-solid` 恢复 `font-weight: 900 !important`，选择器需压过 `#post-list ... .post-meta i` 那条。
- R3 不改图标的颜色（仍是 meta 灰）。
- R4 不影响 `far` 常规图标（日历、文件夹）。
- R5 验证：图钉元素计算字重 900，视觉为正常图钉；其它 meta 文字仍 400。

## Acceptance Criteria

- [ ] `.pin i`（`fas fa-thumbtack`）计算 `font-weight: 900`。
- [ ] 图钉图标正常显示，不再是方块。
- [ ] 日历、文件夹、meta 文字样式不变。
- [ ] `git diff --stat` 仅 `vendor/patches/patch.css`。
- [ ] `fmt` / `clippy` / `test` / `build --locked` 全绿。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 验证结果（2026-11-03）

改动：`vendor/patches/patch.css` +9 行。

```css
.post-meta i.fas,
.post-meta i.fa-solid,
#post-list .card .card-body .post-meta i.fas,
#post-list .card .card-body .post-meta i.fa-solid {
  font-weight: 900 !important;
}
```

需要写两条更具体的选择器，是因为 `#post-list .card .card-body .post-meta i`（1,3,1）
会压过 `.post-meta i.fas`（0,2,1）。

实测：

| 元素 | 改前 | 改后 |
| --- | --- | --- |
| 置顶图钉 `fas fa-thumbtack` | 字重 400 → 缺字方块 | **900**，正常图钉 |
| 日历 `far fa-calendar` | 400 | 400（不变） |
| meta 文字 | 400 | 400（不变） |
| 图标颜色 | `rgb(107,114,128)` | 不变 |

- 静态构建：仅 CSS 哈希变化，无结构差异。
- 质量：`fmt` 通过、`clippy` 无警告、`test` 98 passed、`build --release --locked` 通过。
