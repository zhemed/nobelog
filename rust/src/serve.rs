// 常驻服务：两种模式共用一套路由与缓存策略。
//
// - 静态模式（默认）：`public/` 里的构建产物优先，常驻内存小；没有 `public/index.html` 时
//   退化为运行时直出（Site::page），供开发态使用。行为对齐 server/serve.ts。
// - 动态模式（`--dynamic`）：完全不依赖 `public/`。资源由 `assets::build_in_memory` 常驻内存，
//   页面用 `Site::page` 实时渲染；每次请求检查 `content/` 的 mtime，有更新就用
//   `spawn_blocking` 重建 Site + 资源并原子替换，做到改 md 即时生效（无需重启/构建）。
//
// 静态文件的读写 / Range / If-Modified-Since 交给 tower-http 的 ServeFile；目录不做 301 重定向。
// 响应压缩：文本类（html/css/js/json/xml/rss/svg）按 Accept-Encoding 压 gzip/br，响应统一带
// `Vary: Accept-Encoding`；ETag 仍基于未压缩内容，304 / Range 206 不压缩。

use crate::assets;
use crate::routes::{PageKind, Site};
use crate::templates::PageAssets;
use anyhow::{Context as _, Result};
use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{Extensions, HeaderMap, HeaderValue, Method, StatusCode, Version, header};
use axum::middleware::{self, Next};
use axum::response::Response;
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeFile;

/// §3 定案的缓存头原值（与 server/serve.ts 逐字一致）。
const ASSET_CACHE: &str =
    "public, max-age=31536000, s-maxage=15552000, stale-while-revalidate=2592000";
const PAGE_CACHE: &str = "public, s-maxage=3600, stale-while-revalidate=86400";
/// 动态模式：内容每次请求都可能变，页面/清单只做协商缓存。
const NO_CACHE: &str = "no-cache";
/// 动态模式：文件名里带内容哈希，内容不可变。
const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";
const NO_STORE: &str = "no-store";

const INDEX_FILE: &str = "index.html";
const ERROR_500: &str = "<h1>500 Internal Error</h1>";

/// 小于这个字节数的响应不压缩（压缩头开销不值得）。
const MIN_COMPRESS_SIZE: u64 = 1024;

/// 动态模式：content/ 指纹的缓存时长。命中 TTL 的请求不再 stat 整棵目录树；
/// 代价是新增 / 删除 / 改写最多延迟这么久才被察觉（仍远小于人工刷新间隔）。
const STAMP_TTL: Duration = Duration::from_millis(1000);

/// 动态模式：重建失败后的退避窗口。坏内容不立刻重试，避免每个请求都重编整站。
const RELOAD_BACKOFF: Duration = Duration::from_secs(10);

/* ------------------------------------------------------------ 静态模式 */

#[derive(Clone)]
struct AppState {
    public: Arc<PathBuf>,
    assets: Arc<PageAssets>,
    /// 静态模式（存在 public/index.html）下为 None：不加载内容层，避免启动时渲染全站。
    site: Option<Arc<Site>>,
    /// true = 只服务 public/，未命中直接返回 404.html。
    static_mode: bool,
}

#[derive(Deserialize)]
struct Manifest {
    css: String,
    js: String,
}

/* ------------------------------------------------------------ 动态模式 */

/// 动态模式的一份完整快照：内容层 + 内存资源；热更新时整份替换。
struct Runtime {
    site: Site,
    assets: PageAssets,
    css: String,
    js: String,
    manifest: String,
    /// 构建时 content/ 的指纹，用来判断是否需要重建。
    stamp: ContentStamp,
}

#[derive(Clone)]
struct DynamicState {
    root: Arc<PathBuf>,
    runtime: Arc<RwLock<Arc<Runtime>>>,
    /// 同一时间只允许一个重建任务，避免多个请求同时编译内容。
    reloading: Arc<AtomicBool>,
    /// 最近一次 content/ 指纹检查（带检查时刻，TTL 内直接复用）。
    stamp: Arc<RwLock<CheckedStamp>>,
    /// 最近一次重建失败时的指纹与时刻：内容没变且还在退避窗口内就不再重试。
    failure: Arc<RwLock<Option<(ContentStamp, Instant)>>>,
}

/// 带检查时刻的指纹缓存。
#[derive(Clone, Copy)]
struct CheckedStamp {
    checked_at: Instant,
    stamp: ContentStamp,
}

impl DynamicState {
    async fn load(root: &Path) -> Result<Self> {
        let root = root.to_path_buf();
        let build_root = root.clone();
        let runtime = tokio::task::spawn_blocking(move || build_runtime(&build_root))
            .await
            .context("动态模式初始化失败")??;
        let stamp = runtime.stamp;
        Ok(Self {
            root: Arc::new(root),
            runtime: Arc::new(RwLock::new(Arc::new(runtime))),
            reloading: Arc::new(AtomicBool::new(false)),
            stamp: Arc::new(RwLock::new(CheckedStamp {
                checked_at: Instant::now(),
                stamp,
            })),
            failure: Arc::new(RwLock::new(None)),
        })
    }

    fn snapshot(&self) -> Arc<Runtime> {
        self.runtime
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn replace(&self, runtime: Arc<Runtime>) {
        *self
            .runtime
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = runtime;
    }

    /// content/ 当前指纹。TTL 内的请求复用上次结果，避免每个请求（含 404）都同步 walkdir。
    fn content_stamp(&self) -> ContentStamp {
        {
            let checked = self
                .stamp
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !stamp_cache_expired(checked.checked_at, Instant::now()) {
                return checked.stamp;
            }
        }
        let stamp = content_stamp(&self.root);
        self.remember_stamp(stamp);
        stamp
    }

    fn remember_stamp(&self, stamp: ContentStamp) {
        *self
            .stamp
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = CheckedStamp {
            checked_at: Instant::now(),
            stamp,
        };
    }

    /// 内容指纹没变且仍在上次失败的退避窗口内 → 这次请求不再重试。
    fn is_backing_off(&self, stamp: ContentStamp) -> bool {
        let failure = *self
            .failure
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        reload_backed_off(failure, stamp, Instant::now())
    }

    fn record_failure(&self, stamp: ContentStamp) {
        *self
            .failure
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((stamp, Instant::now()));
    }

    fn clear_failure(&self) {
        *self
            .failure
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    /// 内容有更新就重建并原子替换。重建失败时记录错误并退避一会儿，期间继续用旧快照。
    async fn current(&self) -> Arc<Runtime> {
        let snapshot = self.snapshot();
        let stamp = self.content_stamp();
        if !is_stale(stamp, snapshot.stamp) {
            return snapshot;
        }
        if self.is_backing_off(stamp) {
            // 同一份坏内容刚失败过：不再每个请求都重编一遍，等退避窗口过去或内容变化。
            return snapshot;
        }
        if self
            .reloading
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            // 已有请求在重建：先用旧快照，重建完成后新快照自然生效。
            return snapshot;
        }

        let root = (*self.root).clone();
        let rebuilt = tokio::task::spawn_blocking(move || build_runtime(&root)).await;
        self.reloading.store(false, Ordering::Release);

        match rebuilt {
            Ok(Ok(runtime)) => {
                self.clear_failure();
                // 以快照自带的指纹刷新缓存：构建期间又改过文件时，下一次请求仍能察觉。
                self.remember_stamp(runtime.stamp);
                let runtime = Arc::new(runtime);
                self.replace(Arc::clone(&runtime));
                runtime
            }
            Ok(Err(error)) => {
                eprintln!("动态模式重建失败，继续使用旧快照：{error:#}");
                self.record_failure(stamp);
                snapshot
            }
            Err(error) => {
                eprintln!("动态模式重建任务异常，继续使用旧快照：{error}");
                self.record_failure(stamp);
                snapshot
            }
        }
    }
}

/// 构建一份动态快照。先记 content/ 的 mtime 再读内容：构建期间再改文件会在下一次请求重建。
fn build_runtime(root: &Path) -> Result<Runtime> {
    let stamp = content_stamp(root);
    let built = assets::build_in_memory(root)?;
    let site = Site::new(root)?;
    Ok(Runtime {
        site,
        assets: built.assets,
        css: built.css,
        js: built.js,
        manifest: built.manifest,
        stamp,
    })
}

/* ------------------------------------------------------------ 启动 */

/// 启动常驻服务。`root` 是仓库根（`public/`、`content/` 的父目录）。
/// `dynamic = true` 时不读 `public/`，改用内存资源 + 运行时渲染 + 内容热更新。
pub async fn run(root: &Path, host: &str, port: u16, dynamic: bool) -> Result<()> {
    // SITE_URL 决定所有绝对链接；缺失时只告警一次，继续用内置默认值启动。
    crate::site::warn_if_env_url_missing();

    let listener = tokio::net::TcpListener::bind((host, port))
        .await
        .with_context(|| format!("无法监听 {host}:{port}"))?;
    let bound = listener
        .local_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| format!("{host}:{port}"));

    // HTTP 响应常由多个小写入组成；关闭 Nagle，避免与延迟 ACK 叠加出 ~40ms 等待。
    use axum::serve::ListenerExt;
    let listener = listener.tap_io(|stream| {
        if let Err(err) = stream.set_nodelay(true) {
            eprintln!("设置 TCP_NODELAY 失败：{err}");
        }
    });
    if dynamic {
        println!("nobelog serve  http://{bound}  [dynamic]");
    } else {
        println!("nobelog serve  http://{bound}");
    }

    let app = if dynamic {
        let state = DynamicState::load(root).await?;
        Router::new().fallback(handle_dynamic).with_state(state)
    } else {
        let public = root.join("public");
        let static_mode = public.join(INDEX_FILE).is_file();
        let state = AppState {
            assets: Arc::new(read_assets(&public)),
            public: Arc::new(public),
            site: if static_mode {
                None
            } else {
                Site::new(root).ok().map(Arc::new)
            },
            static_mode,
        };
        Router::new().fallback(handle).with_state(state)
    };

    // 压缩层放在最外层：内层响应先写好 Vary / ETag / Range，压缩只改 body 与 Content-Length。
    let app = app
        .layer(middleware::from_fn(add_accept_encoding_vary))
        .layer(CompressionLayer::new().compress_when(should_compress_response));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("serve 异常退出")?;
    Ok(())
}

/// 动态模式下根据请求推断站点绝对地址：SITE_URL 优先，否则用 Host + X-Forwarded-Proto。
fn request_base_url(headers: &HeaderMap, fallback: &str) -> String {
    if let Some(url) = crate::site::env_url() {
        return url;
    }
    base_url_from_headers(headers, fallback)
}

/// 从请求头推断 base_url。Host / X-Forwarded-Host / X-Forwarded-Proto 全部按白名单校验，
/// 不合法的那一条整条丢弃（回退到下一个来源），绝不把原始值拼进结果：
/// 它们会进入 canonical、og:url、页脚链接与 feed，是典型的头注入落点。
fn base_url_from_headers(headers: &HeaderMap, fallback: &str) -> String {
    let host = headers
        .get("x-forwarded-host")
        .and_then(validated_host)
        .or_else(|| headers.get(header::HOST).and_then(validated_host));
    let Some(host) = host else {
        return fallback.to_string();
    };
    // X-Forwarded-Proto 只认精确的 http / https；file、javascript 等一律当没写。
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| *value == "http" || *value == "https")
        .unwrap_or("http");
    format!("{proto}://{host}")
}

/// 取反向代理链上的第一个 Host 值并校验字符集：只允许主机名/端口/IPv6 方括号。
/// 出现 `"` `<` `>` `@` `/` `\` 空白或控制字符时返回 None（调用方回退）。
fn validated_host(value: &HeaderValue) -> Option<String> {
    let host = value.to_str().ok()?.split(',').next()?.trim();
    if host.is_empty() || !host.chars().all(is_host_char) {
        return None;
    }
    Some(host.to_string())
}

/// Host 白名单：`[A-Za-z0-9.\-:\[\]]`（字母数字、点、连字符、端口冒号、IPv6 方括号）。
fn is_host_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | ':' | '[' | ']')
}

/* ------------------------------------------------------------ 响应压缩 */

/// 压缩谓词：只压白名单文本类、≥1KB，且不是 206/304/204。
/// Range 响应（带 Content-Range）由 tower-http 自己跳过，这里再按状态码挡一次。
fn should_compress_response(
    status: StatusCode,
    _version: Version,
    headers: &HeaderMap,
    _extensions: &Extensions,
) -> bool {
    if status == StatusCode::PARTIAL_CONTENT
        || status == StatusCode::NOT_MODIFIED
        || status == StatusCode::NO_CONTENT
        || status.is_informational()
    {
        return false;
    }
    if headers.contains_key(header::CONTENT_ENCODING) {
        return false;
    }
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(is_compressible_content_type)
        && header_content_length(headers).is_some_and(|length| length >= MIN_COMPRESS_SIZE)
}

/// text/* 一律可压；另加常见 JSON/XML/RSS/SVG 类型；woff2/png 等已压缩格式不在白名单。
fn is_compressible_content_type(content_type: &str) -> bool {
    let essence = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    essence.starts_with("text/")
        || matches!(
            essence.as_str(),
            "application/javascript"
                | "application/json"
                | "application/xml"
                | "application/rss+xml"
                | "application/atom+xml"
                | "application/manifest+json"
                | "application/xhtml+xml"
                | "image/svg+xml"
        )
}

fn header_content_length(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
}

/// 所有响应统一带 `Vary: Accept-Encoding`（含未压缩响应与 304）。
/// 这层在压缩层内层：压缩层看到已有 Vary 就不会重复追加。
async fn add_accept_encoding_vary(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    ensure_accept_encoding_vary(response.headers_mut());
    response
}

fn ensure_accept_encoding_vary(headers: &mut HeaderMap) {
    let already = headers.get_all(header::VARY).iter().any(|value| {
        value
            .to_str()
            .is_ok_and(|text| text.to_ascii_lowercase().contains("accept-encoding"))
    });
    if !already {
        headers.append(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    }
}

/* ------------------------------------------------------------ 静态模式 handler */

/// 静态模式入口：GET/HEAD 先查 public/ 产物，再走运行时直出 / 404 页。
async fn handle(State(state): State<AppState>, req: Request) -> Response {
    let method = req.method().clone();
    let raw_path = req.uri().path().to_owned();

    if method != Method::GET && method != Method::HEAD {
        return text_response(
            StatusCode::NOT_FOUND,
            "text/html; charset=utf-8",
            NO_STORE,
            "<h1>404 Not Found</h1>",
            false,
        );
    }

    if let Some(response) = serve_public(&state, &raw_path, req).await {
        return response;
    }
    render_page(&state, &raw_path, method == Method::HEAD)
}

/// 命中 public/ 就返回静态响应；未命中（或读取失败）返回 None 交给运行时直出。
async fn serve_public(state: &AppState, raw_path: &str, req: Request) -> Option<Response> {
    let file = resolve_static(&state.public, raw_path)?;
    let cache = cache_for(&state.public, &file);
    serve_file(&file, cache, req).await
}

/// 磁盘文件的统一响应：ETag / Last-Modified / Range / 条件请求，再补缓存头。
async fn serve_file(file: &Path, cache: &'static str, mut req: Request) -> Option<Response> {
    let meta = std::fs::metadata(file).ok()?;
    if !meta.is_file() {
        return None;
    }

    let etag = weak_etag(&meta);
    let mtime = meta.modified().ok();

    // If-None-Match 优先于 If-Modified-Since（Express send 的 freshness 语义）。
    if let Some(if_none_match) = req
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
    {
        req.headers_mut().remove(header::IF_MODIFIED_SINCE);
        if etag_matches(&if_none_match, &etag) {
            return Some(not_modified(&etag, cache, mtime));
        }
    }

    // 多段 Range：Express 会忽略并回 200 全文，ServeFile 会回 416；按 serve.ts 的行为忽略。
    let multi_range = req
        .headers()
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains(','));
    if multi_range {
        req.headers_mut().remove(header::RANGE);
    }

    let mut service = ServeFile::new(file);
    let response = service.try_call(req).await.ok()?;
    if response.status() == StatusCode::NOT_FOUND {
        return None;
    }

    let mut response = response.map(Body::new);
    decorate_static(&mut response, cache, &etag, file, mtime);
    Some(response)
}

/// 运行时直出：feed / sitemap / HTML 按 kind 定 Content-Type，200 用 PAGE_CACHE，其余 no-store。
fn render_page(state: &AppState, raw_path: &str, head: bool) -> Response {
    let Some(site) = &state.site else {
        // 静态模式：所有有效路由都已在构建期落盘，未命中就是 404 页。
        if state.static_mode
            && let Ok(html) = std::fs::read_to_string(state.public.join("404.html"))
        {
            return text_response(
                StatusCode::NOT_FOUND,
                "text/html; charset=utf-8",
                NO_STORE,
                &html,
                head,
            );
        }
        return text_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "text/html; charset=utf-8",
            NO_STORE,
            ERROR_500,
            head,
        );
    };

    let page = site.page(raw_path, &state.assets, &site.root_url);
    let status = StatusCode::from_u16(page.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let content_type = match page.kind {
        PageKind::Feed => "application/rss+xml; charset=utf-8",
        PageKind::Sitemap => "application/xml; charset=utf-8",
        _ => "text/html; charset=utf-8",
    };
    let cache = if status == StatusCode::OK {
        PAGE_CACHE
    } else {
        NO_STORE
    };
    text_response(status, content_type, cache, &page.html, head)
}

/* ------------------------------------------------------------ 动态模式 handler */

async fn handle_dynamic(State(state): State<DynamicState>, req: Request) -> Response {
    let method = req.method().clone();
    let raw_path = req.uri().path().to_owned();

    if method != Method::GET && method != Method::HEAD {
        return text_response(
            StatusCode::NOT_FOUND,
            "text/html; charset=utf-8",
            NO_STORE,
            "<h1>404 Not Found</h1>",
            false,
        );
    }
    let head = method == Method::HEAD;
    let request_headers = req.headers().clone();
    let if_none_match = request_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    // 字体 / 图片直接读磁盘，不依赖内容快照，也不会触发重建。
    if let Some(file) = dynamic_asset_file(&state.root, &raw_path)
        && file.is_file()
    {
        return match serve_file(&file, dynamic_file_cache(&raw_path), req).await {
            Some(response) => response,
            None => text_response(
                StatusCode::NOT_FOUND,
                "text/html; charset=utf-8",
                NO_CACHE,
                "<h1>404 Not Found</h1>",
                head,
            ),
        };
    }

    let runtime = state.current().await;

    if raw_path == "/assets-manifest.json" {
        return dynamic_body_response(
            &method,
            &request_headers,
            StatusCode::OK,
            "application/json; charset=UTF-8",
            NO_CACHE,
            None,
            runtime.manifest.as_bytes(),
        );
    }

    if let Some(kind) = in_memory_asset(&raw_path) {
        let (content_type, body) = match kind {
            InMemoryAsset::Css => ("text/css; charset=UTF-8", runtime.css.as_str()),
            InMemoryAsset::Js => ("application/javascript; charset=UTF-8", runtime.js.as_str()),
        };
        return dynamic_body_response(
            &method,
            &request_headers,
            StatusCode::OK,
            content_type,
            IMMUTABLE_CACHE,
            None,
            body.as_bytes(),
        );
    }

    render_dynamic(
        &runtime,
        &raw_path,
        &method,
        &request_headers,
        if_none_match.as_deref(),
    )
}

/// 动态模式渲染：200/404 都走 Site::page（404 自带侧栏），HTML 用 no-cache + 强 ETag。
fn render_dynamic(
    runtime: &Runtime,
    raw_path: &str,
    method: &Method,
    request_headers: &HeaderMap,
    if_none_match: Option<&str>,
) -> Response {
    let base = request_base_url(request_headers, &runtime.site.root_url);
    let page = runtime.site.page(raw_path, &runtime.assets, &base);
    let status = StatusCode::from_u16(page.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let content_type = match page.kind {
        PageKind::Feed => "application/rss+xml; charset=utf-8",
        PageKind::Sitemap => "application/xml; charset=utf-8",
        _ => "text/html; charset=utf-8",
    };

    let etag = strong_etag(&page.html);
    if if_none_match.is_some_and(|value| etag_matches(value, &etag)) {
        return not_modified(&etag, NO_CACHE, None);
    }
    dynamic_body_response(
        method,
        request_headers,
        status,
        content_type,
        NO_CACHE,
        Some(&etag),
        page.html.as_bytes(),
    )
}

/// 动态响应构造：HEAD / Range（单段）/ ETag / 缓存头统一处理；206 只切指定区间。
fn dynamic_body_response(
    method: &Method,
    request_headers: &HeaderMap,
    status: StatusCode,
    content_type: &'static str,
    cache: &'static str,
    etag: Option<&str>,
    body: &[u8],
) -> Response {
    let head = method == Method::HEAD;
    if status == StatusCode::OK {
        match parse_range(request_headers, body.len()) {
            RangeRequest::Partial(start, end) => {
                let slice = &body[start..=end];
                let mut builder = Response::builder()
                    .status(StatusCode::PARTIAL_CONTENT)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CACHE_CONTROL, cache)
                    .header(header::ACCEPT_RANGES, "bytes")
                    .header(
                        header::CONTENT_RANGE,
                        format!("bytes {start}-{end}/{}", body.len()),
                    )
                    .header(header::CONTENT_LENGTH, slice.len());
                if let Some(etag) = etag {
                    builder = builder.header(header::ETAG, etag);
                }
                let body = if head {
                    Body::empty()
                } else {
                    Body::from(slice.to_vec())
                };
                return builder
                    .body(body)
                    .unwrap_or_else(|_| Response::new(Body::empty()));
            }
            RangeRequest::Unsatisfiable => {
                let mut builder = Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CACHE_CONTROL, cache)
                    .header(header::ACCEPT_RANGES, "bytes")
                    .header(header::CONTENT_RANGE, format!("bytes */{}", body.len()))
                    .header(header::CONTENT_LENGTH, 0);
                if let Some(etag) = etag {
                    builder = builder.header(header::ETAG, etag);
                }
                return builder
                    .body(Body::empty())
                    .unwrap_or_else(|_| Response::new(Body::empty()));
            }
            RangeRequest::Whole => {}
        }
    }

    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, body.len());
    if let Some(etag) = etag {
        builder = builder.header(header::ETAG, etag);
    }
    let body = if head {
        Body::empty()
    } else {
        Body::from(body.to_vec())
    };
    builder
        .body(body)
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RangeRequest {
    /// 没有 Range 头、多段 Range 或语法无法识别：按 200 全文返回（与 Express 一致）。
    Whole,
    /// 单段 Range：`start..=end`（闭区间，已按 body 长度裁剪）。
    Partial(usize, usize),
    /// 起点越界 / 后缀为 0：416。
    Unsatisfiable,
}

fn parse_range(headers: &HeaderMap, length: usize) -> RangeRequest {
    let Some(value) = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
    else {
        return RangeRequest::Whole;
    };
    let Some(spec) = value.strip_prefix("bytes=") else {
        return RangeRequest::Whole;
    };
    // 多段 Range：Express 会忽略并回 200 全文，这里保持一致。
    if spec.contains(',') {
        return RangeRequest::Whole;
    }
    let Some((start, end)) = spec.split_once('-') else {
        return RangeRequest::Whole;
    };
    let (start, end) = (start.trim(), end.trim());

    if start.is_empty() {
        // 后缀写法 `-N`：最后 N 个字节。
        let Ok(suffix) = end.parse::<usize>() else {
            return RangeRequest::Whole;
        };
        if suffix == 0 || length == 0 {
            return RangeRequest::Unsatisfiable;
        }
        return RangeRequest::Partial(length.saturating_sub(suffix), length - 1);
    }

    let Ok(start) = start.parse::<usize>() else {
        return RangeRequest::Whole;
    };
    if start >= length {
        return RangeRequest::Unsatisfiable;
    }
    let end = if end.is_empty() {
        length - 1
    } else {
        match end.parse::<usize>() {
            Ok(end) => end.min(length - 1),
            Err(_) => return RangeRequest::Whole,
        }
    };
    if end < start {
        return RangeRequest::Unsatisfiable;
    }
    RangeRequest::Partial(start, end)
}

/// 动态模式下可直读磁盘的两类资源：vendored 字体、content/images。
fn dynamic_asset_file(root: &Path, raw_path: &str) -> Option<PathBuf> {
    if raw_path.starts_with("/assets/") && raw_path.ends_with(".woff2") {
        return resolve_vendor_font(root, raw_path);
    }
    let rest = raw_path.strip_prefix("/images/")?;
    resolve_file_under(&root.join("content/images"), rest)
}

/// 动态模式磁盘文件的缓存策略。
/// `/assets/**` 是构建期固定的 vendor 字体，沿用一年强缓存；
/// `/images/**` 来自 content/，用户可以随时用同名文件替换，必须走可变策略
/// （no-cache + ETag / Last-Modified 复验：内容没变仍是 304，换了文件立刻生效）。
fn dynamic_file_cache(raw_path: &str) -> &'static str {
    if raw_path.starts_with("/assets/") {
        ASSET_CACHE
    } else {
        NO_CACHE
    }
}

/// `/assets/<name>.woff2` → vendor/chirpy 或 vendor/fonts 下的固定文件。
fn resolve_vendor_font(root: &Path, raw_path: &str) -> Option<PathBuf> {
    let name = raw_path.strip_prefix("/assets/")?;
    let decoded = percent_decode_str(name).decode_utf8().ok()?;
    if decoded.is_empty()
        || decoded.contains('/')
        || decoded.contains('\\')
        || decoded.contains("..")
    {
        return None;
    }
    for directory in ["vendor/chirpy", "vendor/fonts"] {
        let candidate = root.join(directory).join(decoded.as_ref());
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InMemoryAsset {
    Css,
    Js,
}

/// `/assets/main.<8 位 hex>.css|js` → 内存资源。哈希对不上也返回当前内容：
/// 本地开发时旧页面引用旧文件名也能正常加载。
fn in_memory_asset(raw_path: &str) -> Option<InMemoryAsset> {
    let name = raw_path.strip_prefix("/assets/")?;
    let (stem, extension) = name.rsplit_once('.')?;
    let hash = stem.strip_prefix("main.")?;
    if hash.len() != 8 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match extension {
        "css" => Some(InMemoryAsset::Css),
        "js" => Some(InMemoryAsset::Js),
        _ => None,
    }
}

/* ------------------------------------------------------------ 响应构造 */

fn text_response(
    status: StatusCode,
    content_type: &'static str,
    cache: &'static str,
    body: &str,
    head: bool,
) -> Response {
    bytes_response(status, content_type, cache, body.as_bytes(), head)
}

fn bytes_response(
    status: StatusCode,
    content_type: &'static str,
    cache: &'static str,
    body: &[u8],
    head: bool,
) -> Response {
    let length = body.len();
    let body = if head {
        Body::empty()
    } else {
        Body::from(body.to_vec())
    };
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache)
        .header(header::CONTENT_LENGTH, length)
        .body(body)
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn not_modified(etag: &str, cache: &'static str, mtime: Option<SystemTime>) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .header(header::CACHE_CONTROL, cache)
        .header(header::ETAG, etag);
    if let Some(date) = mtime.and_then(http_date) {
        builder = builder.header(header::LAST_MODIFIED, date);
    }
    builder
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

/* ------------------------------------------------------------ 工具 */

/// 给静态响应补上缓存头、ETag、以及和 Express 对齐的 Content-Type。
fn decorate_static(
    response: &mut Response,
    cache: &'static str,
    etag: &str,
    file: &Path,
    mtime: Option<SystemTime>,
) {
    let status = response.status();
    let headers = response.headers_mut();

    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    if let Ok(value) = HeaderValue::from_str(etag) {
        headers.insert(header::ETAG, value);
    }
    if status != StatusCode::NOT_MODIFIED {
        if let Some(content_type) = static_content_type(file) {
            headers.insert(header::CONTENT_TYPE, content_type);
        }
    } else if !headers.contains_key(header::LAST_MODIFIED)
        && let Some(date) = mtime.and_then(http_date)
        && let Ok(value) = HeaderValue::from_str(&date)
    {
        headers.insert(header::LAST_MODIFIED, value);
    }
}

/// `/assets/*` 一年强缓存，其余构建产物沿用页面缓存头。
/// 判断依据是解析后的文件路径（和 serve.ts 里 filePath.startsWith(ASSET_ROOT) 一致），
/// 这样 `//assets/x.css` 这类多斜杠写法也会命中资源缓存。
fn cache_for(public: &Path, file: &Path) -> &'static str {
    match file.strip_prefix(public) {
        Ok(relative) if relative.starts_with("assets") => ASSET_CACHE,
        _ => PAGE_CACHE,
    }
}

/// URL 路径 → public/ 下的候选文件。目录不做 301：只有带尾斜杠的路径才补 index.html，
/// 和 express.static({ redirect: false }) 的表现一致（`/about` 落到运行时 404，`/about/` 命中静态）。
fn resolve_static(public: &Path, raw_path: &str) -> Option<PathBuf> {
    let decoded = percent_decode_str(raw_path).decode_utf8().ok()?;
    if decoded.contains('\0') {
        return None;
    }

    let mut path = public.to_path_buf();
    let mut has_segment = false;
    for segment in decoded.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment == "." || segment == ".." || segment.contains('\\') {
            return None;
        }
        path.push(segment);
        has_segment = true;
    }

    if !has_segment || decoded.ends_with('/') {
        path.push(INDEX_FILE);
    }
    Some(path)
}

/// URL 后缀 → 指定目录下的文件（不补 index.html），拒绝目录穿越。
fn resolve_file_under(base: &Path, url_suffix: &str) -> Option<PathBuf> {
    let decoded = percent_decode_str(url_suffix).decode_utf8().ok()?;
    if decoded.contains('\0') {
        return None;
    }
    let mut path = base.to_path_buf();
    let mut has_segment = false;
    for segment in decoded.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment == "." || segment == ".." || segment.contains('\\') {
            return None;
        }
        path.push(segment);
        has_segment = true;
    }
    has_segment.then_some(path)
}

/// content/ 的指纹：所有文件的「相对路径 + 大小 + mtime（符号链接再加链接目标）」整体哈希。
///
/// 只取最大 mtime 会漏掉两类变化：删除文章（剩余文件的最大值不变）和用备份还原
/// （mtime 反而更旧），结果旧快照一直对外服务、删掉的文章还在。整体哈希能把
/// 大小 / 路径 / mtime 的任一变化都算进去，判定语义也随之简化成「不相等即过期」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContentStamp(u64);

/// content/ 下的一个文件入指纹的要素。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StampEntry {
    /// 相对 content/ 的路径。
    path: String,
    len: u64,
    mtime_ns: i128,
    /// 符号链接的原始目标（相对链接按原样保存）；普通文件为 None。
    link_target: Option<String>,
}

/// content/ 下每个文件的（相对路径、字节数、mtime 纳秒）；目录不存在时是空集合。
///
/// 会**跟随符号链接**：`content.rs::read_posts` / `read_pages` 用 `fs::read_to_string` 读文件，
/// 对符号链接是跟随的。指纹必须和加载范围一致，否则「新增一篇链接文章」或「改链接目标」
/// 都不会触发重建（线上表现为改了 md 但页面永远不变）。
fn content_stamp(root: &Path) -> ContentStamp {
    let base = root.join("content");
    let mut entries = Vec::new();
    for entry in walkdir::WalkDir::new(&base)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        // 目录交给 walkdir 递归，不进指纹；目录符号链接也跳过（walkdir 默认不跟随目录链接，
        // content.rs 也是逐目录 read_dir，两边范围一致，同时避免链接成环）。
        if entry.file_type().is_dir() {
            continue;
        }
        // 普通文件与符号链接一律用 fs::metadata（跟随链接）取目标元数据：
        // 链接断掉时这里返回 Err，直接跳过，不 panic，也不影响其它文件。
        let Ok(meta) = std::fs::metadata(path) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let Ok(relative) = path.strip_prefix(&base) else {
            continue;
        };
        // 1970 年之前的 mtime 用负数表示，避免与「拿不到 mtime」混成同一个值。
        let mtime_ns = match modified.duration_since(UNIX_EPOCH) {
            Ok(elapsed) => elapsed.as_nanos() as i128,
            Err(error) => -(error.duration().as_nanos() as i128),
        };
        // 链接目标本身也算身份的一部分：把链接改指向另一个（大小 / mtime 恰好相同的）文件时，
        // 只靠目标元数据会漏掉，带上目标路径就一定能察觉。
        let link_target = if entry.file_type().is_symlink() {
            std::fs::read_link(path)
                .ok()
                .map(|target| target.to_string_lossy().into_owned())
        } else {
            None
        };
        entries.push(StampEntry {
            path: relative.to_string_lossy().into_owned(),
            len: meta.len(),
            mtime_ns,
            link_target,
        });
    }
    stamp_digest(entries)
}

/// 把文件清单折成指纹。先排序：walkdir 的遍历顺序依赖文件系统，不排序会得到不稳定的哈希。
fn stamp_digest(mut entries: Vec<StampEntry>) -> ContentStamp {
    entries.sort_unstable();
    let mut hasher = DefaultHasher::new();
    entries.len().hash(&mut hasher);
    for entry in entries {
        entry.path.hash(&mut hasher);
        entry.len.hash(&mut hasher);
        entry.mtime_ns.hash(&mut hasher);
        entry.link_target.hash(&mut hasher);
    }
    ContentStamp(hasher.finish())
}

/// 快照是否过期：content/ 的指纹与快照构建时的指纹不同（新增 / 删除 / 改写都算）。
fn is_stale(current: ContentStamp, cached: ContentStamp) -> bool {
    current != cached
}

/// 指纹缓存是否已过期（距上次检查已超过 TTL）。
fn stamp_cache_expired(checked_at: Instant, now: Instant) -> bool {
    now.saturating_duration_since(checked_at) >= STAMP_TTL
}

/// 重建失败后的退避判断：内容指纹没变、且距上次失败还在窗口内 → 跳过重试。
/// 内容一变（用户可能已经改好了）就立刻允许重建，不必干等窗口结束。
fn reload_backed_off(
    failure: Option<(ContentStamp, Instant)>,
    stamp: ContentStamp,
    now: Instant,
) -> bool {
    failure.is_some_and(|(failed_stamp, failed_at)| {
        failed_stamp == stamp && now.saturating_duration_since(failed_at) < RELOAD_BACKOFF
    })
}

/// 内容哈希 ETag：sha256 前 16 位十六进制，强校验（带引号）。
fn strong_etag(body: &str) -> String {
    let digest = Sha256::digest(body.as_bytes());
    let hexed = hex::encode(digest);
    format!("\"{}\"", &hexed[..16])
}

/// 与 Express 的静态 ETag 同格式：W/"<size hex>-<mtime 毫秒 hex>"。
/// Node 的 Date / mtimeMs 对亚毫秒部分四舍五入，这里保持一致，ETag 与 Node 版逐字节相同。
fn weak_etag(meta: &std::fs::Metadata) -> String {
    let size = meta.len();
    let mtime_ms = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| (duration.as_micros() + 500) / 1000)
        .unwrap_or(0);
    format!("W/\"{size:x}-{mtime_ms:x}\"")
}

/// If-None-Match 是弱比较，且允许多值 / `*`。
fn etag_matches(if_none_match: &str, etag: &str) -> bool {
    let target = etag.strip_prefix("W/").unwrap_or(etag);
    if_none_match.split(',').any(|raw| {
        let token = raw.trim();
        token == "*" || token.strip_prefix("W/").unwrap_or(token) == target
    })
}

fn http_date(time: SystemTime) -> Option<String> {
    let datetime: chrono::DateTime<chrono::Utc> = time.into();
    Some(datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
}

/// 和 Express（mime + charset）对齐的静态 Content-Type；未覆盖的类型交给 ServeFile 猜。
fn static_content_type(file: &Path) -> Option<HeaderValue> {
    let extension = file.extension()?.to_str()?.to_ascii_lowercase();
    let value = match extension.as_str() {
        "html" | "htm" => "text/html; charset=UTF-8",
        "css" => "text/css; charset=UTF-8",
        "js" | "mjs" => "application/javascript; charset=UTF-8",
        "json" | "map" | "webmanifest" => "application/json; charset=UTF-8",
        "xml" | "rss" => "application/xml",
        "txt" => "text/plain; charset=UTF-8",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "ico" => "image/x-icon",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        _ => return None,
    };
    Some(HeaderValue::from_static(value))
}

/// manifest 缺失或损坏时退化成开发态资源名（此时运行时页面仍可直出，只是静态资源 404）。
fn read_assets(public: &Path) -> PageAssets {
    std::fs::read_to_string(public.join("assets-manifest.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Manifest>(&raw).ok())
        .map(|manifest| PageAssets {
            css: manifest.css,
            js: manifest.js,
        })
        .unwrap_or_else(|| PageAssets {
            css: "/assets/main.css".to_string(),
            js: "/assets/main.js".to_string(),
        })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public() -> PathBuf {
        PathBuf::from("/srv/public")
    }

    #[test]
    fn root_and_directory_index() {
        assert_eq!(
            resolve_static(&public(), "/").unwrap(),
            PathBuf::from("/srv/public/index.html")
        );
        assert_eq!(
            resolve_static(&public(), "/about/").unwrap(),
            PathBuf::from("/srv/public/about/index.html")
        );
        // 目录不带尾斜杠时不补 index.html（Express redirect:false 的语义：交给运行时直出）
        assert_eq!(
            resolve_static(&public(), "/about").unwrap(),
            PathBuf::from("/srv/public/about")
        );
    }

    #[test]
    fn decodes_percent_encoded_segments() {
        assert_eq!(
            resolve_static(&public(), "/tags/%E7%BD%91%E7%BB%9C/").unwrap(),
            PathBuf::from("/srv/public/tags/网络/index.html")
        );
    }

    #[test]
    fn rejects_traversal() {
        assert!(resolve_static(&public(), "/../etc/passwd").is_none());
        assert!(resolve_static(&public(), "/%2e%2e/%2e%2e/etc/passwd").is_none());
        assert!(resolve_static(&public(), "/a/%2e%2e/b").is_none());
        assert!(resolve_static(&public(), "/a%5Cb").is_none());
    }

    #[test]
    fn resolves_images_under_base() {
        let base = Path::new("/srv/content/images");
        assert_eq!(
            resolve_file_under(base, "post-theme.svg").unwrap(),
            PathBuf::from("/srv/content/images/post-theme.svg")
        );
        assert_eq!(
            resolve_file_under(base, "sub/%E5%9B%BE.png").unwrap(),
            PathBuf::from("/srv/content/images/sub/图.png")
        );
        assert!(resolve_file_under(base, "../secret").is_none());
        assert!(resolve_file_under(base, "a/%2e%2e/b").is_none());
        assert!(resolve_file_under(base, "/").is_none());
    }

    #[test]
    fn cache_scope_matches_serve_ts() {
        let root = public();
        assert_eq!(
            cache_for(&root, Path::new("/srv/public/assets/main.b3027ab5.css")),
            ASSET_CACHE
        );
        assert_eq!(
            cache_for(&root, Path::new("/srv/public/assets/fa-solid-900.woff2")),
            ASSET_CACHE
        );
        assert_eq!(
            cache_for(&root, Path::new("/srv/public/index.html")),
            PAGE_CACHE
        );
        assert_eq!(
            cache_for(&root, Path::new("/srv/public/images/post.svg")),
            PAGE_CACHE
        );
        assert_eq!(
            cache_for(&root, Path::new("/srv/public/sitemap.xml")),
            PAGE_CACHE
        );
    }

    #[test]
    fn etag_matching_is_weak_and_list_aware() {
        assert!(etag_matches("W/\"1a-2b\"", "W/\"1a-2b\""));
        assert!(etag_matches("\"1a-2b\"", "W/\"1a-2b\""));
        assert!(etag_matches("W/\"other\", W/\"1a-2b\"", "W/\"1a-2b\""));
        assert!(etag_matches("*", "W/\"1a-2b\""));
        assert!(!etag_matches("W/\"other\"", "W/\"1a-2b\""));
    }

    #[test]
    fn etag_format_matches_node() {
        let dir = std::env::temp_dir().join("nobelog-serve-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("etag.txt");
        std::fs::write(&file, b"hello").unwrap();
        let meta = std::fs::metadata(&file).unwrap();
        let etag = weak_etag(&meta);
        // 亚毫秒按四舍五入（Node 的 Date 语义），ETag 才能和 Express 版逐字节一致
        let duration = meta.modified().unwrap().duration_since(UNIX_EPOCH).unwrap();
        let mtime_ms = (duration.as_micros() + 500) / 1000;
        assert!(etag.starts_with("W/\"5-"));
        assert!(etag.ends_with('"'));
        assert_eq!(etag, format!("W/\"5-{mtime_ms:x}\""));
    }

    #[test]
    fn recognizes_in_memory_assets() {
        assert_eq!(
            in_memory_asset("/assets/main.b3027ab5.css"),
            Some(InMemoryAsset::Css)
        );
        assert_eq!(
            in_memory_asset("/assets/main.d6018855.js"),
            Some(InMemoryAsset::Js)
        );
        assert_eq!(in_memory_asset("/assets/main.css"), None);
        assert_eq!(in_memory_asset("/assets/main.b3027ab5.woff2"), None);
        assert_eq!(in_memory_asset("/assets/main.zzzzzzzz.css"), None);
        assert_eq!(in_memory_asset("/assets/other.b3027ab5.css"), None);
        assert_eq!(in_memory_asset("/assets/main.b3027ab5.cssx"), None);
    }

    #[test]
    fn strong_etag_is_quoted_16_hex_and_content_sensitive() {
        let etag = strong_etag("<h1>hi</h1>");
        assert!(etag.starts_with('"') && etag.ends_with('"'));
        assert_eq!(etag.len(), 18);
        assert!(etag[1..17].chars().all(|ch| ch.is_ascii_hexdigit()));
        assert_ne!(etag, strong_etag("<h1>hello</h1>"));
        assert_eq!(etag, strong_etag("<h1>hi</h1>"));
    }

    /* ---------------------------- content/ 指纹（动态模式热更新） */

    /// 独立的临时目录：每个测试一个，Drop 时删掉，避免互相踩。
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            use std::sync::atomic::AtomicUsize;
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "nobelog-serve-{name}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(dir.join("content")).unwrap();
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// 普通文件（非链接）的指纹要素。
    fn entry(path: &str, len: u64, mtime_ns: i128) -> StampEntry {
        StampEntry {
            path: path.to_string(),
            len,
            mtime_ns,
            link_target: None,
        }
    }

    #[test]
    fn content_stamp_tracks_added_and_removed_files() {
        let dir = TempDir::new("stamp-files");
        let content = dir.path().join("content");
        let post = content.join("posts").join("hello.md");
        std::fs::create_dir_all(post.parent().unwrap()).unwrap();

        let empty = content_stamp(dir.path());
        assert!(!is_stale(empty, empty), "同一份空目录不能被判为过期");

        std::fs::write(&post, "hello").unwrap();
        let one = content_stamp(dir.path());
        assert!(is_stale(one, empty), "新增文章必须触发重建");
        assert_eq!(one, content_stamp(dir.path()), "内容不变时指纹必须稳定");

        std::fs::write(content.join("draft.md"), "draft").unwrap();
        let two = content_stamp(dir.path());
        assert!(is_stale(two, one), "再新增文件必须触发重建");

        // 删除文章：剩余文件的最大 mtime 不变，旧实现永远检测不到。
        std::fs::remove_file(&post).unwrap();
        let after_remove = content_stamp(dir.path());
        assert!(is_stale(after_remove, two), "删除文章必须触发重建");
        assert_ne!(after_remove, one, "文件集合不同，指纹不能相同");
    }

    #[test]
    fn content_stamp_sees_size_change_with_same_mtime() {
        // 不引入依赖就没法把 mtime 改回旧值，所以在「折叠成指纹」这一层验证：
        // 路径与 mtime 完全相同、只有大小不同，指纹必须不同。
        let at: i128 = 1_700_000_000_000_000_000;
        let small = stamp_digest(vec![entry("posts/a.md", 10, at)]);
        let larger = stamp_digest(vec![entry("posts/a.md", 11, at)]);
        assert_ne!(small, larger, "同 mtime 下大小变化必须触发重建");
        assert!(is_stale(larger, small));
        assert!(!is_stale(small, small));
    }

    #[test]
    fn stamp_digest_tracks_link_target() {
        // 链接目标也要算身份：否则「改指向另一个大小/mtime 恰好相同的文件」会漏检。
        let plain = entry("posts/linked.md", 10, 5);
        let mut linked_a = plain.clone();
        linked_a.link_target = Some("/tmp/a.md".to_string());
        let mut linked_b = plain.clone();
        linked_b.link_target = Some("/tmp/b.md".to_string());

        assert_ne!(
            stamp_digest(vec![linked_a.clone()]),
            stamp_digest(vec![linked_b.clone()]),
            "链接目标不同必须触发重建"
        );
        assert_ne!(
            stamp_digest(vec![linked_a]),
            stamp_digest(vec![plain]),
            "链接与普通文件不能同指纹"
        );
    }

    #[cfg(unix)]
    #[test]
    fn content_stamp_follows_symlinked_posts() {
        use std::os::unix::fs::symlink;

        let dir = TempDir::new("stamp-link");
        let content = dir.path().join("content");
        std::fs::create_dir_all(content.join("posts")).unwrap();
        // 链接目标放在 content/ 之外，模拟「仓库里只放链接、正文在别处」。
        let target = dir.path().join("outside").join("post.md");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "第一版").unwrap();
        symlink(&target, content.join("posts").join("linked.md")).unwrap();

        let linked = content_stamp(dir.path());
        assert_eq!(linked, content_stamp(dir.path()), "链接未变时指纹稳定");

        let empty_dir = TempDir::new("stamp-link-empty");
        let empty = content_stamp(empty_dir.path());
        assert_ne!(linked, empty, "链接文章必须计入指纹（新增链接也要重建）");

        // 目标文件变大 / mtime 变化 → 指纹必须变化（不跟随链接的旧实现恒为 linked）。
        std::fs::write(&target, "第二版（更长一些）").unwrap();
        assert_ne!(
            content_stamp(dir.path()),
            linked,
            "链接目标改动必须触发重建"
        );

        // 目标被删 → 链接变 broken：跳过即可，不 panic，指纹退回「没有这个文件」。
        std::fs::remove_file(&target).unwrap();
        assert_eq!(
            content_stamp(dir.path()),
            empty,
            "broken 链接必须被稳定跳过，且不 panic"
        );

        // content/ 内的目录符号链接不展开、不计入指纹（walkdir 默认不跟随目录链接，
        // content.rs 也是逐目录 read_dir，两边范围一致，也避免链接成环）。
        let shared_dir = dir.path().join("outside-dir");
        std::fs::create_dir_all(&shared_dir).unwrap();
        std::fs::write(shared_dir.join("nested.md"), "nested").unwrap();
        symlink(&shared_dir, content.join("posts").join("linked-dir")).unwrap();
        assert_eq!(
            content_stamp(dir.path()),
            empty,
            "目录链接不展开、不计入指纹"
        );
    }

    #[test]
    fn stamp_digest_is_order_independent_and_path_sensitive() {
        let entries = vec![entry("posts/a.md", 10, 1), entry("posts/b.md", 20, 2)];
        let mut reversed = entries.clone();
        reversed.reverse();
        assert_eq!(
            stamp_digest(entries.clone()),
            stamp_digest(reversed),
            "遍历顺序不同不能算出不同指纹"
        );

        let renamed = vec![entry("posts/a.md", 10, 1), entry("posts/c.md", 20, 2)];
        assert_ne!(
            stamp_digest(entries),
            stamp_digest(renamed),
            "改名（集合变化）必须触发重建"
        );
    }

    #[test]
    fn stamp_cache_expires_after_ttl() {
        let now = Instant::now();
        assert!(!stamp_cache_expired(now, now), "刚检查过应复用缓存");
        assert!(!stamp_cache_expired(now - Duration::from_millis(500), now));
        assert!(stamp_cache_expired(now - STAMP_TTL, now), "到 TTL 必须重查");
        assert!(stamp_cache_expired(now - Duration::from_secs(30), now));
    }

    #[test]
    fn rebuild_backoff_only_while_content_is_unchanged() {
        let now = Instant::now();
        let failed = ContentStamp(7);
        let changed = ContentStamp(8);

        assert!(
            reload_backed_off(Some((failed, now - Duration::from_secs(1))), failed, now),
            "同一份坏内容在窗口内不再重试"
        );
        assert!(
            !reload_backed_off(Some((failed, now - Duration::from_secs(1))), changed, now),
            "内容变了要立刻允许重试（用户可能已经修好）"
        );
        assert!(
            !reload_backed_off(Some((failed, now - RELOAD_BACKOFF)), failed, now),
            "退避窗口结束后必须重试"
        );
        assert!(!reload_backed_off(None, failed, now), "没失败过就不退避");
    }

    #[test]
    fn dynamic_images_use_mutable_cache() {
        // content/images/** 可以被随时替换，绝不能用一年强缓存。
        assert_eq!(dynamic_file_cache("/images/post.svg"), NO_CACHE);
        assert_eq!(dynamic_file_cache("/images/sub/图.png"), NO_CACHE);
        assert_ne!(dynamic_file_cache("/images/post.svg"), ASSET_CACHE);
        // 构建期固定的 vendor 字体保持长缓存。
        assert_eq!(
            dynamic_file_cache("/assets/fa-solid-900.woff2"),
            ASSET_CACHE
        );
    }

    fn headers_with(content_type: &'static str, length: Option<u64>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
        if let Some(length) = length {
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&length.to_string()).unwrap(),
            );
        }
        headers
    }

    fn should_compress(content_type: &'static str, length: Option<u64>) -> bool {
        should_compress_response(
            StatusCode::OK,
            Version::HTTP_11,
            &headers_with(content_type, length),
            &Extensions::new(),
        )
    }

    #[test]
    fn compresses_text_like_types_only() {
        for content_type in [
            "text/html; charset=utf-8",
            "text/css; charset=UTF-8",
            "text/plain; charset=UTF-8",
            "application/javascript; charset=UTF-8",
            "application/json; charset=UTF-8",
            "application/xml; charset=utf-8",
            "application/rss+xml; charset=utf-8",
            "image/svg+xml",
        ] {
            assert!(should_compress(content_type, Some(4096)), "{content_type}");
        }
        for content_type in [
            "font/woff2",
            "image/png",
            "image/jpeg",
            "application/octet-stream",
        ] {
            assert!(!should_compress(content_type, Some(4096)), "{content_type}");
        }
    }

    #[test]
    fn skips_small_partial_and_not_modified_responses() {
        assert!(!should_compress("text/html; charset=utf-8", Some(1023)));
        assert!(should_compress("text/html; charset=utf-8", Some(1024)));
        assert!(!should_compress("text/html; charset=utf-8", None));
        let headers = headers_with("text/html; charset=utf-8", Some(4096));
        assert!(!should_compress_response(
            StatusCode::PARTIAL_CONTENT,
            Version::HTTP_11,
            &headers,
            &Extensions::new()
        ));
        assert!(!should_compress_response(
            StatusCode::NOT_MODIFIED,
            Version::HTTP_11,
            &headers,
            &Extensions::new()
        ));
    }

    #[test]
    fn vary_header_is_added_once() {
        let mut headers = HeaderMap::new();
        ensure_accept_encoding_vary(&mut headers);
        assert_eq!(headers.get_all(header::VARY).iter().count(), 1);
        ensure_accept_encoding_vary(&mut headers);
        assert_eq!(headers.get_all(header::VARY).iter().count(), 1);

        let mut existing = HeaderMap::new();
        existing.insert(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding, Origin"),
        );
        ensure_accept_encoding_vary(&mut existing);
        assert_eq!(existing.get_all(header::VARY).iter().count(), 1);

        let mut other = HeaderMap::new();
        other.insert(header::VARY, HeaderValue::from_static("Origin"));
        ensure_accept_encoding_vary(&mut other);
        assert_eq!(other.get_all(header::VARY).iter().count(), 2);
    }

    fn range_header(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, HeaderValue::from_static(value));
        headers
    }

    #[test]
    fn parses_single_byte_ranges() {
        assert_eq!(parse_range(&HeaderMap::new(), 1000), RangeRequest::Whole);
        assert_eq!(
            parse_range(&range_header("bytes=0-99"), 1000),
            RangeRequest::Partial(0, 99)
        );
        assert_eq!(
            parse_range(&range_header("bytes=990-"), 1000),
            RangeRequest::Partial(990, 999)
        );
        assert_eq!(
            parse_range(&range_header("bytes=-64"), 1000),
            RangeRequest::Partial(936, 999)
        );
        assert_eq!(
            parse_range(&range_header("bytes=0-99999"), 1000),
            RangeRequest::Partial(0, 999)
        );
        assert_eq!(
            parse_range(&range_header("bytes=1000-"), 1000),
            RangeRequest::Unsatisfiable
        );
        assert_eq!(
            parse_range(&range_header("bytes=-0"), 1000),
            RangeRequest::Unsatisfiable
        );
        assert_eq!(
            parse_range(&range_header("bytes=0-0,2-3"), 1000),
            RangeRequest::Whole
        );
        assert_eq!(
            parse_range(&range_header("bytes=abc"), 1000),
            RangeRequest::Whole
        );
        assert_eq!(
            parse_range(&range_header("items=0-1"), 1000),
            RangeRequest::Whole
        );
    }

    /* ---------------------------- request_base_url / Host 校验 */

    /// 恶意 Host 必须整条丢弃、回退到 fallback，绝不能出现在返回值里。
    const FALLBACK_URL: &str = "http://127.0.0.1:10086";

    fn request_headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(*name, HeaderValue::from_static(value));
        }
        headers
    }

    #[test]
    fn base_url_uses_host_and_forwarded_proto() {
        // 正常反代：X-Forwarded-Proto: https + 带端口的 host。
        let headers =
            request_headers(&[("host", "example.com:8080"), ("x-forwarded-proto", "https")]);
        assert_eq!(
            base_url_from_headers(&headers, FALLBACK_URL),
            "https://example.com:8080"
        );

        // 没有 XFP 时按 http；X-Forwarded-Host 优先于 Host；逗号列表只取第一个。
        let headers = request_headers(&[
            ("host", "origin.example"),
            ("x-forwarded-host", "example.com:8080, proxy.internal"),
        ]);
        assert_eq!(
            base_url_from_headers(&headers, FALLBACK_URL),
            "http://example.com:8080"
        );

        // IPv6 方括号主机 + 端口。
        let headers = request_headers(&[("host", "[::1]:10086")]);
        assert_eq!(
            base_url_from_headers(&headers, FALLBACK_URL),
            "http://[::1]:10086"
        );
    }

    #[test]
    fn base_url_rejects_injected_hosts() {
        for evil in [
            // 试图闭合并跳出 href / 属性
            "x\"><img src=x>",
            // userinfo 注入：真实 host 是 evil.com
            "good.com:80@evil.com",
            // 把路径拼进主机
            "evil.com/",
            "evil.com\\path",
            // 空白
            "evil .com",
            // 结构字符
            "evil.com;x",
            "evil.com(x)",
        ] {
            // X-Forwarded-Host 非法且没有 Host：整条丢弃，回退 fallback。
            let headers = request_headers(&[("x-forwarded-host", evil)]);
            let base = base_url_from_headers(&headers, FALLBACK_URL);
            assert_eq!(base, FALLBACK_URL, "恶意 XFH 未回退：{evil:?} → {base}");
            assert!(!base.contains(evil), "恶意值进入了 base_url：{evil:?}");

            // Host 头同样要校验。
            let headers = request_headers(&[("host", evil)]);
            let base = base_url_from_headers(&headers, FALLBACK_URL);
            assert_eq!(base, FALLBACK_URL, "恶意 Host 未回退：{evil:?} → {base}");
            assert!(!base.contains(evil), "恶意值进入了 base_url：{evil:?}");
        }

        // Host 是合法值时，非法的 X-Forwarded-Host 只是被丢弃，回退到 Host。
        let headers = request_headers(&[
            ("x-forwarded-host", "good.com:80@evil.com"),
            ("host", "example.com"),
        ]);
        assert_eq!(
            base_url_from_headers(&headers, FALLBACK_URL),
            "http://example.com"
        );

        // 控制字符：HeaderValue 允许 tab，但 host 白名单不接受（内部 tab 不会被 trim 掉）。
        let value = HeaderValue::from_bytes(b"evil\tcom").unwrap();
        assert!(validated_host(&value).is_none());
        for ch in ['\u{0}', '\u{1}', '\t', '\n', '\r', ' ', '\u{7f}'] {
            assert!(!is_host_char(ch), "控制字符/空白通过了白名单：{ch:?}");
        }
    }

    #[test]
    fn base_url_ignores_unknown_forwarded_proto() {
        // XFP 只认精确的 http / https；file 之类当没写。
        let headers = request_headers(&[("host", "example.com"), ("x-forwarded-proto", "file")]);
        assert_eq!(
            base_url_from_headers(&headers, FALLBACK_URL),
            "http://example.com"
        );

        let headers = request_headers(&[("host", "example.com"), ("x-forwarded-proto", "HTTPS")]);
        assert_eq!(
            base_url_from_headers(&headers, FALLBACK_URL),
            "http://example.com"
        );

        let headers = request_headers(&[
            ("host", "example.com"),
            ("x-forwarded-proto", "https, http"),
        ]);
        assert_eq!(
            base_url_from_headers(&headers, FALLBACK_URL),
            "https://example.com"
        );
    }
}
