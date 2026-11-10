# 用户当轮原话（逐字记录）

> 就听你的，开始修复吧。

## 当轮上下文与边界

- 上一轮 Lead 对用户提供的 6 条加固建议做了只读评估，给出优先级：
  1. CI workflow（最高性价比）；
  2. `tools/fetch-baseline.sh`（从 GitHub Release 还原 `public/` 冻结基线）；
  3. panic/strip 取舍（全局 `panic=unwind` + `tower-http` catch-panic，或维持现状）；
  4. fixture / 拆 `serve.rs` / HTML 校验 / 拆 `.trellis` 暂不做。
- 用户「就听你的」= 按该优先级实施第 1、2、3 项，第 4 项不做。
- 授权范围：仓库内改动（CI、工具脚本、Cargo profile、serve 路由边界、配套文档）。
- 不含：推送远端、发版、部署变更、版本号变更、内容/模板输出变更。
- 指令时间：2026-11-11 00:12（Asia/Shanghai）。
