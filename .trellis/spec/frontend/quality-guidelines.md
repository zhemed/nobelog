# 质量基线

## 提交前必跑

```bash
cd rust
cargo fmt --check
cargo clippy --all-targets
cargo test
```

- `cargo test` 目前覆盖 4 个模块：`serve.rs`（路径解析/穿越/百分号解码）、`highlight.rs`、
  `highlight_map.rs`、`css_prune.rs`。改这几处必须补/改用例。
- 改了渲染输出再加一步：`nobelog build --self-assets --out dist` +
  `./rust/target/release/nobelog diff public dist`，确认差异只有预期部分（历史提交 `ad9cd3f` 就是 fmt/clippy 修复的样板）。
- 只改文档/样式补丁时至少保证 `cargo build` 通过、`diff` 无意外变化。

## 提交信息

历史风格：`<type>: <中文说明>`，type 用 `feat` / `fix` / `style` / `content` / `refactor` / `test` / `chore`。
例：

```
fix: 站点绝对地址统一支持 SITE_URL，动态模式按 Host 兜底
style: 分类行分隔线适配暗色
content: 多命令代码块拆分为独立块，便于单独复制
```

Trellis 仓库的提交必须绑定任务锚点：`[task:<任务目录名>]`（如 `[task:10-19-enable-trellis]`）。
全局 git 钩子会拦截：缺少锚点、缺 `directive.md`、或用"比 directive 更旧的文件"冒充本次改动。
不要 `--no-verify` 绕过。

## 禁止事项

| 禁止 | 原因 | 正确做法 |
| --- | --- | --- |
| 手改 `public/` | 它是逐字节对比的冻结基准，也是部署产物 | 改模板/`patch.css` 后重新构建 |
| 改 `vendor/chirpy/*.css`、`vendor/github-markdown/*.css` | 上游产物，重新抓取即丢 | 写 `vendor/patches/patch.css`，按分区追加 |
| 请求路径里 `unwrap()` / `expect()` | 触发 panic 直接 500，且会 abort（release `panic="abort"`） | 返回 404 或用 `anyhow` + `context` 上抛 |
| 手拼 URL / 手写 percent-encoding | 中文标签、特殊字符会不一致 | `url::encode/decode`、`templates::{post_url,tag_url,category_url}` |
| 写死绝对地址 | `SITE_URL` 部署后链接错误 | 模板用 `ctx.base_url`，路由用 `Site::root_url` |
| 在模板里输出未转义的 frontmatter | XSS/结构破坏 | `escape_html` |
| 用字节长度/切片做 UTF-16 语义的截断 | 与 JS 版不一致、可能切断字符 | 按 `len_utf16()` 累计 |
| 引入前端框架/打包器/npm 依赖 | 本仓库刻意零依赖（`rust/client/main.js` 是唯一脚本） | 原生 JS + 现有资源管线 |
| 随意升级依赖 | `--locked` 发布；锁文件漂移会改变产物 | 有意升级时单独提交并跑全套验证 |
| 在 `serve.rs`/`routes.rs` 里加阻塞 IO | 动态模式在 async 上下文用 `spawn_blocking` 是有意的 | 保持异步边界 |

## 兼容性红线

- 输出兼容以 `nobelog diff` 为准：为了让 diff 变绿，**不要**去改 `public/` 基准，改你的实现。
- `markdown.rs` 与 `templates.rs` 的目标是复刻 TS 产物；"顺手优化输出"必须明确征得同意，
  因为会同时改变归档、RSS、sitemap 与所有既有页面的字节。
- 代码块高亮是唯一被允许的逐字节差异（syntect vs highlight.js），但 hljs 类名结构与 DOM 结构不能变。

## 常见事故

1. **样式上线后丢失**：CSS 裁剪误删（动态拼接的 class）。先 `NOBELOG_CSS_PRUNE=0` 复现，
   再修 `css_prune` 规则，别长期关掉裁剪。
2. **中文标签页 404**：手拼 URL 漏了 `url::encode`，或直接在 `build.rs` 的磁盘路径用了转义后的串。
3. **新文章日期显示"刚刚"**：`card_date` 按 UTC+8 挂钟比较，frontmatter 写的是本地时间；
   写了未来日期会直接显示绝对日期。
4. **草稿被发布**：`draft: true` 在 `content::load` 过滤；若在别处绕过 `load` 直接读文件就会漏。
5. **改了代码但线上没变**：服务跑的是 release 二进制，需重新 `cargo build --release` 并重启单元。
6. **RSS 时间偏 8 小时**：`Post::date` 是 UTC+8 挂钟，RSS 必须经 `templates::rss_pub_date` 减 8h；
   新增绝对时间输出（JSON-LD、sitemap lastmod 等）时同样要换算，别直接序列化 `DateTime<Utc>`。
7. **构建报 slug/标签非法**：内容层与 `file_for_route` 都有校验（拒绝 `/`、`\`、`.`/`..` 段、控制字符），
   这是安全拦截不是 bug；改内容命名，不要放宽校验。

## 文档与流程

- 项目规则入口：`AGENTS.md`、`.trellis/workflow.md`；本目录 spec 由 `00-bootstrap-guidelines` 建立，后续用
  `trellis-update-spec` 增量维护。
- 会话收尾：`task.py finish` → `task.py archive <name> --skip-branch-validation` → 写 journal。
- 发现新约定（例如新增了一类必须的验证），先更新本目录，再写代码。
