# 样式与资源

## 样式装配顺序（`rust/src/paths.rs::STYLE_ORDER`）

```
vendor/chirpy/main.css          // 上游主题（已压缩产物）
vendor/fonts/fonts.css          // 自托管字体 @font-face
vendor/github-markdown/github-markdown.css
vendor/patches/patch.css        // 本站补丁，最后加载、天然覆盖上游
```

- `assets.rs::build_css` 按这个顺序**直接拼接**再压缩；顺序即层叠优先级，改顺序 = 改全站外观。
- 上游文件不要改；本站调整按 `patch.css` 里已有的注释分区追加：
  调色板（`①`）、版式（`②`）、组件细节（`③`）…；每段前写清"为什么加这条"，因为上游升级后要能判断是否还需要。

## 暗色模式（三态：light / dark / auto）

- 服务端：`render_layout` 在 `<head>` 里输出 `EARLY_SCRIPT`（首屏前读 `localStorage["nobelog-mode"]`，
  只认显式选择，未选过不写 `data-mode`，交给 CSS 的 `prefers-color-scheme` 跟随系统）。
  同时输出两个 `theme-color` meta（light `#f7f7f7` / dark `#1b1b1e`）。
- 浏览器：`rust/client/main.js` 的 `applyMode` 维护 `html[data-mode]` + `localStorage`；
  `auto` 时**移除**属性，让媒体查询接管。
- 样式：`patch.css` 里 `[data-mode="dark"]` 的变量块与 `@media (prefers-color-scheme: dark)` 并存，
  改暗色要同时考虑"显式 dark"与"系统 dark"两条路径。

## 字体

- 自托管 `vendor/fonts/*.woff2`（Lato 400/700/900、Source Sans Pro 400/400italic）；拉丁走 Source Sans Pro，
  中文回退系统字体（见 `patch.css` ①）。
- 字体与 FA 图标文件必须**保持原文件名**：主题 CSS 用 `/assets/fa-*.woff2`、`lato-*.woff2` 写死引用；
  构建时 `copy_vendored_assets` 原样复制到产物 `assets/`。

## 资源构建（`rust/src/assets.rs`）

产物：

```
assets/main.<sha256前8位>.css
assets/main.<sha256前8位>.js
assets-manifest.json          // {"css":"/assets/main.xxx.css","js":"/assets/main.xxx.js"}
assets/fa-*.woff2, lato-*.woff2, source-sans-pro-*.woff2
```

- 哈希 = `sha256(最终内容)` 十六进制前 8 位（`content_hash`），与 TS 版 `contentHash` 对齐；
  内容不变则文件名不变，可安全长缓存。
- CSS 压缩器是自写的保守实现：去注释（保留 `!` 版权注释）、去冗余空白、去块内最后分号；
  **不动选择器、声明顺序、字符串与 url()**，`a :hover` 这类后代空格、`calc()` 里 `+ -` 两侧空格必须保留。
  不追求 esbuild 逐字节一致，只要求视觉等价。
- JS 压缩同样自写（去注释与缩进），源码只有 `rust/client/main.js`（零依赖、ES module）。

## CSS 裁剪（默认开启）

- 压缩后再按实际使用情况保守删除未使用的选择器（`rust/tools/css_prune.rs`）。
  使用情况优先取 `public/**/*.html`（最准确），没有 HTML 时回退到 `rust/src/**/*.rs` 源码字面量；
  两者都不可用时**跳过裁剪**并打印提示。
- 开关：`NOBELOG_CSS_PRUNE=0`（或 `off` / `false` / `no`）关闭；`NOBELOG_CSS_REPORT=1` 打印裁剪统计。
- 排障顺序：样式"上线后消失" → 先 `NOBELOG_CSS_PRUNE=0` 重新构建，若恢复，就是裁剪误删（选择器只出现在动态拼接的
  class 里），去修 `css_prune` 的保守规则，不要直接关裁剪了事。

## 浏览器 JS（`rust/client/main.js`）

功能固定为六块，文件头有编号：亮暗三态、移动端侧栏、回到顶部、代码复制、TOC 跟随/折叠、分享"复制链接"。

- 选择器与模板强耦合：`#mode-toggle`、`#sidebar-trigger` / `sidebar-display`、`#mask`、`#back-to-top`、
  `[data-copy]`（在 `.highlight` 容器内取 `code`）、`#toc` 下的 `a.toc-link` / `.is-active-li` / `.is-active-link`、
  `[data-copy-link]`、`.markdown-body img`。改模板结构必须同步改这里。
- `localStorage` 访问都包了 `try/catch`（隐私模式可能抛错），新代码保持这个习惯。
- 没有构建步骤、没有依赖；改完直接会被 `assets::build_js` 压缩进产物。

## 反模式

- 直接改 `vendor/chirpy/main.css` 或 `vendor/github-markdown/*.css` → 下次从上游重新抓取即丢失。
- 在 `patch.css` 里用 `#id` 提高特异度硬压 → 优先用变量或与上游同层选择器，避免升级后打补丁套补丁。
- 新增图片资源放 `rust/src` 或 `vendor/` → 站内图片应放 `content/images/`（构建复制到 `images/`）。
- 依赖压缩器行为写样式（例如依赖注释被删除、空白被折叠）。
