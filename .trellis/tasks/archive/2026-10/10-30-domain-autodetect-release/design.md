# 技术设计：域名自动识别维护与 v0.0.30 发版

## 1. 背景与问题

现状优先级：`SITE_URL` 最高；动态模式默认只读 `Host` 且 scheme 固定 `http`；只有 `NOBELOG_TRUST_FORWARDED` 为显式真值时才全量采信 `X-Forwarded-*`。

需要修复的问题：

1. 客户生产机在 TLS 反代后未配置 `SITE_URL` 时，canonical、og、RSS、sitemap 全部写成 `http://域名`。
2. `README.md:51` 与 `routes-and-urls.md:52-55` 仍写按 `Host` 与 `X-Forwarded-Proto` 自动推断，与 v0.0.24 后行为不一致。
3. `SITE_URL` 优先级、请求推断组合、真实反代链路缺测试。
4. example.com 已下线，但 `build.rs` 测试样例与 `package.sh` 仍默认沿用旧域名或本机地址，容易误打包。

## 2. 关键决策

1. **peer 规则**：动态模式下，客户端 socket 地址为环回或私网时，仅额外采信 `X-Forwarded-Proto`；`Host` 仍只由 `SITE_URL` 或总开关决定，`X-Forwarded-Host` 默认不采信。`NOBELOG_TRUST_FORWARDED=1` 保留为全量信任。
2. **信任维度拆分**：`base_url_from_headers` 的单一布尔拆成 host 与 proto 两个参数，避免 peer 规则意外放开域名来源。
3. **peer 获取**：使用 axum 0.8 的 `ConnectInfo<SocketAddr>`，把 `axum::serve(listener, app)` 换成 `app.into_make_service_with_connect_info::<SocketAddr>()`，在动态 handler 的请求扩展里读取，无需新增依赖。
4. **发布包静态基准**：example.com 视为已下线，打包用中性示例域名 `https://example.com`；`package.sh` fail-fast 拒绝空值、http、example.com 与 127.0.0.1。
5. **版本**：0.0.30，两源同步，属于可观察的默认行为变化，Release notes 必须说明。

## 3. 行为矩阵

| peer 地址 | 总开关 | 域名来源 | scheme 来源 | 结果 |
|---|---|---|---|---|
| 任意 | 开 | `X-Forwarded-Host` 优先，其次 `Host` | `X-Forwarded-Proto` | 与 v0.0.24 信任模式一致 |
| 环回或私网 | 关 | `Host` | `X-Forwarded-Proto`，缺失时 http | 零配置还原 https |
| 环回或私网 | 关 | `Host` | 无 XFP | 与现状一致 |
| 公网 | 关 | `Host` | 固定 http | 与现状一致，伪造 XFP 无效 |
| 任意 | 关 | 无有效 Host | 固定 http，回退 `SITE.url` | 与现状一致 |

信任范围：IPv4 环回 `127.0.0.0/8`、IPv6 环回 `::1`、RFC1918 `10/8`、`172.16/12`、`192.168/16`、IPv6 ULA `fc00::/7`、链路本地 `169.254/16` 与 `fe80::/10`；`None` 一律不信任。

安全取舍：私网内其他客户端仍可伪造 scheme，影响仅限协议，不涉及域名劫持；公网直连伪造无效。此取舍写入 spec。

## 4. 代码改动

`rust/src/serve.rs`：

```rust
fn request_base_url(
    headers: &HeaderMap,
    fallback: &str,
    peer: Option<std::net::IpAddr>,
) -> String {
    if let Some(url) = crate::site::env_url() {
        return url;
    }
    let trust_all = trust_forwarded_headers();
    let trust_proto = trust_all || peer_is_loopback_or_private(peer);
    base_url_from_headers(headers, fallback, trust_all, trust_proto)
}

fn peer_is_loopback_or_private(peer: Option<std::net::IpAddr>) -> bool { /* 见上表 */ }

fn base_url_from_headers(
    headers: &HeaderMap,
    fallback: &str,
    trust_host: bool,
    trust_proto: bool,
) -> String
```

- `run()` 改用 `into_make_service_with_connect_info::<SocketAddr>()`。
- `handle_dynamic` 从请求扩展读取 `ConnectInfo<SocketAddr>`，把 `Option<IpAddr>` 传入 `render_dynamic` 与 `request_base_url`。
- 监听器当前用 `tap_io` 设置 TCP_NODELAY；若与 `ConnectInfo` 不兼容，去掉该包装并在需要的层级另行设置 nodelay，保证连接信息可用，实现阶段以编译与测试为准。

`rust/src/site.rs`：更新未设置 `SITE_URL` 的启动告警文案，说明默认规则、peer 规则与两个开关的关系。

`tools/package.sh`：

1. `SITE_URL` 必填，必须以 `https://` 开头，拒绝包含 `example.com` 与 `127.0.0.1`，打印实际值。
2. `VERSION` 可显式传入；校验与 `rust/Cargo.toml` 的 `version` 一致，打印实际值。
3. 其余构建、打包、SHA256SUMS 行为不变。

## 5. 测试计划

新增或更新：

1. peer 判定矩阵：环回、私网、链路本地、ULA 为真；公网、`None` 为假。
2. 私网 peer 加 XFP https 得 https scheme，同时伪造 XFH 不改域名。
3. 公网 peer 伪造 XFP 与 XFH 均无效。
4. `SITE_URL` 最高优先于 peer 规则与请求头。
5. 总开关开启时公网 peer 恢复全量信任，保持兼容。
6. 既有 5 组转发头测试适配新签名，注入类用例保持回退。
7. `build.rs` robots 测试样例域名改中性域名，补 `SITE_URL` 尾斜杠 trim 断言。

## 6. 发版与上线

1. 质量门禁：`cargo fmt --check && cargo clippy --all-targets && cargo test`。
2. 基准重建：`SITE_URL=https://example.com ... build --self-assets --out public`，连续两次构建哈希一致；同参数重建 `rust/dist`，`nobelog diff public rust/dist` 只剩预期差异。
3. 打包：`VERSION=0.0.30 SITE_URL=https://example.com tools/package.sh`，校验 sha256、包内 VERSION、二进制 `--version`。
4. 提交与 tag：代码提交、版本提交、发版提交均带任务锚点；annotated tag `v0.0.30`；push main 与 tag；`gh release create` 附包与 SHA256SUMS。
5. 上线：`cargo build --release --locked`；`systemctl restart nobelog-local`；冒烟 9 条路由与 404、图标、canonical、环回 peer 加 XFP、CSS 哈希。
6. 回滚：保留 `dist/nobelog-0.0.29` 包；异常时解包旧二进制重启，或 checkout v0.0.29 重建。

## 7. 兼容性

- 默认行为变化：环回或私网 peer 的 `X-Forwarded-Proto` 现在会生效；公网 peer 与 `X-Forwarded-Host` 行为不变。
- 静态构建行为不变，绝对地址仍由构建期 `SITE_URL` 决定。
- 发布包内静态 `public/` 使用示例域名 `example.com`，只作示例；静态部署的客户必须用自己 `SITE_URL` 重建。
