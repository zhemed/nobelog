# vendor/github-markdown —— GitHub 的 Markdown 渲染样式

用户 2026-10-17 指定："GitHub 的配色和按钮我很喜欢…直接把 GitHub 的拿过来"。

| 项 | 值 |
|---|---|
| 文件 | `github-markdown.css`（31,068 B） |
| 来源 | `https://unpkg.com/github-markdown-css@5/github-markdown.css` |
| 包 | `github-markdown-css` **v5.9.0**（"The minimal amount of CSS to replicate the GitHub Markdown style"） |
| 作者 | Sindre Sorhus |
| 许可 | **MIT**（可自由使用/修改/再分发，保留版权声明） |

## 用法

- 只作用于带 `.markdown-body` 类的容器（见 `src/templates.ts` 的正文容器），侧栏/卡片仍是 Chirpy 主题；
- 样式里所有颜色都是 CSS 变量（`--fgColor-*` / `--bgColor-*` / `--borderColor-*`），
  暗色由包内 `@media (prefers-color-scheme: dark)` 提供；本站显式切换暗色（`data-mode="dark"`）
  的那份变量覆盖写在 `src/styles/patch.css` 里（因为包的暗色只跟系统偏好）。
- 原文件**未做修改**；本站的补充（正文字体、复制按钮、暗色变量）全部在 `patch.css`。

## 更新方式

```bash
curl -sSL --compressed -o vendor/github-markdown/github-markdown.css \
  https://unpkg.com/github-markdown-css@5/github-markdown.css
```
更新后需同步 `patch.css` 里的暗色变量块（从新文件 `@media (prefers-color-scheme: dark)` 段复制）。
