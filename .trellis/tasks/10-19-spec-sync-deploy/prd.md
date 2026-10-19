# 同步 spec 时区约定并部署 P1 修复

## Goal

让 `.trellis/spec/` 与修复后的实现一致，并把 P1 修复提交、推送、构建 release、重启 `nobelog-local` 生效。

## 背景

- 审查（归档任务 `10-19-blog-audit`）确认三条 P1，修复任务 `10-19-fix-p1-audit` 已完成：
  slug/标签/分类路径穿越双层拦截、Host/XFH 白名单 + base_url 全量转义、RSS pubDate 时区；
  复审两轮后补了 RFC3339 归一化、mtime 回退归一、带空格偏移容错。52 个测试全绿。
- spec 旧文档写"时区统一 UTC"，与实现（UTC+8 挂钟存储、RSS 输出真实 UTC）不符。

## Requirements

- R1 更新时区相关 spec：`content-and-frontmatter.md`、`rendering-templates.md`、`quality-guidelines.md`；
  明确：内部存储 = UTC+8 挂钟；RFC3339 与 mtime 都归一化；RSS 输出真实 UTC；带偏移写法现已支持。
- R2 提交消息带 `[task:10-19-spec-sync-deploy]`，包含 5 个源文件 + 测试 + spec + Trellis 记录；推送 `origin/main`。
- R3 部署：`cargo build --release --locked`，重启 `nobelog-local.service`，验证服务健康（8 条路由、feed pubDate 已变、无 5xx）。
- R4 若 `SITE_URL` 未设置，保持启动告警（不臆造域名，不写死值）。

## Acceptance Criteria

- [ ] spec 与实现一致，无"统一 UTC"旧说法。
- [ ] `git status` 干净，`origin/main` 同步，提交含任务锚点。
- [ ] release 二进制 mtime 更新，服务重启后 pid 变化、8 条路由 200/404 预期、feed pubDate `14:40:00 GMT`。
- [ ] 线上 canonical 不再受 XFH 注入影响（带恶意头实测）。
- [ ] 收尾：journal + 归档。
