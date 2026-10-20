# 默认不信任 X-Forwarded-*（SITE_URL 未设时）

## Goal

修掉本地验收 FAIL：`SITE_URL` 未设置时，任意客户端只要发一个语法合法的 `X-Forwarded-Host`
就能改写动态模式的 canonical / og:url / RSS / sitemap 站点地址。改为**默认忽略** `X-Forwarded-*`，
只有显式开关才信任；`SITE_URL` 仍然最高优先。

## 方案

1. `serve.rs::request_base_url` 拆成显式参数的纯函数：
   - `SITE_URL` 有值 → 直接用它（不变）。
   - `SITE_URL` 无值 + **默认**：只用 `Host` 头（仍走现有字符白名单），忽略 `X-Forwarded-Host`/`X-Forwarded-Proto`，scheme 固定 `http`；Host 非法 → fallback。
   - `SITE_URL` 无值 + 显式开关 `NOBELOG_TRUST_FORWARDED=1`（接受 `1/true/yes/on`）→ 现有行为：优先 XFH + XFP。
2. 启动时若 `SITE_URL` 未设，告警文案补充"默认不信任 X-Forwarded-*，如需请设 NOBELOG_TRUST_FORWARDED=1"。
3. 补单测：
   - 默认（trust=false）：XFH=evil.example.com + Host=good.example → 结果用 good.example（http）；
   - XFH 含 `@`/非法 → 回退 Host；
   - trust=true：XFH 优先、XFP https 生效；
   - 无 Host 且 trust=false → fallback。
4. 更新 `.trellis/spec/frontend/build-serve-deploy.md` 的 SITE_URL/拓扑说明：默认不信任转发头，生产建议设 SITE_URL。
5. 验证：门禁全绿；临时实例复跑三种 XFH 请求（默认下 canonical 不变）；重建 release 并重启 `nobelog-local`（用户已选"重启本机"），复跑本机安全回归。

## 约束

- 不改其它行为；不引入依赖。
- 不访问远程；测试用临时实例（高位端口）与本机 10086。
- 发版（tag/Release）不在本轮，除非用户随后要求。

## Acceptance

- [ ] 默认配置下 `X-Forwarded-Host: evil.example.com` 不影响 canonical/og/feed/sitemap。
- [ ] `NOBELOG_TRUST_FORWARDED=1` 时恢复旧行为；`SITE_URL` 始终最高优先。
- [ ] 90+ 测试全绿、clippy 0 warning、fmt 干净。
- [ ] 本机服务重启后仍健康，安全回归 PASS。
