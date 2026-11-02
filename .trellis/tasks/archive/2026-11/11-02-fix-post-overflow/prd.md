# 修复文章页宽内容导致的横向溢出

## Goal

消除文章页在 1200–1649px 视口下的横向滚动：含长代码行或宽表格的文章会把 flex 列撑宽，导致右侧栏被挤出视口。修复后页面整体宽度不超过视口，宽内容在代码块/表格自身容器内横向滚动，首页与其它页面布局不变。

## 背景

`vendor/patches/patch.css` 的版式补丁（侧栏 | 56 | 主栏 ≤1060 | 56 | 右栏 240）给容器设置了 `width: 100% !important` 与左右各 56px 外边距，超出的 112px 依赖 flex 收缩吸收。

`main` 使用默认 `min-width: auto`，当内容 min-content 较大（长代码行、宽表格）时收缩失效，容器向外扩张，右栏被顶出视口，页面出现横向滚动条。实测 1264px 视口下文章页 `scrollWidth 1515 > clientWidth 1264`。

## Requirements

- R1 改动范围：只改 `vendor/patches/patch.css`；不动 `rust/src/templates.rs`、`rust/client/main.js`、`vendor/chirpy/main.css`、`content/`。
- R2 保持既有版式意图：主栏上限 1060、左右间隔 56、右栏 240，这些数值不改。
- R3 允许收缩：布局链上的 flex 项必须能收缩到内容 min-content 以下，不得再把父容器撑宽。
- R4 宽内容不裁切：长代码行在代码块内部横向滚动，宽表格在表格容器内部横向滚动，行为与 Chirpy 原主题一致。
- R5 不引入横向滚动条：文章页 `documentElement.scrollWidth` 应等于 `clientWidth`。
- R6 回归：首页、分类、标签、归档、关于页在 1264px 视口下布局与改动前一致。
- R7 亮色与暗色均无异常；本次只碰布局，不改颜色令牌。
- R8 验证：`cargo build --locked` 通过；静态构建与改动前产物对比，HTML 逐字节一致，仅 CSS 资源哈希变化。

## Acceptance Criteria

- [x] `/posts/typeset-test/` 在 1264px 视口下 `scrollWidth === clientWidth`，无横向滚动条。
- [x] 右侧栏完整可见：`#panel-wrapper` 右边缘不超过视口宽度。
- [x] 长代码块在自身容器内可横向滚动（`pre.scrollWidth > pre.clientWidth`）。
- [x] 首页在 1264px 视口下布局与改动前一致（容器宽度与右栏位置符合同一公式）。
- [x] `git diff --stat` 仅 `vendor/patches/patch.css` 一个文件。
- [x] `cargo build --locked` 通过；静态构建 HTML 与改动前逐字节一致，仅 CSS 哈希不同。
- [x] 任务归档并记录 journal，提交带任务锚点。

## 非目标

- 不重构 `width: 100% + 外边距` 的写法本身（如需，另立任务评估）。
- 不改 1200px 以下断点的响应式布局。

## 验证结果（2026-11-03）

- 改动：`vendor/patches/patch.css` +9 行（`@media (min-width:1200px)` 块内新增 `min-width:0 !important`，作用于 `#main-wrapper > .container` 与 `#main-wrapper .row.flex-grow-1 > main`）。
- 浏览器实测（1264px 视口，动态模式）：
  - 文章页 `/posts/typeset-test/`：`scrollWidth 1249 === clientWidth 1249`，溢出 0；`#panel-wrapper` 953–1193 完整可见；长代码块 `pre` 529/835 内部滚动，行为与 Chirpy 原主题一致。
  - 首页：溢出 0，容器 877（= clientWidth 1249 − 侧栏 260 − 左右间隔 112），右栏 953–1193，与改动前同一公式，布局未变。
  - 其它页 `/about/`、`/archives/`、`/categories/`、`/tags/` 均 200。
- 静态构建：`build --self-assets --out public` 15 个页面 153219 字节；与改动前产物归一化资源哈希后逐文件比对，42 个文件仅 `feed.xml` 的 `lastBuildDate` 不同（构建时间），HTML 全部一致。
- 质量检查：`cargo fmt --check` 通过、`cargo clippy --all-targets --locked` 无警告、`cargo test --locked` 98 passed / 0 failed、`cargo build --release --locked` 通过。
- 范围：`git diff --stat` 仅 `vendor/patches/patch.css | 9 +++++++++`。
