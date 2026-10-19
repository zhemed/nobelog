# 修复 P3：符号链接指纹、.mdx 扫描、保留名大小写、updated 校验

## Goal

清掉 P2 复核残留的四条 P3，让热更新、CSS 裁剪与内容校验在三处边界上行为一致。

## 要求

1. **`rust/src/serve.rs`**：`content_stamp` 用 `entry.file_type().is_file()` 会跳过符号链接，而 `content.rs::read_posts`
   会跟随链接加载 → 链接文章改动不触发重建。改用 `metadata()`（跟随链接）或显式拒绝 content 下符号链接，二选一并写测试。
2. **`rust/src/assets.rs`**：CSS usage 的正文扫描只含 `content/**/*.md`，而文章接受 `.mdx` → 补 `.mdx`。
3. **`rust/src/content.rs`**：page slug 保留名大小写敏感（`Tags` 可通过）且缺 `assets`/`images`/`CNAME`/`robots.txt` 等；
   统一为小写比较并补齐保留名单（含输出根文件）。补测试。
4. **`rust/src/content.rs`**：`updated` 字段非法时静默忽略 → 与 `date` 一致 fail-fast（带文件路径）。补测试。
5. 不改现有行为语义之外的东西；不引入依赖。

## 约束

- 不提交、不推送、不重启服务、不 `cargo build --release`。
- 三块分属不同文件，分别由对应成员完成：serve.rs / assets.rs / content.rs。

## Acceptance Criteria

- [ ] 四条 P3 均修复并带测试。
- [ ] `cargo fmt --check`、`cargo clippy --all-targets`、`cargo test` 全绿。
- [ ] 仓库内容构建 15 页不变；与冻结 `public/` 的差异不扩大。
- [ ] 完成后由 Lead 提交、推送并按用户授权部署。
