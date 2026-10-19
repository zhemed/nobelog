# Bootstrap Task: Fill Project Development Guidelines

**状态：已完成（2026-10-19）。** 本文件保留作为本次填充结果与依据的记录。

## 背景

开发者首次运行 `trellis init`，`.trellis/spec/` 是空模板（React 假设：components / hooks / state / TypeScript）。
本项目实际是 **Rust 静态站生成器 + axum 服务**，模板层用 `format!` 拼 HTML、浏览器只跑一个
零依赖 `rust/client/main.js`，因此模板里的 React 类指南不适用。

## 本次填充结果

### 最终文件集（`.trellis/spec/frontend/`）

| 文件 | 内容 | 依据 |
| --- | --- | --- |
| `index.md` | 层次速览、指南索引、三条不可协商规则、与 TS 版的逐字节兼容背景 | `README.md`、`rust/src/main.rs` |
| `directory-structure.md` | 真实目录布局、Rust 模块表、vendor 规则、反模式 | `rust/src/*`、`vendor/`、`tools/package.sh` |
| `content-and-frontmatter.md` | frontmatter 字段表、UTC 时区约定、排序、摘要 UTF-16 截断、阅读时长公式 | `content.rs`、`model.rs` |
| `rendering-templates.md` | `Ctx`/`PageAssets` 契约、转义、三种日期格式、分页、侧栏高亮、sitemap/RSS | `templates.rs`、`routes.rs` |
| `markdown-pipeline.md` | 共享/独立渲染器、标题 id、提示块、代码块结构、TOC 2–4 级 | `markdown.rs`、`highlight*.rs` |
| `styling-and-assets.md` | `STYLE_ORDER`、暗色三态、字体、哈希命名、CSS 裁剪开关 | `paths.rs`、`assets.rs`、`patch.css` |
| `routes-and-urls.md` | 路由表、新增路由清单、`encodeURIComponent` 对齐、`SITE_URL` 优先级 | `routes.rs`、`url.rs`、`site.rs` |
| `build-serve-deploy.md` | 构建两阶段、静态/动态模式、缓存头、压缩、打包与 systemd 部署 | `build.rs`、`serve.rs`、`tools/package.sh` |
| `quality-guidelines.md` | fmt/clippy/test/diff、提交前缀与任务锚点、禁止事项、常见事故 | 仓库历史与全局 git 钩子行为 |

### 删除的模板文件

`component-guidelines.md`、`hook-guidelines.md`、`state-management.md`、`type-safety.md`
—— 本项目没有组件库、hooks、前端状态管理或 TypeScript。

### 保留

`.trellis/spec/guides/` 两个 thinking guide 与项目无冲突，未改动。

## Acceptance

- [x] 每个文件都指向真实路径/函数，而不是通用建议。
- [x] 无 `TBD` / `To fill` / 空标题 / 复制样板（已 grep 校验）。
- [x] `index.md` 与实际文件集一致。
- [x] 记录了关键的项目特有约束：`public/` 是冻结基准、逐字节 `diff`、UTC 日期语义、CSS 裁剪开关。
