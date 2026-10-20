# 核对远程站点 z.miio.cc 是否已包含修复

## Goal

在用户授权只读的前提下，判断 https://z.miio.cc/ 是否已包含 v0.0.22 的修复，并指出哪些修复无法从外部确认。

## 只读检查项

| 检查 | 期望（已修复） | 方法 |
| --- | --- | --- |
| RSS 时区 | `/feed.xml` 的 `pubDate` 为 `14:40:00 GMT` / `04:15:00 GMT` | GET |
| 确定性 CSS | 首页 link 指向 `main.1f82b0f7.css`（或与 `/assets/assets-manifest.json` 一致） | GET |
| canonical | `https://z.miio.cc/`（说明远端设置了 SITE_URL 或反代透传正确） | GET |
| sitemap | 绝对地址为 `https://z.miio.cc/...` | GET |
| 注入头 | `X-Forwarded-Host` 不应改变 canonical（若已设 SITE_URL 则天然忽略） | 只读 GET，单次 |
| 404 | 未知路径返回 404 | GET |
| 缓存头 | `/images/*` 为 `no-cache`；`/assets/*` 长缓存 | HEAD |
| 版本线索 | 响应头/页面生成器标记、发布时间等 | GET |

## 约束

- 只允许 `https://z.miio.cc/` 的 GET/HEAD；不做 POST、不做漏洞利用、不扫描、不访问其它主机。
- 不改配置、不部署、不重启。

## Acceptance Criteria

- [ ] 给出“是否已修复”的结论，并区分「外部可确认」与「无法从外部确认」两类。
- [ ] 每条结论附请求/响应证据（状态码、关键 header/字段）。
- [ ] 若发现远程落后，给出与本地 v0.0.22 的差异点。
