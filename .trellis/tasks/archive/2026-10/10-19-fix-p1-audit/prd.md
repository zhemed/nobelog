# 修复审查发现的 P1 问题

## Goal

修复博客审查（task `10-19-blog-audit`）确认的三条 P1，并补回归测试。**不提交、不推送、不重启线上服务。**

## 问题与修法

### P1-1 构建期路径穿越（slug / 标签 / 分类未校验）

- 注入点：`rust/src/content.rs:90`（slug 原样）、`content.rs:103-104`（tags/categories），
  `rust/src/build.rs:124-141`（`file_for_route` 解码后 `out.join` 不校验）。
- 修法：
  1. 内容层校验：文章/单页 slug 拒绝空值、`/`、`\`、`.`/`..` 段、控制字符；标签/分类拒绝 `/`、`\`、控制字符。
     失败时用 `anyhow` + 文件路径上下文**让构建失败**，不要静默跳过。
  2. 磁盘层兜底：`file_for_route` 逐段拒绝 `.`/`..`/空段与绝对路径，写文件前对父目录做 canonicalize 前缀断言。
  3. 测试：`file_for_route` 对 `../../x`、`%2e%2e/...`、`/etc/passwd` 必须报错；内容校验单测。

### P1-2 动态模式 Host / X-Forwarded-* 注入 + URL 未转义

- 注入点：`rust/src/serve.rs:219-241`、`templates.rs:758/770/777/362/1487/1519/1522/1535`。
- 修法：
  1. `request_base_url`：`X-Forwarded-Proto` 只接受 `http`/`https`；host 只接受 `[A-Za-z0-9.\-:\[\]]`
     （含端口），出现 `"`/`<`/`>`/`@`/`/`/`\`/空白/控制字符一律丢弃该头并回退；
     `Host` 头同样校验；全部非法时用传入 fallback。
  2. 模板层所有 `base_url` 插值：HTML 走 `escape_html`，RSS/sitemap 走 XML 转义函数；抽一个 `escape_xml`。
  3. `site.rs`：`SITE_URL` 未设置时启动告警一次（不拒绝启动，避免影响现有部署）。

### P1-3 RSS pubDate 时区偏移 8 小时

- 现状：frontmatter 日期按 UTC+8 挂钟存储（`templates.rs::card_date` 的注释与实现假定如此），
  但 `feed_xml` 把该挂钟值直接当 GMT 输出 → 整份 feed 偏 8 小时。
- 修法：输出 RSS 前把挂钟值换算为真实 UTC（减 8 小时）再格式化 GMT；`lastBuildDate` 保持真实 UTC。
  在 `content.rs`/`feed_xml` 附近写明这一约定（挂钟 = UTC+8，序列化为 UTC）。
- 测试：构造已知日期，断言 feed `pubDate` 比 frontmatter 挂钟早 8 小时。

## 约束

- 只改代码与测试；**不提交、不推送、不重启 nobelog-local.service**。
- 验证用 debug 构建或现有 release 二进制；**不要 `cargo build --release`**（生产二进制保持不动）。
- 不引入新依赖。

## Acceptance Criteria

- [ ] 三条 P1 修复并带回归测试。
- [ ] `cargo fmt --check`、`cargo clippy --all-targets`、`cargo test` 全绿。
- [ ] 用修复后的 debug 二进制复跑 PoC：slug `../..` 不再逃出 out；恶意 XFH 不再进入 canonical/feed。
- [ ] 静态构建 `diff` 与冻结 `public/` 的差异不扩大（除既有的时间敏感/RSS 时区修正）。
- [ ] 线上服务未被重启，`git status` 只含本次代码改动与未提交的 Trellis 记录。
