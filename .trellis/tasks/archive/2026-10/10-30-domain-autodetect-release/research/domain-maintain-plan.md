# Research: 域名自动识别维护与发版改动清单

- Query: 维护域名自动识别逻辑并修复 2.3.4 核查发现的问题，随后发版；旧域名已下线
- Scope: mixed，内部代码与文档盘点、发布流程事实、axum 可行性评估、本地服务只读核对
- Date: 2026-10-30
- 核查时刻: 2026-10-30T11:35Z 至 11:45Z，命令在仓库根 `/root/博客/nobelog` 执行
- 约束: 全程只读；未改代码、配置、spec、任务文件；未 commit/push/fetch；未重启服务；未新起实例；未访问公网博客域名；GitHub 仅只读查询

## 一、结论摘要

1. **硬编码盘点**：旧域名虽然已下线，仓库里仍有 19 个已跟踪文件共 48 处；另有被忽略的 `public/` 与 `rust/dist/` 各 82 处，历史发布包内 82 处。已跟踪部分真正需要随这次改动处理的是 `rust/src/build.rs:422-458` 的 7 处测试样例与 `.trellis/spec/frontend/build-serve-deploy.md:175` 的冻结基准说明；其余 41 处集中在历史归档任务、journal 与旧研究报告，建议保留为历史记录，不改写。`content/`、`README.md`、`tools/` 本身没有旧域名。
2. **发布流程事实**：`tools/package.sh` 在打包时用当前环境变量 `SITE_URL` 重建一次 `public/` 再打包；脚本不校验 `SITE_URL`，缺失时构建退回 `SITE.url` 的 `http://127.0.0.1:10086`。版本来源是 `rust/Cargo.toml` 与 `rust/Cargo.lock` 两处；包名版本来自 `VERSION` 环境变量或 `git describe`，与二进制版本可能脱节。tag、GitHub Release 与本机重启是 spec 里的手工步骤。
3. **代码可行性**：`rust/Cargo.lock` 显示 axum 0.8.9、tower 0.5.3，当前代码完全未使用 `ConnectInfo`。把 `run()` 里的 `axum::serve(listener, app)` 改为 `app.into_make_service_with_connect_info::<SocketAddr>()`，在动态 handler 通过 `req.extensions()` 读取 peer 地址即可；改动集中在 `rust/src/serve.rs`，无新增依赖。需要把 `base_url_from_headers` 的单一信任布尔拆成 host 与 proto 两个开关。
4. **方案推荐**：采用「对端地址是环回或私网时，只额外采信 `X-Forwarded-Proto` 用于 scheme；`Host` 仍只由 `SITE_URL` 或显式总开关决定」，保留 `NOBELOG_TRUST_FORWARDED=1` 作为全量信任开关。理由：同机或局域网 TLS 反代可以零配置得到正确 https；`X-Forwarded-Host` 域名劫持面不扩大；公网对端伪造 XFP 无效。
5. **最小改动集**：代码 2 个文件，即 `serve.rs` 与 `site.rs` 告警文案；测试约 6 组新增并更新既有 5 组；文档 3 个文件，即 `README.md`、`routes-and-urls.md`、`build-serve-deploy.md`；发布配置为 `tools/package.sh` 增加强制校验，并把版本两源升到 0.0.30。
6. **发版硬前置**：新域名必须先确定。否则新包内静态 `public/` 的 canonical 只能是 `127.0.0.1:10086` 或已下线的旧域名；`public/` 与 `rust/dist/` 必须用新域名重建。旧 `dist` 的 v0.0.29 包与 GitHub v0.0.29 保留为历史，不改。本机重启与推送都需要用户另行授权。

## 二、硬编码盘点

### 1. example.com

已跟踪文件统计：19 个文件、48 处。被忽略产物与历史包另计。

| 分类 | 位置 | 数量 | 处置建议 |
|---|---|---|---|
| 源码测试 | `rust/src/build.rs:422-458` | 7 处 | 改为中性样例域名，如 `https://example.com`，去掉对下线域名的依赖 |
| 活跃 spec | `.trellis/spec/frontend/build-serve-deploy.md:175` | 1 处 | 随新域名重建后更新，明确 旧域名已下线 |
| 归档任务与旧报告 | `.trellis/tasks/archive/2026-10/**` | 15 个文件、32 处 | 历史记录，保留不改；新任务与 journal 说明即可 |
| workspace 历史 | `.trellis/workspace/zhemed/index.md`、`journal-1.md` | 2 个文件、8 处 | 历史记录，保留不改 |
| 本地冻结基准 | `public/` 42 个文件 | 82 处 | 新域名确定后用新 `SITE_URL` 重建 |
| 过期构建 | `rust/dist/` 42 个文件 | 82 处 | 删除或按新 `SITE_URL` 重建，不要用于验收 |
| 历史发布包 | `dist/nobelog-0.0.29-linux-x86_64.tar.gz` 内 17 个 public 成员 | 82 处 | 保留为历史包，不修改；新版本重新打包 |
| GitHub Release | v0.0.29 及更早资产的包内 public | — | 历史资产，保留不改 |
| 内容与工具 | `content/`、`README.md`、`tools/` | 0 处 | 无需处理 |

补充事实：

- 当前 `public/index.html`、`public/posts/typeset-test/index.html`、`rust/dist/index.html` 的 canonical 都是 `https://example.com/`。
- `public/` 构建于 2026-10-21 15:28，`rust/dist/` 构建于 2026-10-30 10:19，两者都含下线域名。
- git 历史里有 12 个提交改动过 example.com 相关行，全部是历史发布、核对与文档记录，不需要重写历史。
- `content/` 中没有 example.com 或 example.com 引用，域名下线不影响文章内容。

### 2. SITE_URL

已跟踪统计：39 个文件、137 处。有效位置分类如下。

| 分类 | 位置 | 数量 | 处置建议 |
|---|---|---|---|
| 源码 | `rust/src/site.rs` 4 处、`rust/src/serve.rs` 3 处、`rust/src/build.rs` 6 处、`rust/src/templates.rs` 1 处 | 14 处 | 保留；随方案更新注释与告警文案，逻辑拆成 host 与 proto 两个信任维度 |
| 测试 | `rust/src/build.rs` 的 robots 用例以参数注入 base_url；`serve.rs` 测试未读取环境变量 | 0 处直接环境测试 | 新增 `SITE_URL` 优先级与纯函数注入测试 |
| 活跃文档 | `build-serve-deploy.md` 6 处、`routes-and-urls.md` 5 处、`rendering-templates.md` 3 处、`quality-guidelines.md` 2 处、`index.md` 1 处 | 17 处 | 更新部署与优先级相关的前两个文件；其余为示例或索引描述，可保留 |
| 工具脚本 | `tools/package.sh` | 0 处 | 新增强制设置与校验 |
| 历史任务与 journal | `.trellis/tasks/archive/**`、`.trellis/workspace/**` | 其余约 106 处 | 保留历史，不改写 |
| 部署配置 | systemd 单元 `nobelog-local.service` 的 `Environment=` 为空 | — | 发版上线时另行决定是否注入新域名，需用户授权 |

### 3. 127.0.0.1:10086

| 位置 | 用途 | 处置建议 |
|---|---|---|
| `rust/src/site.rs:58` | `SITE.url` 内置默认值，开发态回退 | 保留；生产不得依赖，告警文案可说明这是开发回退值 |
| `rust/src/serve.rs:1711` | 测试常量 `FALLBACK_URL` | 保留 |
| `rust/src/build.rs:437` | robots 测试断言默认地址 | 保留 |
| `content/posts/typeset-test.md:60` | 排版测试文章里的示例命令 | 保留，属内容正文 |
| `public/` 与 `rust/dist/` 的 `posts/typeset-test/index.html` | 上述内容的静态渲染 | 随重建保留 |
| 历史任务、journal、研究报告 | 历史记录 | 保留 |

结论：`127.0.0.1:10086` 是开发默认值与测试样例，不是需要下线的生产域名，本次无需改动；只要保证生产部署时优先使用 `SITE_URL`。

### 4. NOBELOG_TRUST_FORWARDED

| 位置 | 用途 | 处置建议 |
|---|---|---|
| `rust/src/serve.rs:314`、`322-325`、`327-335` | 注释、环境变量读取、取值解析 | 保留原语义，作为全量信任开关；配合新的 peer 规则调整注释 |
| `rust/src/site.rs:51` | 启动告警文案中的开关说明 | 更新文案，说明默认规则与开关关系 |
| `.trellis/spec/frontend/build-serve-deploy.md:74`、`93` | 部署文档中的开关说明 | 随新默认行为更新 |
| 历史任务与 journal | 历史记录 | 保留 |

## 三、发布流程事实

### 1. tools/package.sh 逐段事实

```bash
 5  cd "$(dirname "$0")/.."
 7  VERSION="${VERSION:-$(git describe --tags --always --dirty | sed 's/^v//')}"
11  /root/.cargo/bin/cargo build --release --locked --manifest-path rust/Cargo.toml
12  ./rust/target/release/nobelog build --self-assets --out public >/dev/null
17  cp -a content vendor public "$stage/"
22  mkdir -p "$OUT"
23  tar -C "$(dirname "$stage")" -czf "$OUT/$NAME.tar.gz" "$NAME"
24  (cd "$OUT" && sha256sum "$NAME.tar.gz" > SHA256SUMS)
```

- 每次打包都会重建仓库根的 `public/`，见第 12 行；`--out public` 相对当前工作目录解析，脚本已 cd 到仓库根。
- `SITE_URL` 只从调用环境继承，脚本不设置、不校验、不打印；未设置时 `build.rs:66` 回退 `SITE.url`，即 `http://127.0.0.1:10086`。
- 包内 `public/` 的 canonical、feed、sitemap、robots 等绝对地址全部来自打包时那次构建的 `SITE_URL`；动态模式运行时不会读包内 public 的页面，静态部署才会读。
- 版本号来源两处：`rust/Cargo.toml:3` 与 `rust/Cargo.lock:765-766`，决定二进制 `--version`；包名来源是 `VERSION` 环境变量或 `git describe`，两者可能不一致。
- 第 11 行把 cargo 路径写死为 `/root/.cargo/bin/cargo`，换用户或换机器会失败，属可移植性隐患，不在本次最小改动内。
- 脚本没有 `SITE_URL` 校验、没有 `VERSION` 与 Cargo 版本一致性校验、没有干净工作区检查、没有 tag 与 GitHub Release 步骤。

### 2. 发版 spec 固定动作

`.trellis/spec/frontend/build-serve-deploy.md:184-207` 定义了七步：

```text
1. 版本同步：rust/Cargo.toml 与 rust/Cargo.lock 两处一起改
2. 门禁：cargo fmt --check / clippy --all-targets / cargo test
3. 打包：VERSION=<x.y.z> tools/package.sh，校验 sha256 与 --version
4. 提交与 tag：提交带任务锚点，打 annotated tag v<x.y.z>，push main 与 tag
5. GitHub Release：gh release create 附二进制与 SHA256SUMS
6. 上线：cargo build --release --locked 后 systemctl restart nobelog-local（需用户授权）
7. 留痕：Trellis journal 与任务归档
```

注意第 3 步的示例命令只写了 `VERSION=<x.y.z> tools/package.sh`，没有写 `SITE_URL`；历史上实际发版是靠操作者手工带上 `SITE_URL=https://example.com`，这属于流程文档缺口。spec 第 212 行要求版本字段与 tag 一致，但 package.sh 不校验。

### 3. 版本来源与当前状态

| 项 | 值 | 证据 |
|---|---|---|
| Cargo.toml 版本 | 0.0.29 | `rust/Cargo.toml:3` |
| Cargo.lock 版本 | 0.0.29 | `rust/Cargo.lock:765-766` |
| 运行二进制版本 | 0.0.29 | `./rust/target/release/nobelog --version` |
| 最新 git tag | v0.0.29 | `git tag --sort=-v:refname` |
| 当前 HEAD | 99b138f，`v0.0.29-8-g99b138f` | `git describe --tags --always` |
| 本地相对远程 | ahead 8，behind 0 | `git status --short --branch` 与 `origin/main=28f80ea` |
| 最新 GitHub Release | v0.0.29 | `gh release list` |

如果 0.0.30 打包时不显式传 `VERSION=0.0.30`，包名会变成 `nobelog-0.0.29-8-g99b138f-linux-x86_64.tar.gz`，与二进制 `nobelog 0.0.30` 不一致。

### 4. 包内绝对地址来源与 public 重建

- 静态构建：`build.rs:66` 的 `base_url = env_url().unwrap_or(SITE.url)`，构建期固化到每个 HTML 的 canonical、og:url、og:image、页脚与分享链接，以及 feed、sitemap、robots。
- 动态模式：`serve.rs:315-320` 的 `request_base_url` 每次请求推断，SITE_URL 优先，否则按 Host 与信任规则。
- 当前冻结基准：`public/` 用 `https://example.com` 构建，42 个文件，`spec:175` 明确写着基线域名是 example.com，复跑 `nobelog diff` 必须对齐同一个 `SITE_URL`。
- 打包第 12 行重建 `public/` 等于覆盖冻结基准，这是现有设计的一部分；一旦传错 `SITE_URL`，本地基准会被污染，后续 diff 全部失真。

## 四、代码与方案对比

### 1. 现有实现链

| 位置 | 行为 |
|---|---|
| `rust/src/site.rs:37-42` | `env_url()` 读 `SITE_URL`，trim 并去尾部斜杠，空值视为未设置 |
| `rust/src/site.rs:47-55` | 未设置时启动告警一次 |
| `rust/src/serve.rs:257-311` | `run()` 启动服务；第 306 行 `axum::serve(listener, app)` |
| `rust/src/serve.rs:313-320` | `request_base_url(headers, fallback)`，`SITE_URL` 优先 |
| `rust/src/serve.rs:322-335` | `NOBELOG_TRUST_FORWARDED` 读取与解析 |
| `rust/src/serve.rs:337-373` | `base_url_from_headers(headers, fallback, trust_forwarded)`，单一信任布尔 |
| `rust/src/serve.rs:375-388` | `validated_host` 与 `is_host_char` 白名单 |
| `rust/src/serve.rs:580-653` | 动态 handler，只克隆请求头，没有 peer 信息 |
| `rust/src/serve.rs:656-663` | `render_dynamic` 注入 `base`，没有 peer 参数 |
| `rust/src/build.rs:66` | 静态构建的 base_url |
| `rust/src/routes.rs:56` | `Site::new` 时固化 `root_url`，静态与回退页面使用 |

当前没有任何 `ConnectInfo`、`SocketAddr`、`IpAddr` 使用，也没有客户端地址相关的测试。

### 2. axum ConnectInfo 可行性

事实：`rust/Cargo.lock` 中 axum 0.8.9、tower 0.5.3；tokio 已启用 `net` feature；当前 `axum::serve` 直接吃 Router。

可行改动：

1. `run()` 第 306 行改为启动带连接信息的服务：

```rust
axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
)
```

2. 动态 handler 通过请求扩展取 peer，不需要改函数签名也能拿到：

```rust
let peer = req
    .extensions()
    .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
    .map(|info| info.0.ip());
```

3. 把 peer 继续传给 `render_dynamic` 与 `request_base_url`。

待验证点，实现阶段必须以编译为准：

- 当前监听器经过 `axum::serve::ListenerExt::tap_io` 包装，用于设置 TCP_NODELAY。`into_make_service_with_connect_info` 依赖 `Connected` 语义，需确认 `TapIo` 包装后仍能提供对端地址。若无法编译或取不到地址，备选方案是去掉 `tap_io`，在自定义 accept 循环里设置 nodelay，或保留 `tap_io` 但改用 axum 其他连接信息入口。
- 该能力属 axum 0.8 的标准入口，不需要新增 crate；`SocketAddr` 来自标准库。

### 3. 方案对比

| 方案 | 改动量 | 同机 TLS 反代且未配置 | 私网反代且未配置 | 公网对端伪造 XFP | XFH 域名劫持面 | 推荐度 |
|---|---|---|---|---|---|---|
| 方案一：保持现状，仅修文档 | 0 代码 | scheme 错误 http | scheme 错误 http | 无效 | 不扩大 | 低，未解决客户问题 |
| 方案二：peer 环回或私网时只采信 XFP | 小，serve.rs 一个文件 | 正确 https | 正确 https | 无效 | 不扩大，Host 仍严格 | 推荐 |
| 方案三：默认采信任意对端 XFP | 极小，一行 | 正确 https | 正确 https | 可把 scheme 伪造成 https，只影响协议不影响域名 | 不扩大 | 不推荐，收益与风险不匹配 |
| 方案四：新增 proto 专用环境开关 | 小 | 需配置 | 需配置 | 无效 | 不扩大 | 可作为补充，不解决零配置 |

补充说明：方案三的风险是任意客户端都能把 canonical 从 http 改成 https，虽然不涉及域名劫持，但属于用不可信输入改写站点绝对地址；方案二把输入限制在本地或可信网络对端，风险明显更低。

### 4. 推荐方案与实现草图

推荐方案二，并保留 `NOBELOG_TRUST_FORWARDED=1` 作为全量信任开关，供反代不在私网或无法识别对端时使用。逻辑示意：

```rust
// rust/src/serve.rs
fn request_base_url(
    headers: &HeaderMap,
    fallback: &str,
    peer: Option<std::net::IpAddr>,
) -> String {
    if let Some(url) = crate::site::env_url() {
        return url; // SITE_URL 仍然最高优先
    }
    let trust_all = trust_forwarded_headers();          // 现有总开关，信任 Host 与 Proto
    let trust_proto = trust_all || peer_is_loopback_or_private(peer);
    base_url_from_headers(headers, fallback, trust_all, trust_proto)
}

fn base_url_from_headers(
    headers: &HeaderMap,
    fallback: &str,
    trust_host: bool,   // 只由总开关决定
    trust_proto: bool,  // 总开关或 peer 规则
) -> String
```

行为矩阵：

| 对端 | 显式总开关 | Host 来源 | scheme 来源 | 结果 |
|---|---|---|---|---|
| 任意 | 开 | `X-Forwarded-Host` 优先，其次 Host | `X-Forwarded-Proto` | 与 v0.0.24 现有信任模式一致 |
| 环回或私网 | 关 | Host | `X-Forwarded-Proto` | 域名正确，scheme 可还原 https |
| 环回或私网 | 关 | Host | 无 XFP 时 http | 与现状一致 |
| 公网 | 关 | Host | 固定 http | 与现状一致 |
| 任意 | 关 | 无有效 Host | 固定 http | 回退 `SITE.url` |

peer 判定建议范围：IPv4 环回 127.0.0.0/8、IPv6 环回 `::1`、RFC1918 私网、IPv6 ULA `fc00::/7`、链路本地 `169.254.0.0/16` 与 `fe80::/10`。`None` 一律不信任。实现时注意标准库 `Ipv4Addr::is_private` 只覆盖 RFC1918，链路本地与 IPv6 ULA 需要显式判断，具体 API 以当前工具链编译结果为准。

风险与取舍：

- 环回或私网内的其他客户端仍可伪造 `X-Forwarded-Proto`，影响仅限 scheme；`X-Forwarded-Host` 依旧不被采信，域名劫持面没有扩大。建议在 spec 中写明这一取舍。
- 反代在公网网段或跨网段时规则不生效，仍需 `SITE_URL` 或总开关，文档要写清楚。
- 反代必须透传 `Host`，否则域名仍会退回上游地址；spec 的 nginx 示例片段已有 `proxy_set_header Host $host`，位置 `build-serve-deploy.md:126`。

## 五、测试与文档清单

### 1. 现有相关测试

全仓 92 个单元测试，与本次直接相关的是：

| 测试 | 位置 | 覆盖内容 |
|---|---|---|
| `forwarded_headers_used_only_when_trusted` | `rust/src/serve.rs:1722` | 信任模式下 XFP https、XFH 优先、逗号列表、默认模式只用 Host、IPv6 |
| `default_mode_ignores_forged_forwarded_host` | `rust/src/serve.rs:1763` | 默认忽略伪造 XFH 与 XFP；无 Host 时回退；两种模式对比 |
| `base_url_rejects_injected_hosts` | `rust/src/serve.rs:1812` | 7 类恶意 Host 与 XFH 回退，控制字符白名单 |
| `base_url_ignores_unknown_forwarded_proto` | `rust/src/serve.rs:1873` | XFP 只认 http 与 https，大小写与逗号处理 |
| `trust_switch_accepts_only_explicit_truthy_values` | `rust/src/serve.rs:1898` | 总开关取值解析 |
| `feed_escapes_root_url`、`sitemap_escapes_root_url`、`html_context_escapes_base_url` | `rust/src/templates.rs:1640`、`1649`、`1656` | base_url 进入 XML 与 HTML 前的转义 |
| `robots_txt_lists_sitemap_for_env_url_and_default_url` 等 | `rust/src/build.rs:419`、`441`、`464` | 静态构建 base_url 与 robots 行为；样例域名硬编码 example.com |

### 2. 测试缺口

1. `env_url()` 与环境变量优先级没有单元测试；测试明确不依赖环境变量。
2. `request_base_url` 的组合逻辑没有测试，只测了内部的 `base_url_from_headers` 与开关解析。
3. `trust_forwarded_headers()` 读取环境变量这一层没有测试。
4. 没有任何 peer 地址、`ConnectInfo` 相关测试。
5. 没有动态 handler 到 feed、sitemap 的端到端测试；当前只有人工 curl。
6. `build.rs` 的 robots 测试把样例域名写成 example.com，域名下线后测试仍会通过但语义过时。

### 3. 建议新增测试点

1. `peer_is_loopback_or_private` 矩阵：`127.0.0.1`、`::1`、`10.x`、`172.16.x`、`192.168.x`、`169.254.x`、ULA、链路本地为真；`8.8.8.8`、公网 IPv6、`None` 为假。
2. 私网对端加 `X-Forwarded-Proto: https` 加 `Host: good.example` 得 `https://good.example`；同时带 `X-Forwarded-Host: evil.example` 仍只取 good。
3. 公网对端带同样伪造头，结果仍为 `http://good.example`。
4. 环回对端加 XFP 时，`SITE_URL` 仍最高优先，纯函数注入验证，避免改环境变量。
5. 总开关开启时，公网对端恢复 XFH 与 XFP 全量信任，保持 v0.0.24 兼容。
6. 默认模式、信任模式、peer 模式的非法 Host 与非法 XFP 回退矩阵。
7. 可选：`build.rs` 的样例域名改为 `https://example.com`，并补一条 `SITE_URL` 带尾斜杠的 trim 断言。

### 4. 文档漂移清单

| 文件与行 | 当前原文要点 | 问题 | 建议表述 |
|---|---|---|---|
| `README.md:50-51` | 第 51 行写「动态模式会按请求的 Host / X-Forwarded-Proto 自动推断」 | v0.0.24 起默认不信任 XFP，表述过时 | 改为按 Host 推断域名；scheme 默认 http，仅本机或私网对端、或设置 `NOBELOG_TRUST_FORWARDED` 时才采信 XFP |
| `.trellis/spec/frontend/routes-and-urls.md:52-55` | 第 53 至 54 行写「按 Host / X-Forwarded-Proto 拼出 scheme://host」 | 未说明默认只信 Host、scheme 固定 http、以及信任开关 | 重写优先级链：SITE_URL；动态推断的 Host；scheme 由 peer 规则与总开关决定；最后 SITE.url |
| `.trellis/spec/frontend/build-serve-deploy.md:69-75` | 「默认只信 Host、scheme 固定 http，并忽略 X-Forwarded-*」「TLS 反代后没设 SITE_URL 会退化成 http」 | 本次改动后默认行为变化，需要重写 | 分三种场景写：SITE_URL 推荐；环回或私网反代自动还原 https；公网反代仍需总开关 |
| 同上 `:93` | 反代拓扑说明「必须设 SITE_URL 或 NOBELOG_TRUST_FORWARDED=1；默认只信 Host、scheme 固定 http」 | 同上 | 补充 peer 规则与「Host 必须透传」 |
| 同上 `:175` | 「冻结基准当前基线为 https://example.com」 | 域名已下线 | 改为新域名，或改为中性基准域名并说明 旧域名已下线 |
| 同上 `:193` | 打包命令只写 `VERSION=<x.y.z> tools/package.sh` | 没写 SITE_URL，历史上靠操作者记忆 | 改为 `SITE_URL=https://<正式域名> VERSION=<x.y.z> tools/package.sh`，并写明禁止使用已下线域名 |
| 同上 `:80-83` | 发布包说明未提 SITE_URL 注入 | 读者不知道包内绝对地址来自打包时环境变量 | 补一句「打包时以 SITE_URL 重建 public，包内绝对地址即来自该值」 |
| `.trellis/spec/frontend/rendering-templates.md:12`、`71` | base_url 注释与 RSS/sitemap 说明 | 与本次改动基本兼容，未提 peer 规则 | 可选补充一句指向 routes-and-urls |

另外建议在 `build-serve-deploy.md` 增加一条硬规则：发布包禁止使用已下线域名或本机地址作为 `SITE_URL`，打包前必须确认域名可访问。

## 六、最小可交付改动集

### 1. 代码

| 文件 | 改动 | 预期行为 |
|---|---|---|
| `rust/src/serve.rs` | 新增 peer 判定函数；`base_url_from_headers` 拆成 host 与 proto 两个信任参数；`request_base_url` 接收 peer；`handle_dynamic` 读取 `ConnectInfo`；`render_dynamic` 传递 peer；`run()` 改用 connect-info 服务 | 环回或私网对端可还原 https；XFH 仍默认不采信；SITE_URL 仍最高优先；公网伪造无效 |
| `rust/src/site.rs` | 第 47-55 行告警文案更新 | 告警说明默认规则、peer 规则与两个开关的关系，不再让部署者误解为完全自动 |

### 2. 测试

| 文件 | 改动 | 预期行为 |
|---|---|---|
| `rust/src/serve.rs` | 更新现有 5 组测试的函数签名调用；新增 peer 矩阵、mixed trust、SITE_URL 优先级、总开关兼容、注入回退等约 6 组 | 新规则与旧规则都被覆盖，XFH 仍不可被伪造 |
| `rust/src/build.rs` | 第 422 至 458 行的样例域名改为中性域名，补 trim 断言 | 测试不再依赖已下线域名 |

### 3. 文档

| 文件 | 改动 |
|---|---|
| `README.md` | 第 50 至 51 行改为本次改后的真实行为 |
| `.trellis/spec/frontend/routes-and-urls.md` | 第 46 至 58 节重写优先级与 peer 规则 |
| `.trellis/spec/frontend/build-serve-deploy.md` | 第 69 至 75、93、175、193 行更新；补充发布包域名硬规则 |
| 可选 `rendering-templates.md` | 补一句指向新优先级说明 |

### 4. 发布配置

| 项 | 改动 | 预期行为 |
|---|---|---|
| `tools/package.sh` | 构建前强制 `SITE_URL`：非空、以 `https://` 开头、拒绝本机地址与已下线域名；打印实际值；校验 `VERSION` 与 `rust/Cargo.toml` 版本一致；可选校验工作区干净 | 错误配置在打包前失败，不再产出域名错误的包 |
| 版本两源 | `rust/Cargo.toml:3` 与 `rust/Cargo.lock:765-766` 改为 0.0.30 | 二进制 `--version` 为 0.0.30 |
| `public/` | 新域名确定后执行 `SITE_URL=https://<新域名> cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out public` | 冻结基准与包内产物使用新域名 |
| `rust/dist/` | 用同一 `SITE_URL` 重建，或清理后重建 | 回归 diff 的对照产物不再含下线域名 |
| `dist/` | 新版本包照常产出；旧 v0.0.29 包保留 | 历史包与 SHA256SUMS 的既有约定不变 |

### 5. 版本号建议

建议 **0.0.30**。理由：

1. 相对 v0.0.29 有默认行为变化：动态模式对私网或环回对端新增采信 XFP，属于可观察的行为变更，需要发版并在 Release notes 说明。
2. 同时包含文档修正、测试补齐与发布脚本加固，不是单纯补丁。
3. 项目历史节奏是每个发版递增 0.0.x，0.0.30 是下一个自然版本，tag 与 Cargo 版本可直接对齐。

### 6. 本次调研不做的事

1. 不写代码、不改 spec 与配置、不 bump 版本。
2. 不 commit、不 tag、不 push、不创建 GitHub Release、不重启服务。
3. 不改 systemd 单元，不向本机服务注入 `SITE_URL`。

## 七、发版风险

### 1. 旧域名下线后的发布包域名处理建议

1. **先定新域名，再打包**。`SITE_URL` 是唯一可信来源；新域名未定时不要产出正式包，否则包内静态 public 的 canonical 只能是 `http://127.0.0.1:10086`，或继续沿用已下线的 example.com。
2. **package.sh 增加 fail-fast 校验**：`SITE_URL` 为空、不是 https、是 example.com 或 127.0.0.1 时直接报错退出，避免误打包。
3. **重建冻结基准**：用新域名重建 `public/`，然后连续构建两次确认无差异，再作为后续 diff 基准；同时重建 `rust/dist/`。
4. **历史资产不动**：`dist/nobelog-0.0.29` 包与 GitHub v0.0.29 Release 资产保留原样，作为历史版本；新的 0.0.30 包用新域名重新生成。
5. **文档同步**：`build-serve-deploy.md:175` 的基线域名、`:193` 的打包命令、`:69-75` 的部署说明一起更新；历史归档任务不改。

### 2. 历史 Release 与旧包对比风险

- 新包与 v0.0.29 包的差异会包含：域名替换导致的所有 public HTML 变化、代码行为变化、版本号变化。做包对比时不要把域名差异当成回归。
- GitHub v0.0.29 与更早 Release 的资产保持不变；如果需要复现旧产物，仍可下载历史包，但包内域名已无意义。

### 3. 发布前检查清单

1. `SITE_URL` 已确定为正式域名且可访问。
2. `cargo fmt --check`、`cargo clippy --all-targets`、`cargo test` 全绿。
3. `SITE_URL=https://<新域名> ... build --self-assets --out public` 生成基准，连续两次构建逐字节一致。
4. `nobelog diff public rust/dist` 使用同一个 SITE_URL，差异只剩预期项。
5. 版本两源都是 0.0.30，tag 与版本一致，包内 `VERSION` 与二进制 `--version` 一致。
6. 包内 `public/index.html` 的 canonical 是新域名，包内 `robots.txt` 的 Sitemap 是新域名。
7. `(cd dist && sha256sum -c SHA256SUMS)` 通过。

### 4. 上线与本机服务

- 新二进制需要重新 `cargo build --release --locked` 并重启 `nobelog-local` 才能生效，重启需要用户当轮授权；当前服务环境没有 `SITE_URL`。
- 动态模式忽略包内 public 的页面，本机服务能否显示 https canonical 取决于请求头的 peer 规则与是否设置 `SITE_URL`；如果希望本机也显示正式域名，需要在 systemd 单元加 `Environment=SITE_URL=https://<新域名>`，这属于配置变更，需单独授权。
- 上线冒烟建议覆盖：9 条路由与 404、canonical、feed、sitemap、XFH 伪造头、私网 peer 加 XFP、CSS 哈希。

### 5. 推送与发布授权

- 当前本地相对 `origin/main` 领先 8 个提交，发版后需要 `git push origin main` 与 `git push origin v0.0.30`；本次调研不执行。
- GitHub Release 是项目既定默认动作，但创建 release 与上传资产属于外部写操作，需用户授权后由实现方执行。
- 本机重启、systemd 配置变更同样需要授权。

### 6. 其它风险

1. `ConnectInfo` 与 `tap_io` 的兼容性需要在实现阶段编译验证；若不可用，需要调整监听器封装方式，属实现风险但不阻塞方案本身。
2. 私网 peer 可伪造 scheme 的取舍需要写进 spec，避免后续安全审计再次提出。
3. `package.sh` 写死 `/root/.cargo/bin/cargo`，在其它机器或用户下会失败；本次可不改，但发版前确认执行环境。
4. 新默认规则上线后，若有部署方依赖「默认一定 http」的旧行为，可能观察到 canonical 变化，Release notes 需要说明。

## 八、证据输出摘录

### 1. 硬编码计数

```bash
$ git grep -c 'example\.com' | awk -F: '{s+=$2; n++} END{print "files:", n, "occurrences:", s}'
files: 19 occurrences: 48
$ for d in public rust/dist; do echo -n "$d: "; grep -rc 'example\.com' "$d" | awk -F: '{s+=$2} END{print s}'; done
public: 82
rust/dist: 82
$ git grep -c 'SITE_URL' | awk -F: '{s+=$2; n++} END{print "files:", n, "occurrences:", s}'
files: 39 occurrences: 137
$ grep -c 'SITE_URL' rust/src/*.rs | grep -v ':0'
rust/src/build.rs:6
rust/src/serve.rs:3
rust/src/site.rs:4
rust/src/templates.rs:1
$ grep -c 'NOBELOG_TRUST_FORWARDED' rust/src/*.rs | grep -v ':0'
rust/src/serve.rs:4
rust/src/site.rs:1
```

### 2. 冻结基准与包内域名

```bash
$ grep -o 'rel="canonical" href="[^"]*"' public/index.html
rel="canonical" href="https://example.com/"
$ grep -o 'rel="canonical" href="[^"]*"' rust/dist/index.html
rel="canonical" href="https://example.com/"
$ tar -xzOf dist/nobelog-0.0.29-linux-x86_64.tar.gz \
    nobelog-0.0.29-linux-x86_64/public/index.html | grep -o 'rel="canonical" href="[^"]*"'
rel="canonical" href="https://example.com/"
```

### 3. 版本与远程状态

```bash
$ grep -n '^version' rust/Cargo.toml
3:version = "0.0.29"
$ grep -n -A2 'name = "nobelog"' rust/Cargo.lock
765:name = "nobelog"
766-version = "0.0.29"
$ git describe --tags --always
v0.0.29-8-g99b138f
$ git status --short --branch
## main...origin/main [ahead 8]
?? .trellis/tasks/10-30-domain-autodetect-release/
$ git ls-remote origin refs/heads/main
28f80ea8887c51ec7b13f52bc37aaa955c5fb972	refs/heads/main
$ gh release list -R zhemed/nobelog -L 1 --json tagName,isLatest
[{"isLatest":true,"tagName":"v0.0.29"}]
```

### 4. 现有实现与依赖

```bash
$ grep -n -A3 '^name = "axum"' rust/Cargo.lock | head -4
125:name = "axum"
126-version = "0.8.9"
$ grep -rn 'ConnectInfo\|SocketAddr\|IpAddr' rust/src || echo none
none
$ grep -n 'axum::serve' rust/src/serve.rs
306:    axum::serve(listener, app)
$ grep -n 'request_base_url\|base_url_from_headers\|trust_forwarded_headers' rust/src/serve.rs
315:fn request_base_url(headers: &HeaderMap, fallback: &str) -> String
319:    base_url_from_headers(headers, fallback, trust_forwarded_headers())
323:fn trust_forwarded_headers() -> bool
346:fn base_url_from_headers(headers: &HeaderMap, fallback: &str, trust_forwarded: bool) -> String
663:    let base = request_base_url(request_headers, &runtime.site.root_url);
```

### 5. 本机服务现状

```bash
$ systemctl show nobelog-local -p ActiveEnterTimestamp,ExecMainPID,Environment
ExecMainPID=797659
Environment=
ActiveEnterTimestamp=Wed 2026-10-21 15:30:36 UTC
$ ./rust/target/release/nobelog --version
nobelog 0.0.29
```

## 九、未覆盖项与不确定

1. **新域名未知**：用户只说 旧域名已下线，没有给出新域名；发版前必须补齐，否则无法正确重建基准与打包。
2. **ConnectInfo 与 tap_io 兼容性未编译验证**：本次只读，不运行 cargo；axum 0.8.9 的标准入口需要实现阶段用 `cargo build` 与测试确认。若 `TapIo` 包装后无法提供连接信息，需要调整监听器封装，方案本身仍成立。
3. **未运行任何构建或测试**：测试数量、行号与行为结论来自现有源码与上一轮只读核查记录；实现后需以实际 `cargo test` 结果为准。
4. **未访问公网域名**：新域名是否可用、远端部署形态与 TLS 反代配置均未验证。
5. **未验证的历史发布包**：旧包只做了域名存在性检查，未逐字节重算 sha256；历史资产按保留处理，不影响新版本。
6. 本报告本身也在 `.trellis/tasks/10-30-domain-autodetect-release/research/` 中新增了 example.com 字样，属任务记录，不算产品硬编码。
