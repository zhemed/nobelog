# 发版 0.0.37

## 内容

无功能变化：Rust 固定 1.99.1、`tools/check-rust.sh` 自检、package.sh 走 PATH、README 工具链章节。作为工具链锁定的可追溯点发布。

## 流程与验证

1. 版本 0.0.36 → 0.0.37；门禁 fmt/clippy/test(98) 全绿，`--version` = 0.0.37；
2. 打包 `VERSION=0.0.37 tools/package.sh`（打包前自检 rustc/cargo = 1.99.1）；sha256 通过；解包版本正确；
3. commit + tag `v0.0.37` + push（含积压的 11 个提交）+ `gh release create`；
4. 重启线上服务（二进制无变化，保持一致）。
