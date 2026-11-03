# 发版 0.0.33

## 内容

0.0.32 之后的改动：右侧栏比例宽度、撤回右对齐、手机端许可/分享单行并垂直对齐。

## 流程与验证

1. 版本 0.0.32 → 0.0.33（Cargo.toml + Cargo.lock）；
2. 门禁：`fmt` / `clippy` / `test`（98）全绿，`nobelog --version` = 0.0.33；
3. 打包：`VERSION=0.0.33 tools/package.sh`（SITE_URL 默认 example.com）；
4. `sha256sum -c SHA256SUMS` 通过，解包 `--version` = 0.0.33，包内含 4 处 example.com；
5. commit + tag `v0.0.33` + push + `gh release create`；
6. 重启线上服务。
