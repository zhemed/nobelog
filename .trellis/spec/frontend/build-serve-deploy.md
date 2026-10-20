# 构建 · 服务 · 部署

## 命令入口

```bash
# 本地动态模式（改 md 刷新即生效）
pnpm serve                 # = cargo run --release --manifest-path rust/Cargo.toml -- serve --host 0.0.0.0 --port <port> --dynamic
pnpm serve:static          # 同端口，静态模式
pnpm build                 # = ... build --self-assets --out public
```

等价的 cargo 命令见 README。端口不在本规范里写死：文档统一用占位符 `<port>`，实际默认值见 `package.json` 的 scripts 与 CLI。
`rust/rust-toolchain.toml` 固定 `1.99.1` + `rustfmt` + `clippy`。

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

- **必须设置 `SITE_URL`**（如 `SITE_URL=https://<domain>`）：canonical / RSS / sitemap / 页脚 / 分享链接都用它；
  未设置时动态模式**默认只信 `Host`、scheme 固定 `http`，并忽略 `X-Forwarded-*`**（避免任意客户端伪造
  `X-Forwarded-Host` 改写 canonical）；静态构建回退到 `SITE.url` 的内置默认地址（见 `rust/src/site.rs`；不要依赖它），
  `serve` 启动时会打一行告警。
  如果应用确实部署在**可信反向代理**后面，需要转发头生效，必须二选一：设置 `SITE_URL`（推荐），
  或显式设 `NOBELOG_TRUST_FORWARDED=1`（接受 `1/true/yes/on`）才信任 `X-Forwarded-Host` / `X-Forwarded-Proto`。
  注意：在 TLS 反代后运行又没设 `SITE_URL` 时，canonical 会退化成 `http://<host>`。
- 本机当前以 systemd 单元 `nobelog-local.service` 托管 release 二进制：
  `nobelog serve --host 0.0.0.0 --port <port> --root <仓库根> --dynamic`（端口默认值见 CLI）。
  排查"线上页面不对"时先 `systemctl status nobelog-local`，再确认它跑的是 `rust/target/release/nobelog`
  （改完代码要重新 `cargo build --release` 才会生效）。
- 发布包：`tools/package.sh`（`VERSION` 可覆盖，默认 `git describe`），内部执行
  `cargo build --release --locked` + `nobelog build --self-assets --out public`，产物
  `dist/nobelog-<version>-linux-x86_64.tar.gz` + `SHA256SUMS`。包内放二进制、`content/ vendor/ public/`、
  `rust/client/main.js`、README、LICENSE、VERSION。
- 动态模式的内存开销来自"整站内容 + CSS/JS 常驻"；内容规模变大时先量内存再谈优化，不要随意改缓存策略。

## 部署拓扑

构建产物（`public/`）可以落在三种拓扑下，**缓存头该改在哪一层取决于拓扑**。先判断自己属于哪种：

- **(a) nginx 反代 → 应用**：有独立应用进程（`nobelog serve` 静态模式）托管 `public/`，`nginx -T` 里该站点是 `proxy_pass`。
  响应中能看到应用下发的 `PAGE_CACHE` / `ASSET_CACHE` 原值，ETag 是 `weak_etag()` 形态
  （`W/"<size 十六进制>-<mtime_ms 十六进制>"`）。
  > 应用在反代后若需要 `X-Forwarded-*`（例如 canonical 用外网域名），必须设 `SITE_URL` 或 `NOBELOG_TRUST_FORWARDED=1`；
  > 默认只信 `Host`、scheme 固定 `http`。
- **(b) nginx root/alias 纯静态**：`nginx -T` 里该站点用 `root` / `alias` 直接指向构建产物目录，没有应用进程参与；
  静态文件、压缩、缓存头、错误页全归 nginx。
- **(c) 动态模式直连**：直接访问应用端口，或前置层只做 TLS 透传、不改写缓存头。

| 拓扑 | TLS 终止 | 静态文件托管 | 压缩 | 缓存头由谁下发 | 改 `/images/**` 缓存头落在哪层 |
| --- | --- | --- | --- | --- | --- |
| (a) nginx 反代 → 应用 | nginx | 应用（`nobelog serve` 读 `public/`） | 应用（`serve.rs`，≥1024 字节才压；nginx 可再压） | 应用（`ASSET_CACHE` / `PAGE_CACHE`） | nginx：`proxy_hide_header` 丢掉应用的，再 `add_header ... always` 覆盖 |
| (b) nginx root/alias | nginx | nginx | nginx | nginx | nginx：直接 `add_header ... always` |
| (c) 动态模式直连 | 应用 / 前置透传 | 应用（内存资源 + 运行时渲染） | 应用 | 应用（`NO_CACHE` / `IMMUTABLE_CACHE`） | 代码层已对 `/images/**` 返回 `no-cache`（`dynamic_file_cache`），一般无需改 nginx |

> 本节 nginx 片段都是**示例**：`<domain>` / `<port>` / `<app_port>` / `<app_upstream>` / `<cert_path>` / `<static_root>` 为占位符；
> `<app_upstream>` 是反代目标（socket 或 `<host>:<app_port>`），`<static_root>` 是构建产物（`public/` 内容）在 nginx 主机上的目录，
> TLS 证书在 `<cert_path>`，产物目录布局、压缩与 CDN 策略
> 都由实际部署决定，本仓库不预设。**改远端配置需用户/运维授权，由运维执行。**
> 两段片段已用 nginx 1.18 在独立 prefix 下通过 `nginx -t`，并起临时实例实测：
> `/images/**` 响应只有一条 `Cache-Control: no-cache`，缺失文件 404 仍带该头（`always` 生效）。
> 真实环境仍需按自己的 upstream/证书/压缩/CDN 配置调整。

### 覆盖 `/images/**` 的缓存头

为什么需要覆盖：`assets/**` 的文件名带内容哈希，可以长缓存；而 `content/images/` 的文件名不含哈希，
默认落到 `PAGE_CACHE`（`s-maxage=3600` + SWR 86400）→ 换同名图片后最长约 1 小时（SWR 可到 1 天）仍可能返回旧图。
若要「换图即生效」，按拓扑二选一：

#### (a) 反代拓扑：应用托管文件

```nginx
# (a) nginx 反代 → 应用。文件在应用侧，nginx 的 root 下没有这些文件。
location ^~ /images/ {
    proxy_pass http://<app_upstream>;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_hide_header Cache-Control;              # 丢弃应用下发的 PAGE_CACHE
    add_header Cache-Control "no-cache" always;   # 由 nginx 覆盖
}
```

#### (b) 静态 root 拓扑：nginx 直接托管文件

```nginx
# (b) nginx root/alias 纯静态。文件在磁盘上，才能用 try_files。
location ^~ /images/ {
    root <static_root>;              # 目录布局由运维决定，不要照抄
    try_files $uri =404;
    add_header Cache-Control "no-cache" always;
}
```

**改前必看的三点**：

1. **`try_files` 只适用于 (b)**：(a) 下 nginx 本地没有这些文件，写 `try_files $uri =404;` 会让所有图片直接 404；
   (a) 只做 `proxy_pass`，文件是否存在由应用判断。
2. **`add_header` 的继承规则**：只要当前 `location` 里出现任意一条 `add_header`，父级（`http` / `server`）的
   `add_header` 就不再继承。给 `/images/` 加缓存头后，父级配的安全头（如 `Strict-Transport-Security`、
   `X-Content-Type-Options`）不会自动带下来 —— 要么在该 `location` 里重复声明，要么用 `map` 统一生成，
   或 `more_set_headers`（第三方模块，需运维评估）。
3. **CDN 只保证回源验证**：`no-cache` 只保证缓存层回源校验；若该前缀前面还有 CDN，CDN 仍可能按自己的 TTL
   返回缓存，需要在 CDN 侧配置不缓存该前缀或加版本查询串 —— 控制权在 CDN 配置，不在本仓库。

不改 nginx 的替代方案：图片文件名带内容哈希，或改用 `--dynamic`
（`serve.rs::dynamic_file_cache` 已对 `/images/**` 返回 `no-cache`）。

### 静态产物注意

- **`robots.txt` 由构建产出**：`build` 会写 `User-agent: * / Allow: / / Sitemap: <base_url>/sitemap.xml`；
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

