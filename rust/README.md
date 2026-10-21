# nobelog（Rust 重写）

本目录是 `nobelog` 静态博客生成器的 Rust 实现。**自 2026-10-17 起构建与服务都走 Rust**
（当时通过仓库根的 `pnpm build` / `pnpm serve` 别名调用，这两个别名已随 `package.json` 删除）：
构建 `cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out public`，
服务 `serve`（axum，行为对齐 TS 时代的 `server/serve.ts`）。
Node 源码已移除（完整归档在 `/root/博客/.backups/`，可整包回退）。

- 内核：**content → render → artifacts**；默认产物写 `public/`，对照构建可写 `rust/dist/`。
- 逐字节基准：Node 产物归档在 `/root/博客/.backups/public-node-baseline-20261017-030023.tar.gz`，
  解包后的 `verify-baseline/public` 可用于对照。
- 对比：`diff --normalize-assets` 会在比较前把 `/assets/main.<8hex>.css|js` 归一化（自建资源哈希必然不同）；
  与 Node 原始产物比较时，除资源哈希外 69 个文件逐字节一致。

## 工具链

- `rust-toolchain.toml` 钉死 **Rust 1.99.1**（components：`rustfmt`、`clippy`；profile：`minimal`），不要用其它版本构建。
- `edition = "2024"`，`rust-version = "1.99"`。
- 提交 `Cargo.lock`；构建与验收统一带 `--locked`（`cargo build --locked`）。

## 命令

在 `rust/` 目录下执行：

| 命令 | 说明 |
| --- | --- |
| `cargo build --locked` | 编译 |
| `cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out public`（仓库根执行） | 默认发布：自建 CSS/JS 并写发布目录 |
| `cargo run -- build` | 生成到 `rust/dist/`。默认复用 `public/assets` 与 manifest，便于逐字节对照 |
| `cargo run -- build --self-assets --out public` | 由 Rust 自建 CSS/JS（`src/assets.rs`）并写发布目录 |
| `cargo run -- diff ../public dist` | 与基准逐字节对比；`<lastBuildDate>` 自动归一化；加 `--normalize-assets` 忽略资源哈希 |
| `cargo run -- serve --host 0.0.0.0 --port <port>` | 本地/局域网静态服务（axum），缓存头与 Node 版一致 |
| `cargo run --release --manifest-path rust/Cargo.toml -- serve --host 0.0.0.0 --port <port>`（仓库根执行） | 同上，release 构建 |

`build` 可选参数：`--out <目录>`（默认 `dist`）、`--root <仓库根>`（默认 `rust/` 的上一级）。
`diff` 的两个参数是相对当前工作目录的路径（默认 `public dist`）；在 `rust/` 下运行时写 `../public dist`。

通过判据：diff 最后输出

```text
✓ 逐字节一致（feed 的 lastBuildDate 已归一化）
```

## 目录职责

| 路径 | 职责 |
| --- | --- |
| `src/model.rs` | 中间表示：`Post` / `TocEntry` / `SitePage` / `Taxonomy` / `Content` |
| `src/site.rs` | 站点配置，逐字对齐 `src/site.ts` |
| `src/paths.rs` | 路径与资源常量（样式顺序、`public/` 保留文件等） |
| `src/content.rs` | 读取 `content/`、解析 frontmatter、派生摘要/阅读时长/标签与分类索引 |
| `src/routes.rs` | 路由表与每页数据装配，对齐 `server/routes.ts` |
| `src/markdown.rs` | Markdown → HTML + TOC，逐字节复刻 `src/markdown.ts` |
| `src/templates.rs` | 页面模板与 HTML 转义/日期/分页/侧栏/面板/TOC，逐字节复刻 `src/templates.ts` |
| `src/build.rs` | 构建器：清理并重建 `dist/`、准备资源、写 HTML/feed/sitemap/404 |
| `src/assets.rs` | 阶段二资源构建：拼接压缩 CSS、最小化客户端 JS、内容哈希命名、拷贝字体 |
| `src/url.rs` | `encodeURIComponent` / `decodeURIComponent` 等价实现 |
| `src/highlight.rs` | 代码高亮：syntect tokenize + `hljs-*` 类名输出（见「已知取舍」） |
| `src/highlight_map.rs` | syntect scope → `hljs-*` 类名映射表 |
| `src/serve.rs` | axum 静态服务；有 `public/index.html` 时为纯静态模式（常驻内存约 6 MB），否则运行时直出 |
| `client/main.js` | 客户端脚本源（亮暗模式、TOC、复制代码等） |

## 验收标准

1. 非代码页面与 Node 基准逐字节一致（`<lastBuildDate>` 归一化、资源哈希归一化）。
2. 代码页允许 token 层差异（syntect 与 highlight.js 语法集不同），但代码块外壳、空白、文本内容必须一致。
3. `cargo build --locked` 零警告；`cargo fmt --check` 与 `cargo clippy --all-targets -- -D warnings` 通过。
4. 资源（`--self-assets`）只要求视觉等价，哈希与 Node 必然不同。

## 已知取舍

### 代码高亮：syntect（真 Rust 实现）

- `syntect` 逐行 tokenize，`highlight_map.rs` 把 scope 映射成主题认识的 `hljs-*` 类名。
- 语言别名大小写不敏感；未标语言用 first_line / 首 token / 轻量打分探测；无可用语法时按纯文本输出并汇总一条 warning。
- 与 highlight.js 的差异：`powershell`、`prisma` 等 syntect 默认语法集没有的语言会退化为纯文本（颜色变灰）；
  其余语言颜色与原版接近但不保证逐字节一致。这是接受项。
- 不再依赖 Node；高亮完全由 syntect 在构建/运行时完成。

### 阶段二：已切换为默认（2026-10-17）

- 默认发布命令（`build --self-assets --out public`）使用 `--self-assets`：CSS 按 `paths::STYLE_ORDER` 拼接并轻量压缩，JS 由 `client/main.js` 最小化，
  文件名沿用 `main.<sha256 前 8 位>.css/js`。
- 只承诺**视觉等价**（截图/状态对比），不追 esbuild 的字节级一致，因此哈希与 Node 产物必然不同；
  对比时用 `diff --normalize-assets`。
- 回退：从 `.backups/nobelog-pre-node-removal-20261017-032548.tar.gz` 解包可恢复 Node 版本。
