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
  未设置时动态模式按 `Host` + `X-Forwarded-Proto` 推断（两个头都经白名单校验，非法值整条丢弃），
  静态构建回退到 `SITE.url`（`http://127.0.0.1:10086`），`serve` 启动时会打一行告警。
  即便有白名单，未设 `SITE_URL` 时**字符合法**的 `X-Forwarded-Host` 仍能改写 canonical（无注入，但可被缓存污染），
  生产环境必须显式设置。
- 本机当前以 systemd 单元 `nobelog-local.service` 托管 release 二进制：
  `nobelog serve --host 0.0.0.0 --port 10086 --root <仓库根> --dynamic`。
  排查"线上页面不对"时先 `systemctl status nobelog-local`，再确认它跑的是 `rust/target/release/nobelog`
  （改完代码要重新 `cargo build --release` 才会生效）。
- 发布包：`tools/package.sh`（`VERSION` 可覆盖，默认 `git describe`），内部执行
  `cargo build --release --locked` + `nobelog build --self-assets --out public`，产物
  `dist/nobelog-<version>-linux-x86_64.tar.gz` + `SHA256SUMS`。包内放二进制、`content/ vendor/ public/`、
  `rust/client/main.js`、README、LICENSE、VERSION。
- 动态模式的内存开销来自"整站内容 + CSS/JS 常驻"；内容规模变大时先量内存再谈优化，不要随意改缓存策略。

## 静态部署（nginx 反代）注意

远程 `https://z.miio.cc` 的形态是 **nginx 反代 + nobelog 静态模式**（origin 输出 `weak_etag()`、`cache_for()` 的
`PAGE_CACHE`，nginx 只做反代/压缩）。由此有两条部署侧约定：

1. **`/images/**` 在静态部署下没有 no-cache**：`assets/**` 带内容哈希可以长缓存，而 `content/images/` 的文件名不含哈希，
   默认会落到 `PAGE_CACHE`（`s-maxage=3600` + SWR 86400）→ 换同名图片后最长约 1 小时（SWR 可到 1 天）仍可能返回旧图。
   若要求"换图即生效"，在 nginx 里给该前缀单独设头（**改远端配置需用户授权，由运维执行**）：
   ```nginx
   location ^~ /images/ {
       add_header Cache-Control "no-cache" always;
       try_files $uri =404;
   }
   ```
   替代方案：图片文件名带内容哈希，或改用 `--dynamic`（`serve.rs::dynamic_file_cache` 已对 `/images/**` 返回 no-cache）。
2. **`robots.txt` 由构建产出**：`build` 会写 `User-agent: * / Allow: / / Sitemap: <base_url>/sitemap.xml`；
   若 out 目录里已有手写 `robots.txt`（`PRESERVE_IN_PUBLIC` 保留）则不覆盖。远程若仍是 404，说明部署的还是旧产物。


## 验证清单（改完必做）

```bash
cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test
cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out dist
./rust/target/release/nobelog diff public dist        # 只允许出现本次预期差异
```

- 改了 `markdown.rs` / `templates.rs`：`diff` 是最关键的一步，它直接给出首个差异行。
- 改了 `serve.rs`：必须跑 `cargo test`（含路径穿越、目录 index、百分号解码用例）。
- 改了资源构建：对比 `assets-manifest.json` 与构建日志（`NOBELOG_CSS_REPORT=1` 可看裁剪统计）。

## 发版流程（默认包含 GitHub Release）

> 用户 2026-10-20 授权：**以后每次发版默认更新 GitHub Release**，无需再逐次询问。

固定动作：

1. **版本同步**：`rust/Cargo.toml`、`package.json`、`rust/Cargo.lock`（`name = "nobelog"` 那一项）三处一起改成新版本号。
2. **门禁**：`cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test`。
3. **打包**：仓库根 `VERSION=<x.y.z> tools/package.sh` → `dist/nobelog-<version>-linux-x86_64.tar.gz` + `SHA256SUMS`；
   `(cd dist && sha256sum -c SHA256SUMS)`，并解包确认 `./nobelog --version` 为新版本。
4. **提交与 tag**：提交消息带任务锚点；打 annotated tag `v<x.y.z>`，message 用 `<x.y.z>：<一句话主题>`；
   `git push origin main && git push origin v<x.y.z>`。
5. **GitHub Release（默认动作）**：用 `gh` 建 release 并附二进制与校验文件：
   ```bash
   gh release create v<x.y.z> --title "v<x.y.z>" --notes-file <notes 文件> \
     dist/nobelog-<x.y.z>-linux-x86_64.tar.gz dist/SHA256SUMS
   gh release view v<x.y.z> --json tagName,name,isDraft,isPrerelease,url,assets
   ```
   notes 写：本版安全/正确性/构建变更要点 + 包 sha256 + Full Changelog 链接。
   若 release 已存在，用 `gh release upload --clobber` + `gh release edit --notes-file` 更新，不要删了重建。
6. **上线**：`cargo build --release --locked` 后 `systemctl restart nobelog-local`（需用户当轮授权）；
   冒烟 9 条路由 + 404、CSS 哈希、feed pubDate 与 XFH 回退。
7. **留痕**：Trellis journal + 任务归档。

注意事项：

- `dist/` 里旧版本包会被保留，但 `SHA256SUMS` 每次打包被覆盖（只含最新包）；历史校验值以 GitHub Release 附件为准。
- 版本字段与 tag 必须一致，否则 `nobelog --version` 与 release 名对不上。

