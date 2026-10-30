# 回退小熊线稿测试图标

## Goal

用户说明上一轮换图标仅为测试；本任务把站点图标回退到 10-21 粉笔少女版本，删除小熊源图，并恢复本地站点原图标。

## Requirements

1. **revert 图标改动**：对提交 `b07a0cd` 执行 `git revert --no-commit`，恢复 `content/images/site-icon` 七件套为 `28f80ea` 版本，删除 `tools/assets/lying-bear-icon-source.jpg`。
2. **不撤销的内容**：保留 `9157153` 的 spec 修正，即构建输出目录与 SITE_URL 验证说明、图标产物命名修正；保留已归档的 10-30-icon-bear-lineart 任务记录与 journal Session 38。
3. **提交规范**：提交消息使用 `revert:` 前缀并带任务锚点 `[task:10-30-revert-bear-test]`；不 push、不打包、不重启服务。
4. **即时生效验证**：本地 `nobelog-local.service` 动态模式直读 `content/images`，回退文件后验证 7 个图标 URL 返回 200、内容 sha256 与回退后磁盘文件一致、首页 head 7 条声明在位。
5. **清理测试预览**：删除本轮测试生成的 `.run/review/lying-bear-icon/` 预览目录；不改动仓库根 `dist/` 中已恢复的 v0.0.29 发布包。

## Acceptance Criteria

- [ ] `git diff --cached 28f80ea -- content/images/site-icon* tools/assets/lying-bear-icon-source.jpg` 为空：7 个图标与 28f80ea 完全一致，小熊源图已删除。
- [ ] revert 提交已生成，消息带 `revert:` 前缀与 `[task:10-30-revert-bear-test]` 锚点。
- [ ] 本地 7 个图标 URL 返回 200，sha256 与工作区文件一致，首页 head 6 条 link 加 1 条 og:image。
- [ ] `.run/review/lying-bear-icon/` 已删除；`dist/nobelog-0.0.29-linux-x86_64.tar.gz` 与 SHA256SUMS 未被触碰。
- [ ] 未 push、未打包、未重启服务；工作区除 Trellis 任务文件外干净。

## Constraints

- 不执行 `git reset`，不重写历史；用 revert 提交保留审计轨迹。
- 不撤销 spec 修正与 v0.0.29 恢复成果。
- 不修改模板、public/、vendor/。

## Notes

- 回退目标为提交 `28f80ea` 的图标内容，即 10-21 粉笔少女细节版。
- 用户原话：回退吧，这次换图标仅测试而已。
