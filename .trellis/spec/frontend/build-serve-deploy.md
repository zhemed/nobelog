# 构建 · 服务 · 部署

## 命令入口

```bash
# 本地动态模式（改 md 刷新即生效）
pnpm serve                 # = cargo run --release --manifest-path rust/Cargo.toml -- serve --host 0.0.0.0 --port 10086 --dynamic
pnpm serve:static          # 同端口，静态模式
pnpm build                 # = ... build --self-assets --out public
```

等价的 cargo 命令见 README。`rust/rust-toolchain.toml` 固定 `1.99.1` + `rustfmt` + `clippy`。

三个子命令（`rust/src/main.rs`）：

| 子命令 | 作用 |
| --- | --- |
| `build` | 生成站点；默认 `--out dist`，`--self-assets` 由 Rust 自建 CSS/JS |
| `serve` | axum 服务；`--dynamic` 切动态模式，`--host/--port/--root` 可覆盖 |
| `diff` | 与冻结基准逐字节对比（归一化 `<lastBuildDate>`；`--normalize-assets` 归一化资源哈希） |

`build`/`serve` 找不到仓库根时按 `NOBELOG_ROOT` → 当前目录 → 可执行文件目录 → 编译期目录
（`default_root()`）依次尝试，判定标志是 `vendor/chirpy/main.css` 存在。

## 构建两阶段

`build::build(root, out, self_assets)`：

1. **资源**：
   - 默认（阶段一）：复用 `public/assets` 与 `public/assets-manifest.json`（缺少 manifest 直接报错）。
   - `--self-assets`（阶段二）：`assets::build_assets` 从 `vendor/` 自建，写 `assets/main.<hash>.css|js` + manifest + 字体。
2. **内容 → HTML**：`Site::new` 加载内容，`all_routes()` 遍历 `collect_paths()`，200 路由写成文件：
   - `/` → `index.html`；`/a/` → `a/index.html`；无尾斜杠路径 → 同名文件（`file_for_route`，先解码、拒绝 `\0`）。
3. **清理**：`out` 已存在时先清空，但保留 `PRESERVE_IN_PUBLIC`（`CNAME`、`_headers`、`_redirects`、`robots.txt`、`.gitkeep`）。
4. 额外写根 `404.html`，打印页面数/字节数/耗时。

注意：**产物写 `rust/dist/`（默认），不要写 `public/`** —— `public/` 是逐字节对比的冻结基准。

## 服务两种模式（`rust/src/serve.rs`）

| 模式 | 触发 | 行为 |
| --- | --- | --- |
| 静态（默认） | 无 `--dynamic` | 优先服务 `public/`；命中文件交给 `tower-http ServeFile`（Range / If-Modified-Since）；没有 `public/index.html` 时退化为运行时直出 `Site::page` |
| 动态 | `--dynamic` | 完全不依赖 `public/`：资源 `assets::build_in_memory` 常驻内存，页面实时渲染；每次请求检查 `content/` mtime，变化则 `spawn_blocking` 重建 `Site` + 资源并原子替换 |

- 路径穿越必须继续被拒绝（`resolve_static` / `resolve_file_under` 有单测覆盖 `..`、`%2e%2e`、`%5C`）。
- 目录不做 301 重定向（Express `redirect:false` 语义）：`/about` 不会自动跳 `/about/`。
- 压缩：文本类（html/css/js/json/xml/rss/svg）按 `Accept-Encoding` 压 gzip/br，
  **响应 ≥1024 字节才压**；ETag 基于未压缩内容；304/206 不压缩；统一带 `Vary: Accept-Encoding`。

### 缓存头（常量在 `serve.rs` 顶部，与 TS 版逐字一致）

| 常量 | 值 | 用于 |
| --- | --- | --- |
| `ASSET_CACHE` | `public, max-age=31536000, s-maxage=15552000, stale-while-revalidate=2592000` | 静态模式带哈希资源 |
| `PAGE_CACHE` | `public, s-maxage=3600, stale-while-revalidate=86400` | 静态模式页面 |
| `NO_CACHE` | `no-cache` | 动态模式页面/清单 |
| `IMMUTABLE_CACHE` | `public, max-age=31536000, immutable` | 动态模式带哈希资源 |
| `NO_STORE` | `no-store` | 动态模式内容接口 |

改缓存策略属于线上行为变更，先确认部署方式（CDN/反代是否另有缓存头覆盖）。

## 部署

- **必须设置 `SITE_URL`**（如 `SITE_URL=https://example.com`）：canonical / RSS / sitemap / 页脚 / 分享链接都用它；
  未设置时动态模式按 `Host` + `X-Forwarded-Proto` 推断，静态构建回退到 `SITE.url`（`http://127.0.0.1:10086`）。
- 本机当前以 systemd 单元 `nobelog-local.service` 托管 release 二进制：
  `nobelog serve --host 0.0.0.0 --port 10086 --root <仓库根> --dynamic`。
  排查"线上页面不对"时先 `systemctl status nobelog-local`，再确认它跑的是 `rust/target/release/nobelog`
  （改完代码要重新 `cargo build --release` 才会生效）。
- 发布包：`tools/package.sh`（`VERSION` 可覆盖，默认 `git describe`），内部执行
  `cargo build --release --locked` + `nobelog build --self-assets --out public`，产物
  `dist/nobelog-<version>-linux-x86_64.tar.gz` + `SHA256SUMS`。包内放二进制、`content/ vendor/ public/`、
  `rust/client/main.js`、README、LICENSE、VERSION。
- 动态模式的内存开销来自"整站内容 + CSS/JS 常驻"；内容规模变大时先量内存再谈优化，不要随意改缓存策略。

## 验证清单（改完必做）

```bash
cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test
cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out dist
./rust/target/release/nobelog diff public dist        # 只允许出现本次预期差异
```

- 改了 `markdown.rs` / `templates.rs`：`diff` 是最关键的一步，它直接给出首个差异行。
- 改了 `serve.rs`：必须跑 `cargo test`（含路径穿越、目录 index、百分号解码用例）。
- 改了资源构建：对比 `assets-manifest.json` 与构建日志（`NOBELOG_CSS_REPORT=1` 可看裁剪统计）。
