# 用户当轮原话（逐字记录）

> 域名怎么可能定呢？这是发到GitHub得，你真的觉得没问题吗？

> （选择弹窗）发布去向：只发 GitHub Release 包

## 当轮上下文与边界

- 用户指出：发布包要传到公开 GitHub，不能把真实域名写进包里；此前反复追问“定哪个域名”的问法错误。
- 结论：发布流程不需要真实域名。包一律用中性域名 `https://example.com` 构建后上传 GitHub Release；
  真实域名只在部署机上用 `SITE_URL` 重建 `public/`，或改用动态模式按请求 Host 自动识别。
- 版本：0.0.30 → 0.0.31（v0.0.30 已存在 tag）。
- 边界：不把任何真实域名写入仓库、构建产物或 Release notes；`tools/banned-domains.txt` 保持本地未入库。
