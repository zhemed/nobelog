# 只读盘点与工作流放置评估记录

日期：2026-10-30；执行：Lead 与团队成员 trellis-scout；证据以 nobelog 仓库与 /root/博客 工作区为准。

## 1 任务与残留

- 活跃任务：1 个，即本任务 10-30-trellis-residue-placement，状态 planning。
- 归档任务：42 个，全部 status=completed 且 completedAt 齐全；task.py list-archive 显示 2026-10 共 42 个；journal 的 42 个 Session 与之基本对应。Lead 复核：脚本统计 42/42 为 completed，无缺失 completedAt。
- 活跃树中没有未闭环 Trellis 任务；git stash 为空，无暂存改动；工作树唯一未跟踪项是当前任务目录。
- 低风险运行期残留：.trellis/.runtime 内旧会话状态与 update-check marker，被 .trellis/.gitignore 忽略；scripts/common/__pycache__ 内 21 个 .pyc；空目录 .run/review/。
- 旧世代残留：/root/博客/.backups 内 3 个备份包含旧 .trellis 快照，其中存在未归档任务，例如 10-17-freeze-backup、00-bootstrap-guidelines、10-17-rust-default、10-17-rust-toolchain-pin；旧 journal 共 12 个 Session，最后活跃 2026-10-17。Lead 复核：nobelog-src-20261017-020319.tar.gz 内 10-17-freeze-backup 与 00-bootstrap-guidelines 的 task.json 位于 tasks 根而非 archive。
- 根目录 /root/博客 只含 nobelog 与 .backups，没有 .trellis，也没有 .git；.backups 约 11M，SHA256SUMS 校验 5/5 通过（由团队执行，Lead 未复跑）。
- journal 中的业务待决项，例如 SITE_URL 注入、公网反代验证、README 与路由文档过时、2.3.4 原文待提供，属于用户决策事项，不是任务残留。

## 2 Git 与仓库

- nobelog 分支 main 跟踪 origin/main，无 ahead 或 behind，HEAD 为 daa1212。
- .trellis 已纳入版本控制，git ls-files 计 266 个文件；.developer 与 .runtime 被 .trellis/.gitignore 忽略。
- /root/博客 不是 git 仓库；nobelog 是工作区内唯一仓库。
- 历史决策：10-19-enable-trellis/prd.md 记录「在 nobelog 仓库内启用 Trellis 并把产物提交进仓库」。

## 3 放置评估

结论：维持 .trellis 位于 nobelog 项目根，不迁移到 /root/博客 主工作区。

理由：

1. 主工作区不是 git 仓库，任务与提交绑定、归档提交、journal 自动提交都会降级为仅写盘。
2. .trellis 的 266 个跟踪文件与 .agents、.dsh、AGENTS.md、.gitattributes 同批提交，单独迁走会分裂工作流载体与合并策略。
3. 42 个任务与 144 个提交的历史都在 nobelog，迁移会制造大规模删除提交并切断历史关联。
4. 2026-10-19 的既有决策与本轮用户选择一致。

替代方案：

- 以 /root/博客/nobelog 作为 DSH 会话工作区：零迁移即可让 SessionStart 命中 .trellis，推荐采用。
- 根目录软链接：路径解析层面可行，但 git 降级问题仍在，创建软链需授权，且全局钩子是否识别软链未验证。
- 实体迁移或根目录独立 init：高风险或会造成两套并存，需单独立项并取得明确授权。

未验证项：全局 git 钩子具体逻辑、全局 DSH SessionStart 检测方式、非 git 目录 trellis init 行为，均在 /root/博客 之外，未读取或执行。

## 4 建议

- 维持现状；后续会话工作区使用 /root/博客/nobelog。
- 旧备份保留，禁止解压覆盖当前 .trellis；如需清理或迁移旧历史，另立任务并授权。
- 运行期残留可选清理，均被 git 忽略，不清理也不影响功能。

## 5 本轮处置

- 用户选择：清理运行期残留；旧备份保留并只读留档。
- 已清理：.trellis/.runtime 内 2 个旧会话 update-check marker，保留当前会话 marker；删除 .trellis/scripts/common/__pycache__；删除空目录 .run/review 与空 .run。
- 未动：3 个旧备份包保持原样，未解压、未覆盖、未删除。
- 复核：清理只涉及 git 忽略项；git status 仍只有当前任务目录未跟踪。
