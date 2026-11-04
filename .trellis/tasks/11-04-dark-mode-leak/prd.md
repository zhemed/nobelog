# 修复系统深色下白底白字

## 现象

访问者系统为深色时，文章页正文变浅灰、代码块发白，白底几乎看不清；标题与链接正常。

## 根因

1. 主题 CSS（chirpy）有 `@media (prefers-color-scheme: dark)` 分支，按**操作系统**深浅色切换变量；
2. 前端 JS 在 `auto` 档位会**删掉 `data-mode`**，让上面那段媒体查询接管；
3. 于是系统深色时：代码/语法色变量切成深色值（浅色文字），页面底色却被本站浅色板压成白色 → 白底白字；
4. 引用的 github-markdown 也自带同名媒体查询，且选择器不认 `data-mode`，把 `.markdown-body` 文字刷成 `#f0f6fc`，属于第二重来源。

## 修复

1. `rust/client/main.js`：`auto` 不再删 attribute，而是按 `prefers-color-scheme` 解析成 `light`/`dark` 写入，并监听系统切换；
2. `rust/src/templates.rs` 的 `EARLY_SCRIPT`：首屏前同样按系统解析并写入，避免闪烁与首帧错误；
3. `vendor/patches/patch.css`：为 `html:not([data-mode])` / `html[data-mode="light"]` 覆盖 github-markdown 的媒体查询变量与颜色，防止浅色态被刷成深色值。

## 验证（2048 视口，Chromium 模拟配色）

| 场景 | data-mode | 正文色 | 底色 |
| --- | --- | --- | --- |
| 系统深色（auto） | dark | #f0f6fc | #14171b |
| 系统浅色（auto） | light | #33373d | #ffffff |
| 系统深色 + 用户选浅色 | light | #33373d | #ffffff |

`fmt` / `clippy` / `test` 98 全绿。
