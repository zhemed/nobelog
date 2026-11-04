# 发版 0.0.35

## 内容

0.0.34 之后：桌面侧栏导航上移（123→99px）、手机抽屉侧栏整体上移（标题 84→36、导航 282→198）。

## 流程与验证

1. 版本 0.0.34 → 0.0.35；门禁 fmt/clippy/test(98) 全绿，`--version` = 0.0.35；
2. 打包 `VERSION=0.0.35 tools/package.sh`；sha256 校验通过、解包版本正确、包内 4 处 example.com；
3. commit + tag `v0.0.35` + push + `gh release create`；
4. 重启线上服务。
