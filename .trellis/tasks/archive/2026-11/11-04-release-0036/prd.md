# 发版 0.0.36

## 内容

修复「系统深色下白底白字」：auto 模式解析成 light/dark 写入 data-mode（首屏脚本 + 主脚本），并覆盖 github-markdown 的深色媒体查询变量。

## 流程与验证

1. 版本 0.0.35 → 0.0.36；门禁 fmt/clippy/test(98) 全绿，`--version` = 0.0.36；
2. 打包 `VERSION=0.0.36 tools/package.sh`；sha256 通过；解包版本正确；包内 HTML 含新的早脚本；
3. commit + tag `v0.0.36` + push + `gh release create`；
4. 重启线上服务。
