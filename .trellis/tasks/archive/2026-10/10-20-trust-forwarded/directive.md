# 用户当轮原话（逐字记录）

> 远程不需要我们负责，你把本地测试好就行

## 当轮选择（ask_user_question）

本地验收发现 FAIL（SITE_URL 未设置时合法 `X-Forwarded-Host` 可改写 canonical/feed）后选择：
**改代码默认安全**（SITE_URL 未设置时默认忽略 X-Forwarded-Host/Proto，显式开关才信任；补测试、重跑验收、重启本机；发版另议）。
