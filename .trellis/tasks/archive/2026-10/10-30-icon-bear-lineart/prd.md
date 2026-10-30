# 切换站点图标为小熊线稿

## Goal

把用户 2026-10-30 提供的小熊线稿图接入 nobelog 站点图标体系，替换当前粉笔少女图标，并让本地动态站点立即生效。

## Requirements

1. **源图归档**：把用户附件图复制为 `tools/assets/lying-bear-icon-source.jpg`，权限 644；源图只放 `tools/assets/`，不放进 `content/images/`，避免进入发布包。
2. **重新生成图标**：在仓库根执行 `python3 tools/make-icons.py tools/assets/lying-bear-icon-source.jpg`，沿用脚本默认 lineart 模式；只写 `content/images/`，产出固定 7 个文件：`site-icon.svg`、`site-icon-32-transparent.png`、`site-icon-192-transparent.png`、`site-icon-32-dark.png`、`site-icon-192-dark.png`、`site-icon-180.png`、`site-icon-512.png`。文件名与集合保持不变。
3. **不改模板与产物基准**：`rust/src/templates.rs` 的 7 条 head 图标声明与 `og:image` 保持原样；不手改 `public/`、`vendor/`；不修改 `tools/make-icons.py` 逻辑。
4. **预览证据**：脚本默认预览目录 `.run/review/lying-bear-icon/` 产出 16 与 48 的亮暗小图，用于小尺寸可辨识性人工检查，预览不入库。
5. **质量门禁**：`cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test` 全绿。
6. **构建对比**：执行 `cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out dist`，再执行 `./rust/target/release/nobelog diff public dist`；预期仅 7 个 `images/site-icon*` 文件不同，HTML、RSS、sitemap 零差异，diff 退出码 1 属于预期。
7. **线上生效验证**：本机 `nobelog-local.service` 为动态模式且直接读 `content/images`，替换文件即生效，不重启服务；验证 `http://127.0.0.1:10086/images/site-icon*` 七个 URL 返回 200、sha256 与磁盘文件一致、`Cache-Control: no-cache`；首页 head 仍为 7 条图标声明。
8. **不推送不发版**：本次只做本地仓库与本地站点，不执行 git push、不打包 release、不重启服务；是否推送与发版由用户另行决定。

## Acceptance Criteria

- [ ] `tools/assets/lying-bear-icon-source.jpg` 存在，权限 644，尺寸 2048x2048。
- [ ] `content/images/` 下 7 个 site-icon 文件全部由新源图重新生成：透明版为 RGBA、不透明版为 RGB，尺寸与脚本常量一致，SVG 内含亮色与暗色两张内嵌 PNG。
- [ ] 16 与 48 亮暗预览人工确认可辨识、无白框、无异常色块。
- [ ] `rust/src/templates.rs`、`public/`、`vendor/` 均无改动。
- [ ] fmt / clippy / test 全绿。
- [ ] build 加 diff：仅 7 个图标文件不同，HTML 与 feed/sitemap 无差异。
- [ ] 本地 10086 的 7 个图标 URL 校验通过，首页 head 7 条声明在位。
- [ ] 提交只含源图与 7 件套，提交消息带 `[task:10-30-icon-bear-lineart]`。

## Constraints

- 不编辑 `public/`：它是 gitignore 的冻结基准，仅用于 diff 对比。
- 不删除 `tools/assets/` 历史源图：bear、cat、girl、chalk-girl 全部保留。
- 不重启 `nobelog-local.service`；动态模式读磁盘资源，无需重启即可验证。
- 本任务不涉及发布：不打包、不打 tag、不推送。

## Notes

- 图标管线由任务 10-21-bear-icon 引入，lineart 模式针对白底线稿设计：暗像素 bbox、5% 内边距、方形画布、透明版 flood fill、暗色变体重映射、SVG 内嵌明暗两张 PNG。
- 当前线上图标来自 10-21-chalk-girl 系列任务，本次以新小熊线稿整体替换。
- 预览目录名取源图 stem 去掉 `-source` 后的 `lying-bear-icon`。
