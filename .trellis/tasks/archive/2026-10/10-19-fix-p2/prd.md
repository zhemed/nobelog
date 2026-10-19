# 修复 P2：CSS 确定性、动态热更新可靠性、内容/渲染一致性

## Goal

按 `10-19-blog-audit` 三份报告修复 P2 前几项，目标是：同一 commit 的静态构建与动态模式产出一致、
动态热更新可靠（删文/失败可见）、内容错误不再静默、渲染在边界输入上行为正确。**不提交、不部署。**

## 范围与要求

### A. CSS 确定性（`rust/src/assets.rs`、`rust/tools/css_prune.rs`、`rust/src/build.rs`）

- 现状：CSS 裁剪的使用集合优先扫 `public/**/*.html`，否则扫 `rust/src` → 发布包与动态模式产出两套
  `main.<hash>.css`（0f58b567 vs b174a317），且条件渲染类（`.pagination`、`.row-cols-md-2>*`、`.preview-img`）
  在对应页面状态出现时被误裁。
- 目标：使用集合与 `public/` 是否存在无关——改为「本次实际渲染的全部路由 HTML」∪「`rust/src/**/*.rs` 源码字面量」，
  或等价地显式合并两路；条件类补白名单（`pagination`/`page-*`/`row-cols-*`/`preview-img` 等）。
- 要求：静态构建（`--out public` 与 `--out /tmp/x`）与动态模式（`build_in_memory`）输出**同一份 CSS**；
  加回归测试断言两条路径哈希一致；保留 `NOBELOG_CSS_PRUNE=0` / `NOBELOG_CSS_REPORT=1` 语义。

### B. 动态热更新可靠性（`rust/src/serve.rs`）

- 删除文章/回退旧 mtime 检测不到：`content_stamp` 改为能反映「文件集合 + 大小 + mtime」的指纹（或等价方案）。
- 重建失败静默：至少 `eprintln!` 记录错误，并加失败退避（避免坏内容导致每请求重建）。
- 每请求同步 `walkdir content/`（含 404）：加短 TTL 缓存（如 1s）或等价方案，避免高频 stat。
- `content/images/**` 沿用了 `ASSET_CACHE`（一年）→ 改用可变内容缓存策略（如 `PAGE_CACHE`/`NO_CACHE`）。
- 补单测：stamp 对新增/删除/同 mtime 变化的判定；图片缓存头。

### C. 内容与渲染一致性（`rust/src/content.rs`、`rust/src/markdown.rs`、`rust/src/routes.rs`、`rust/src/templates.rs`）

- 非法 YAML 静默当 Null → `anyhow` 带文件路径失败。
- 重复 slug 静默覆盖 → 构建报错并指出冲突的两篇文章。
- 无 description 时摘要双重转义（`&amp;amp;`）→ 只解码/转义一次，补测试。
- 提示块闭合不识别代码围栏 → 围栏内的 `:::` 不得关闭提示块，补测试。
- sitemap 与 `page()` 不一致（中文/下划线 page slug 进 sitemap 但 404）→ 统一：要么限制 page slug 字符集并 fail-fast，
  要么让 `page()` 支持这些 slug；以「内容校验 + 路由一致」为准，补一致性测试。
- 可选：`format_date` 在 feed/sitemap 的用途注释已修正；不再扩展范围。

## 约束

- 三块由不同成员并行，各自只改自己的文件（A：assets.rs/css_prune.rs/build.rs；B：serve.rs；
  C：content.rs/markdown.rs/routes.rs/templates.rs）。
- 不提交、不推送、不重启服务、不 `cargo build --release`。
- 不引入新依赖；`cargo fmt --check`、`clippy --all-targets`、`test` 全绿。
- 静态构建整站 `diff` 与冻结 `public/` 的差异不得扩大（既有 feed/index/CSS 哈希除外）。

## Acceptance Criteria

- [ ] A/B/C 三块各自修改完成 + 单测。
- [ ] 三块合并后 `cargo test` 全绿，`diff` 无新增非预期差异。
- [ ] 两套 CSS 问题消失（静态/动态同哈希）。
- [ ] 动态删文、坏文章、图片缓存、非法 YAML、重复 slug、提示块围栏、sitemap 一致性均有测试覆盖。
- [ ] 线上服务未被重启；工作区只含源码与 Trellis 记录改动。
