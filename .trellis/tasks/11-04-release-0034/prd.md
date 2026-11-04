# 发版 0.0.34

## 内容

0.0.33 之后：右栏比例宽度、正文图片撑满、侧栏与内容整体按比例（侧栏 16.5vw、正文 62%）、侧栏标题与导航加大（装饰线加大已回滚）。

## 流程与验证

1. 版本 0.0.33 → 0.0.34；门禁 `fmt` / `clippy` / `test`（98）全绿，`--version` = 0.0.34；
2. 打包 `VERSION=0.0.34 tools/package.sh`（SITE_URL 默认 example.com）；
3. `sha256sum -c` 通过、解包版本正确、包内 4 处 example.com；
4. commit + tag `v0.0.34` + push + `gh release create`；
5. 重启线上服务。
