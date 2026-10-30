# Research: 2.3.4 域名自动识别是否完善

- Query: 用户追问「2.3.4 是不是已经完善了，我记得已经好了呀，会自动识别客户生产机的域名」
- Scope: mixed，内部代码与文档溯源，加上本地 10086 服务的只读行为验证
- Date: 2026-10-30
- 核查时刻: 2026-10-30T11:20Z 至 11:30Z，命令在仓库根 `/root/博客/nobelog` 执行
- 约束: 全程只读；未改代码、配置、spec 与任务文件；未 commit/push/fetch；未重启服务；未新起实例；未访问公网博客域名

## 一、结论摘要

1. **「2.3.4」出处没有找到。** 在当前工作树、139 个提交的完整 git 历史、`.trellis` 全部文档、5 个工作区 `.backups` 归档、GitHub issues 与 PR、wiki 中都没有编号型「2.3.4」。唯一字面命中是历史部署脚本里的示例 IP `root@1.2.3.4` 和旧 pnpm 锁文件里的 `punycode.js@2.3.1`，都与需求无关。判断：2.3.4 不是本仓库的编号，最可能来自用户侧的外部需求或验收文档，本仓库查不到原文。
2. **若按能力语义理解「自动识别客户生产机的域名」，结论是部分完善。** 域名部分已经完成并加固：动态模式下未设 `SITE_URL` 时按请求 `Host` 推断，canonical、og:url、og:image、页脚、分享链接、RSS、sitemap 全部跟随 Host；Host 走字符白名单；默认忽略伪造的 `X-Forwarded-*`。协议 scheme 部分只在特定配置下完善：默认固定 `http`，TLS 反代场景必须设 `SITE_URL` 或显式设 `NOBELOG_TRUST_FORWARDED=1`，否则绝对链接变成 `http://域名`。
3. **客户生产机典型场景判断。** 动态模式且反代透传 `Host` 时，域名能被自动识别；但 TLS 反代后未设 `SITE_URL`、未开 `NOBELOG_TRUST_FORWARDED` 时 scheme 错误，canonical 与 RSS/sitemap 会退化为 `http://<域名>`。若反代没有透传 `Host`，nginx 默认会把上游地址作为 Host，域名识别也会失效并退回 `http://127.0.0.1:10086`。
4. **静态构建与静态部署不参与自动识别。** 站点绝对地址在构建期固化，必须用 `SITE_URL`；当前发布包内 `public/` 是用 `https://example.com` 构建的，客户机若直接部署这份静态产物，canonical 会指向 `example.com`。
5. **文档漂移解释了「我记得已经好了」的印象。** `README.md` 第 51 行与 `.trellis/spec/frontend/routes-and-urls.md` 第 53 至 54 行仍写着按 `Host` / `X-Forwarded-Proto` 自动推断，没有反映 v0.0.24 起默认不信任 XFP 的安全改动；v0.0.20 的 Release 说明原文正是这套自动推断表述。
6. **测试覆盖与缺口。** 全仓 92 个单元测试，其中 `serve.rs` 有 5 组测试直接覆盖 Host、转发头与信任开关，注入用例覆盖 7 类恶意 Host；缺口是 `SITE_URL` 优先级与 `request_base_url` 组合没有单测，真实 TLS 反代没有端到端测试。

**明确结论：2.3.4 所指能力若是「自动识别域名」，已完成；若把「https 协议也自动识别」算进去，则只算部分完善，客户生产机在未配置时 scheme 会错。**

## 二、「2.3.4」出处定位

### 1. 检索范围与结果

| 检索对象 | 命令 | 结果 |
|---|---|---|
| 工作树全文件，含被 git 忽略的 public 与 dist | `grep -rIn '2\.3\.4' --exclude-dir=.git --exclude-dir=target .` | 只命中本次任务自己的 prd 与 task.json、directive.md |
| git 全历史提交差异 | `git log --all --oneline -S '2.3.4'` 与 `-G '2\.3\.4'` | 只命中 2b558d5 与 7ffef09，内容为 deploy 脚本示例 IP |
| 提交消息 | `git log --all --oneline --grep='2\.3\.4'` | 无命中 |
| 全历史 markdown 编号标题 | `git grep -n -E '^#+ +[0-9]+(\.[0-9]+)+' $(git rev-list --all) -- '*.md'` | 只有 Trellis workflow 的 1.0 至 3.5 编号，无 2.3.4 |
| 归档任务、spec、workflow、journal、README、content、rust 源码 | 同上 grep 与逐文件核对 | 无命中 |
| 旧 Node/TS 版本备份 5 个 tar 包 | `tar -xzOf` 流式检索全部文本成员 | 无编号命中；只有 `pnpm-lock.yaml` 的 `punycode.js@2.3.1` |
| GitHub issues 与 PR | `gh issue list --state all` 与 `gh pr list --state all` | 均返回空数组 |
| GitHub wiki | `git ls-remote https://github.com/zhemed/nobelog.wiki.git` | Repository not found，即无 wiki 内容 |
| 工作区根 `/root/博客` 与 `.backups` | `grep -rIn '2\.3\.4' /root/博客 --exclude-dir=.git` | 只命中 nobelog 任务目录自身，无其它文档 |

字面命中的原始摘录，确认均非需求编号：

```text
2b558d5:deploy/deploy.sh:10:#   DEPLOY_HOST=root@1.2.3.4 ./deploy/deploy.sh
2b558d5:deploy/deploy.sh:14:HOST="${DEPLOY_HOST:?请设置 DEPLOY_HOST，例如 DEPLOY_HOST=root@1.2.3.4}"
旧 Node 备份: pnpm-lock.yaml: punycode.js@2.3.1
```

### 2. 同义需求检索结果

| 关键词 | 结果 |
|---|---|
| 自动识别 | 只有代码高亮语言的「自动识别」注释，位置在 `rust/src/highlight.rs` 第 309 与 388 行，与域名无关 |
| 生产机 | 仓库与全部 git 历史均无命中 |
| 客户 | 只在提交消息中出现过，没有形成需求条目 |
| 域名、SITE_URL、X-Forwarded、TRUST_FORWARDED、canonical | 命中集中在 spec、journal、发布任务与 `rust/src`，均能对应到「站点绝对地址统一」「XFH 默认安全」这两条已归档任务，没有编号式条目 |

### 3. 最可能解释与候选

候选 A，最可能：**外部需求清单或验收文档的条目编号**。依据是上面全部检索均无命中，而且本仓库没有 2.3.4 这种三级编号体系：Trellis 任务用 R1/R2 或 Requirements 1. 2.，审计用 P0 至 P3，workflow 用 1.0 至 3.5。因此 2.3.4 的原文无法在本仓库复核，需要用户提供该文档或原文。

候选 B：**v0.0.20 发布说明中的能力描述**，这最接近用户记忆。原文摘录：

```json
{
  "tagName": "v0.0.20",
  "name": "v0.0.20：站点地址统一",
  "body": "canonical/og/页脚/分享/RSS/sitemap 统一使用 SITE_URL；未设置时动态模式按请求 Host + X-Forwarded-Proto 自动推断，静态构建回退默认地址。"
}
```

候选 C：**早期免编译部署包的安装期域名配置**。历史提交 a01435c 曾加入 `deploy/install.sh`，其中第 57 行按 `DOMAIN` 自动写入 systemd 环境：

```text
57:  [ -n "$DOMAIN" ] && echo "Environment=SITE_URL=https://$DOMAIN"
```

这是安装期配置，不是运行期自动识别；该部署工具已在提交 7ffef09 中整体删除。

候选 D：**旧 Node 版本遗留的域名待定项**。旧 Node 版本的 `docs/decisions.md` 第 10 节写过「站点域名待定，上线前替换，或用 `SITE_URL` 环境变量覆盖」；旧实现也没有请求级推断，`server/routes.ts:82` 的根地址在启动时固定，取自环境变量 `SITE_URL` 或 `site.url`：

```javascript
// 旧 Node 版本 server/routes.ts:82，来自 .backups 归档的只读流式读取
const rootUrl = (process.env.SITE_URL ?? site.url).replace(/\/$/, "");
```

这说明运行期自动识别是 Rust 重写后 v0.0.20 才加入的能力。

结论：编号本身无法证实，本报告以「自动识别客户生产机域名」这一语义对应实现来核查，并在后文给出分场景判断。

## 三、实现链

### 1. 关键代码位置

| 环节 | 位置 | 行为 |
|---|---|---|
| 读取 `SITE_URL` | `rust/src/site.rs:36-42` | `env_url()` 读环境变量，`trim()` 后去掉尾部 `/`，空值视为未设置 |
| 启动告警 | `rust/src/site.rs:44-55`，调用点 `rust/src/serve.rs:258-259` | 未设 `SITE_URL` 只告警一次，不拒绝启动 |
| 动态地址优先级 | `rust/src/serve.rs:313-320` | `request_base_url`：`SITE_URL` 存在就直接返回，否则走请求头推断 |
| 信任开关解析 | `rust/src/serve.rs:322-335` | 只有 `1`、`true`、`yes`、`on`，不区分大小写，才信任转发头 |
| 请求头推断 | `rust/src/serve.rs:337-373` | 默认只读 `Host` 且 scheme 固定 `http`；信任模式下 `X-Forwarded-Host` 优先、`X-Forwarded-Proto` 只认精确的 http 与 https |
| Host 白名单 | `rust/src/serve.rs:375-388` | 只允许字母数字与 `.`、`-`、`:`、`[`、`]`；逗号列表取第一段；非法整条丢弃并回退 |
| 动态 handler | `rust/src/serve.rs:578-653` | 克隆请求头，资产与清单单独处理，页面交给 `render_dynamic` |
| 动态渲染注入 | `rust/src/serve.rs:656-663` | `let base = request_base_url(request_headers, &runtime.site.root_url);` 再交给 `Site::page` |
| 静态运行时回退 | `rust/src/serve.rs:563` 与 `rust/src/routes.rs:56` | 不使用请求头，只用 `env_url()` 或 `SITE.url`；`Site::new` 时固化 `root_url` |
| 静态构建 | `rust/src/build.rs:66` | `base_url = env_url().unwrap_or(SITE.url)`，构建期固化，没有请求上下文 |
| canonical 与 OG | `rust/src/templates.rs:775-802` | `{url}{path}` 作为 canonical 与 og:url，og:image 为 `{url}/images/site-icon-512.png`，`url` 先做 HTML 转义 |
| 页脚 | `rust/src/templates.rs:356-358` | 页脚站点链接用 `base_url` 并转义 |
| 分享链接 | `rust/src/templates.rs:1113-1115` | 整条 URL 过 `url::encode` |
| RSS | `rust/src/templates.rs:1521-1570` | channel link、atom self link、item link 全部用 `root_url`，XML 转义 |
| sitemap | `rust/src/templates.rs:1573-1590` | 每个 `loc` 为 `root_url + path`，XML 转义 |

优先级链可以概括为：`SITE_URL` 最高；其次是动态模式的请求推断，默认只采信 `Host` 且 scheme 固定 `http`，只有显式信任开关才采信 `X-Forwarded-Host` 与 `X-Forwarded-Proto`；最后回退 `SITE.url` 的内置值 `http://127.0.0.1:10086`。静态构建与静态模式不参与请求推断。

### 2. 版本演化

| 版本 | 提交 | 日期 | 内容 |
|---|---|---|---|
| v0.0.20 | `5b6ed33` | 2026-10-19 | 站点绝对地址统一支持 `SITE_URL`，动态模式按 Host 兜底，首次加入 `request_base_url` |
| v0.0.22 | `9ad2ee3` | 2026-10-19 | P1 修复：动态模式 Host 注入，加 Host 字符白名单与 URL 转义 |
| v0.0.24 | `1f37229` | 2026-10-20 | 安全修复：默认不信任 `X-Forwarded-*`，`SITE_URL` 优先，新增 `NOBELOG_TRUST_FORWARDED` 开关 |
| v0.0.24 之后 | 无 | — | `git log v0.0.24..HEAD -- rust/src/serve.rs rust/src/site.rs rust/src/build.rs rust/src/routes.rs` 为空，URL 核心逻辑未再改动 |

`rust/src/templates.rs` 在 v0.0.24 之后有其他提交，但都是 favicon 图标相关改动，canonical、og、feed、sitemap 的 URL 语义没有变化。当前运行版本为 v0.0.29。

## 四、行为验证矩阵

服务现状：`nobelog-local.service` 自 2026-10-21T15:30:36Z 运行，PID 797659，`--dynamic` 模式，监听 0.0.0.0:10086。进程环境与 systemd 单元都没有 `SITE_URL` 与 `NOBELOG_TRUST_FORWARDED`：

```text
$ systemctl show nobelog-local -p Environment
Environment=
$ tr '\0' '\n' < /proc/797659/environ | grep -Ei 'SITE_URL|TRUST'
# 无输出
$ ./rust/target/release/nobelog --version
nobelog 0.0.29
```

### 1. 首页 absolute 地址矩阵

| 编号 | 请求头 | canonical | 判定 |
|---|---|---|---|
| A | 默认 `Host: 127.0.0.1:10086` | `http://127.0.0.1:10086/` | 回退内置地址 |
| B | `Host: blog.example.com` | `http://blog.example.com/` | 域名自动识别生效，scheme 为 http |
| C | `Host: blog.example.com:8443` | `http://blog.example.com:8443/` | 端口保留 |
| D | 默认 Host 加伪造 `X-Forwarded-Host: evil.example.com` 与 `X-Forwarded-Proto: https` | `http://127.0.0.1:10086/` | 默认忽略转发头，注入失败 |
| E | 只带 `X-Forwarded-Host: evil.example.com` | `http://127.0.0.1:10086/` | 同上 |
| F | `Host: blog.example.com` 加 `X-Forwarded-Proto: https` | `http://blog.example.com/` | 默认不采信 XFP，scheme 仍 http |
| G | 非法 Host，值含引号与尖括号 | `http://127.0.0.1:10086/` | 白名单拒绝并回退 |
| H | `Host: [::1]:10086` | `http://[::1]:10086/` | IPv6 方括号被接受 |
| I | `Host: BLOG.EXAMPLE.COM` | `http://BLOG.EXAMPLE.COM/` | 原样保留大小写，未归一化 |
| J | `Host: a.example.com,b.example.com` | `http://a.example.com/` | 逗号列表取第一段 |
| K | 不带 Host 头 | `http://127.0.0.1:10086/` | 回退内置地址 |
| P | `Host: localhost:10086` | `http://localhost:10086/` | 生效 |
| Q | `Host: blog.example.com.` | `http://blog.example.com./` | 尾点被接受 |
| R | `Host: good.example` 加 `X-Forwarded-Host: evil.example` | `http://good.example/` | 默认只信 Host |
| S | `Host: good.example` 加 `X-Forwarded-Proto: https` 与 `X-Forwarded-Host: evil.example` | `http://good.example/` | scheme 与域名都不受伪造头影响 |
| T | `Host: bad host.example` | `http://127.0.0.1:10086/` | 空格非法，回退 |
| U | `Host: good.com:80@evil.com` | `http://127.0.0.1:10086/` | userinfo 注入被拒绝 |

关键原始输出：

```text
A default            canonical: rel="canonical" href="http://127.0.0.1:10086/"
B Host blog.example  canonical: rel="canonical" href="http://blog.example.com/"
D XFH+XFP 伪造        canonical: rel="canonical" href="http://127.0.0.1:10086/"
F Host + XFP https   canonical: rel="canonical" href="http://blog.example.com/"
G 非法 Host          canonical: rel="canonical" href="http://127.0.0.1:10086/"
```

### 2. 其它绝对地址输出

| 输出点 | 默认 Host | 自定义 `Host: blog.example.com` |
|---|---|---|
| og:url | `http://127.0.0.1:10086/` | `http://blog.example.com/` |
| og:image | `http://127.0.0.1:10086/images/site-icon-512.png` | `http://blog.example.com/images/site-icon-512.png` |
| 404 页 canonical | `http://127.0.0.1:10086/__nope__` | `http://blog.example.com/__nope__` |
| RSS channel link | `http://127.0.0.1:10086/` | `http://blog.example.com/` |
| RSS item link | `http://127.0.0.1:10086/posts/typeset-test/` | `http://blog.example.com/posts/typeset-test/` |
| sitemap 首条 loc | `http://127.0.0.1:10086/` | `http://blog.example.com/` |
| 分享链接中的 url 参数 | 指向 127.0.0.1 | `http://blog.example.com/posts/typeset-test/` 百分号编码 |

原始输出摘录：

```text
$ curl -s -H 'Host: blog.example.com' http://127.0.0.1:10086/feed.xml | grep -o '<link>http[^<]*</link>'
<link>http://blog.example.com/</link>
<link>http://blog.example.com/posts/typeset-test/</link>

$ curl -s -H 'Host: blog.example.com' http://127.0.0.1:10086/sitemap.xml | grep -o '<loc>[^<]*</loc>'
<loc>http://blog.example.com/</loc>
<loc>http://blog.example.com/archives/</loc>
...
```

### 3. 信任模式说明

当前进程没有 `NOBELOG_TRUST_FORWARDED`，也没有 `SITE_URL`，且任务禁止重启或新起实例，因此**信任模式无法做端到端实测**。该模式由单元测试证明，测试在第 1721 至 1760 行覆盖：信任模式下 `Host: example.com:8080` 加 `X-Forwarded-Proto: https` 得到 `https://example.com:8080`；`X-Forwarded-Host` 优先于 Host；未知 proto 回退 http。

## 五、测试覆盖

全仓共 92 个单元测试，`#[test]` 分布为 `serve.rs` 26、`content.rs` 22、`build.rs` 8、`assets.rs` 7、`highlight.rs` 6、`markdown.rs` 4、`templates.rs` 4、`routes.rs` 3、`highlight_map.rs` 3、`tools/css_prune.rs` 9。

与域名识别直接相关的测试：

| 测试名 | 位置 | 覆盖内容 |
|---|---|---|
| `forwarded_headers_used_only_when_trusted` | `rust/src/serve.rs:1721` | 信任模式下 XFP https 生效、XFH 优先于 Host、逗号列表取第一段、默认模式只用 Host、IPv6 主机 |
| `default_mode_ignores_forged_forwarded_host` | `rust/src/serve.rs:1762` | 伪造 XFH 不改写默认模式结果；同一请求在信任模式下才用 XFH；只有 XFH 无 Host 时回退；完全无头时回退 |
| `base_url_rejects_injected_hosts` | `rust/src/serve.rs:1811` | 7 类恶意 Host 与 XFH 在两种模式下都回退，包括属性闭合、userinfo、路径、反斜杠、空白、分号、括号；控制字符与 tab 不在白名单 |
| `base_url_ignores_unknown_forwarded_proto` | `rust/src/serve.rs:1872` | 信任模式下 XFP 只认精确 http 与 https，`file`、大写 `HTTPS` 不生效，`https, http` 取第一段 |
| `trust_switch_accepts_only_explicit_truthy_values` | `rust/src/serve.rs:1897` | 开关值 `1/true/TRUE/yes/on/ On /True` 为真；空串、`0/false/no/off/2/tru/yes!/file` 为假，未设置为假 |
| `html_context_escapes_base_url` | `rust/src/templates.rs:1656` | base_url 进入 HTML 属性与 og:image 前必须转义；页脚同样转义 |
| `feed_escapes_root_url` 与 `sitemap_escapes_root_url` | `rust/src/templates.rs:1640`、`1649` | root_url 进入 XML 前必须转义 |
| `robots_txt_lists_sitemap_for_env_url_and_default_url` | `rust/src/build.rs:419` | 静态构建的 robots.txt 使用传入的 base_url；默认地址形态正确 |

未覆盖或覆盖不足的场景：

1. `env_url()` 与环境变量优先级没有单元测试；测试全部用纯函数注入参数，不读 `SITE_URL`，注释也明确写「测试不依赖环境变量」。
2. `request_base_url` 的组合逻辑没有测试；测试只覆盖它内部调用的 `base_url_from_headers` 与 `parse_trust_forwarded`，没有覆盖 `SITE_URL` 存在时提前返回的分支。
3. `trust_forwarded_headers()` 读取环境变量这一层没有测试；只测了纯解析函数。
4. 没有集成测试把动态 handler、feed、sitemap 与 Host 组合起来做端到端断言；本次行为矩阵由人工 curl 补充。
5. 没有针对真实 TLS 反代链路的测试。
6. Host 大小写归一化、尾点归一化没有测试，当前行为是原样保留。
7. 静态构建与静态模式「不自动识别」的行为没有测试，只有代码路径。

## 六、缺口与分场景判断

### 1. 客户生产机场景

| 场景 | 域名 | scheme | 综合判定 |
|---|---|---|---|
| 动态模式，客户端直接访问 http | 正确，来自 Host | 正确 http | 完善 |
| 动态模式，TLS 反代透传 Host，未设 `SITE_URL`，未开 `NOBELOG_TRUST_FORWARDED` | 正确 | 错误，固定 http | 部分完善 |
| 动态模式，TLS 反代透传 Host，设置 `NOBELOG_TRUST_FORWARDED=1`，反代设置 `X-Forwarded-Proto` | 正确 | 正确 https | 完善 |
| 动态模式，TLS 反代透传 Host，设置 `SITE_URL=https://域名` | 正确 | 正确 https | 完善，推荐 |
| 动态模式，反代没有透传 Host，Host 变成上游地址 | 错误，退回 127.0.0.1:10086 | 错误 http | 不完善 |
| 静态部署 `public/` 产物，未重新构建 | 构建期固化，包内是 example.com | 构建期固化 | 不完善，需构建时设 SITE_URL |

其中第三条和第四条可以正常工作的前提是 nginx 显式透传原始 Host。当前 spec 的 nginx 示例片段有这一行，位置 `.trellis/spec/frontend/build-serve-deploy.md:126`：

```nginx
proxy_pass http://<app_upstream>;
proxy_http_version 1.1;
proxy_set_header Host $host;
proxy_set_header X-Forwarded-Proto $scheme;
```

如果客户机照抄这个片段，Host 会被正确透传；但 `X-Forwarded-Proto` 只有在开了信任开关后才会被应用。

### 2. 缺口清单

1. **TLS 反代零配置不成立**：默认 `scheme= http`，canonical、og:url、RSS、sitemap、分享链接都会写 `http://域名`，与浏览器实际访问的 https 不一致。严重度中，功能可用但 SEO 与分享卡片信号偏弱；部分客户端会跟随 301 到 https，但 canonical 本身已经错了。
2. **强依赖反代透传 Host**：nginx 未设置 `proxy_set_header Host $host` 时，应用看到上游地址，域名自动识别失效并回退 `127.0.0.1:10086`。严重度高。
3. **静态部署不自动识别**：发布包 `public/` 是用 `example.com` 构建的，客户机静态部署会得到 `example.com` 的 canonical；必须用 `SITE_URL` 重新构建。
4. **文档漂移**：`README.md:51` 与 `.trellis/spec/frontend/routes-and-urls.md:53-54` 仍写按 Host 与 X-Forwarded-Proto 自动推断，与 v0.0.24 起的行为不一致；这会让部署者误判为「已经自动好了」。权威说明在 `.trellis/spec/frontend/build-serve-deploy.md:69-75`。
5. **测试缺口**：`SITE_URL` 优先级、`request_base_url` 组合、真实反代链路没有自动化测试，当前结论依赖人工 curl 与纯函数单测。

### 3. 建议，只列方向不实施

1. 生产部署优先显式设置 `SITE_URL=https://域名`，这是 spec 推荐的最高优先级方案，动态与静态都正确。
2. 如果确实需要在反代后依赖请求推断，显式设置 `NOBELOG_TRUST_FORWARDED=1`，并确保反代设置 `X-Forwarded-Proto` 与 `Host`。
3. 把 `README.md:51` 与 `.trellis/spec/frontend/routes-and-urls.md:53-54` 更新为 v0.0.24 之后的实际行为，避免再次误判。
4. 如果产品目标真是「客户生产机零配置自动识别 https」，当前默认策略不满足；需要单独设计，例如默认信任来自环回或私网反代链路的 `X-Forwarded-Proto`，这涉及安全权衡，应另开任务评估，不在本次只读核查范围内定方案。
5. 若 2.3.4 是外部需求文档条目，建议用户提供该条原文，以便逐条验收，而不是按语义推断。

### 4. 对用户问题的直接回答

- 「2.3.4 是不是已经完善了」：如果 2.3.4 指「自动识别客户生产机域名」，域名部分确实已经完成，v0.0.20 加入、v0.0.22 加固、v0.0.24 起默认安全；但 **https 协议部分没有默认自动完成**，客户生产机在 TLS 反代后未设 `SITE_URL`、未开信任开关时会得到 `http://域名` 的绝对地址。整体应回答部分完善，不是全部完善。
- 「我记得已经好了呀」：这个记忆有据可查，v0.0.20 Release 说明原文确实写着「动态模式按请求 Host + X-Forwarded-Proto 自动推断」；后来 v0.0.24 为了修复 XFH 注入，把「默认信任转发头」改成「显式开关才信任」，能力仍在，但默认行为收紧了，README 没有同步更新。

## 七、证据命令与原始输出摘录

### 1. 出处检索

```bash
$ grep -rIn '2\.3\.4' --exclude-dir=.git --exclude-dir=target --exclude-dir=node_modules .
# 只命中 .trellis/tasks/10-30-check-domain-autodetect/ 自身文件

$ git log --all --oneline -S '2.3.4'
7ffef09 chore: 移除部署工具，仓库只保留博客代码与内容
2b558d5 init: 博客（Rust 静态生成器 + 空内容骨架）

$ git grep -n '2\.3\.4' 2b558d5
2b558d5:deploy/deploy.sh:10:#   DEPLOY_HOST=root@1.2.3.4 ./deploy/deploy.sh
2b558d5:deploy/deploy.sh:14:HOST="${DEPLOY_HOST:?请设置 DEPLOY_HOST，例如 DEPLOY_HOST=root@1.2.3.4}"

$ git rev-list --all --count
139
```

### 2. 优先级与告警代码

```rust
// rust/src/serve.rs:313-320
fn request_base_url(headers: &HeaderMap, fallback: &str) -> String {
    if let Some(url) = crate::site::env_url() {
        return url;
    }
    base_url_from_headers(headers, fallback, trust_forwarded_headers())
}

// rust/src/serve.rs:367-372，默认分支
} else {
    // 默认路径不采信任何转发头，也就没有可信的 TLS 终止信息：scheme 固定 http，
    // 对外域名与协议请用 SITE_URL 显式声明。
    "http"
};
format!("{proto}://{host}")
```

```rust
// rust/src/site.rs:47-55
pub fn warn_if_env_url_missing() {
    if env_url().is_none() {
        eprintln!(
            "警告：未设置 SITE_URL，站点绝对链接回退到内置默认地址 {}；生产部署请设置 SITE_URL=https://你的域名。\
             动态模式默认只按 Host 头推断、不信任 X-Forwarded-*；若前面有可信反向代理，请设 NOBELOG_TRUST_FORWARDED=1。",
            SITE.url
        );
    }
}
```

### 3. README 与 spec 漂移

```text
$ git blame -L 50,51 README.md
02b5dbcb ... 50) 对外部署时设置 `SITE_URL` ...，sitemap、RSS、canonical、页脚与分享链接都会使用它。
5b6ed338 ... 51) 未设置时，动态模式会按请求的 `Host` / `X-Forwarded-Proto` 自动推断；静态构建则回退到默认地址。
# 第 51 行来自 v0.0.20 提交 5b6ed33，v0.0.24 之后未更新

$ git blame -L 52,54 .trellis/spec/frontend/routes-and-urls.md
f4c10993 ... 52) 1. 环境变量 `SITE_URL`：静态构建、动态模式都生效。
f4c10993 ... 53) 2. 动态模式的请求推断：`request_base_url(headers, fallback)`，按 `Host` / `X-Forwarded-Proto` 拼出
f4c10993 ... 54)    `scheme://host`，见 `serve.rs`。
# 同样来自 v0.0.20 时期的 spec 初始化，未反映 v0.0.24
```

### 4. 受控测试与全量测试数

```bash
$ grep -rc '#\[test\]' rust --include='*.rs' | awk -F: '{s+=$2} END{print s}'
92
$ grep -n '    fn forwarded_headers_used_only_when_trusted\|    fn default_mode_ignores_forged_forwarded_host\|    fn base_url_rejects_injected_hosts\|    fn base_url_ignores_unknown_forwarded_proto\|    fn trust_switch_accepts_only_explicit_truthy_values' rust/src/serve.rs
1722:    fn forwarded_headers_used_only_when_trusted()
1763:    fn default_mode_ignores_forged_forwarded_host()
1812:    fn base_url_rejects_injected_hosts()
1873:    fn base_url_ignores_unknown_forwarded_proto()
1898:    fn trust_switch_accepts_only_explicit_truthy_values()
```

### 5. 静态发布包域名固化

```bash
$ tar -xzOf dist/nobelog-0.0.29-linux-x86_64.tar.gz \
    nobelog-0.0.29-linux-x86_64/public/index.html | grep -o 'rel="canonical" href="[^"]*"'
rel="canonical" href="https://example.com/"
```

## 八、未覆盖项与不确定

1. **2.3.4 原文未找到**，编号语义无法证实；报告按「自动识别客户生产机域名」做能力核查。若用户持有原始文档，需提供 2.3.4 原文再逐条验收。
2. **信任模式未做端到端实测**：任务禁止重启与新起实例，当前进程环境没有 `NOBELOG_TRUST_FORWARDED`，该模式结论来自单元测试与代码阅读。
3. **静态构建未实测**：构建会写文件，违反只读约束；静态路径结论来自 `build.rs:66`、`routes.rs:56`、`serve.rs:563` 的代码阅读与既有发布包产物。
4. **未访问公网博客域名**，线上 TLS 反代的实际配置与真实 canonical 未验证。
5. 运行二进制与源码关系沿用上一轮核查结论：当前服务是 v0.0.29 发布包内二进制，v0.0.24 之后 URL 核心文件无改动，因此本次代码阅读结论适用于运行进程。
6. 旧 Node/TS 版本代码来自 `/root/博客/.backups` 的只读 tar 流式读取，未解包到磁盘。
