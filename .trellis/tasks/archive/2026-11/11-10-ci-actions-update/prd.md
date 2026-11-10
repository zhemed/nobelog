# 维护 CI actions 版本

## Goal

按用户 2026-11-11「第一你维护一下吧」，修复 CI 首跑暴露的两条非阻断 annotation：

1. `actions/checkout@v4`、`actions/cache@v4` 仍指向 Node20（GitHub 已强制在 Node24 上运行）；
2. `ubuntu-latest` 将于 2026-10-19 迁移 Ubuntu 26（仅提示，无需改）。

做法：把两个官方 action 升级到当前最新 major（2026-11-10 查证：checkout `v7.0.1`、cache `v6.1.0`），工作流其余行为不变。

## Requirements

- R1 只改 `.github/workflows/ci.yml` 中两处 `uses:`：`actions/checkout@v4` → `actions/checkout@v7`；`actions/cache@v4` → `actions/cache@v6`。
- R2 不改变工作流其他行为：触发条件、`permissions`、`concurrency`、缓存 `path`/`key`/`restore-keys`、步骤顺序与命令。
- R3 版本真实存在：用 `gh api repos/<action>/releases/latest` 或 tags 查证 major 可用；YAML 可被 PyYAML 解析。
- R4 推送后 CI 再次全绿；Node20 deprecation annotation 不再出现（ubuntu-latest 迁移提示属 runner 侧信息，非本仓库可修）。
- R5 不引入第三方 action、不改其它文件。

## Acceptance Criteria

- [x] `ci.yml` 仅两处 `uses:` 版本变化（`git diff` 可证）。
- [x] `python3 -c "import yaml; yaml.safe_load(...)"` 通过；两个 action 的 major tag 在 GitHub 真实存在。
- [x] 推送后 CI run 全绿（`gh run watch --exit-status`），annotation 中不再有 Node20 deprecation。
- [x] 提交带任务锚点 `[task:11-10-ci-actions-update]`。

## Notes

- 官方 action 的 v7/v6 均为 ESM 迁移，输入接口不变（checkout 无输入；cache 仍用 path/key/restore-keys），可直接替换。
- 用户原话与边界见 `directive.md`。

## 验证结果（2026-11-10，verifier 独立复核）

- 验收结论：7 项全 PASS（含改动范围、CI YAML、action 真实存在性、门禁复跑），无 FAIL、无越界。
- `git diff -U0 -- .github/workflows/ci.yml`：仅 :35 `checkout@v4→v7`、:38 `cache@v4→v6` 两行变化，其余行未动。
- `gh api` 复核：`actions/checkout` latest v7.0.1、`actions/cache` latest v6.1.0；两者 action.yml 均为 `node24`（v4 为 node20），cache 输入接口不变。
- 推送后 CI run [#42860166864](https://github.com/zhemed/nobelog/actions/runs/42860166864) `success`（2m25s，11 步全绿）；Node20 deprecation annotation 已消失，仅剩 runner 侧 `ubuntu-latest` 迁移提示（非本仓库可修）。
- 提交：`ab6d0cb ci: 升级官方 actions 到 checkout v7 / cache v6 [task:11-10-ci-actions-update]`。
