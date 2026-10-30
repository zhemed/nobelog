# 盘点 Trellis 残留任务并评估工作流放置

## Goal

1. 只读盘点 nobelog 的 .trellis 任务目录与工作区残留，判定是否存在未闭环任务或遗留文件。
2. 评估 .trellis 工作流是否应放在主工作区 /root/博客，交付盘点报告与放置评估结论。
3. 不改动业务代码，不执行任何未授权写操作。

## Requirements

- R1 事实盘点：列出 .trellis/tasks 下全部活跃与已归档任务，含状态、日期、产物完整性与闭环情况；证据以仓库文件与只读命令输出为准。
- R2 残留判定：识别工作区内未闭环或遗留项，覆盖 .trellis/.runtime、workspace journal 线索、git 未跟踪或未提交文件、/root/博客 根目录与 .backups；每项附路径与时间证据。
- R3 放置评估：说明 .trellis 位于 nobelog 项目目录与移到主工作区的差异，给出推荐方案、理由、技术阻碍与替代方案。
- R4 授权边界：迁移、删除、清理等写操作不在本任务内执行，需用户当轮明确授权后另行处理。
- R5 流程留痕：过程与结论写入本任务目录，收尾按 Trellis finish 与 archive 流程执行。

## Acceptance Criteria

- [ ] 报告列出全部任务及状态，残留项逐条有证据路径与时间。
- [ ] 放置评估含明确推荐、理由、技术阻碍与替代方案。
- [ ] 至少一名团队成员完成只读侦察，Lead 复核关键证据。
- [ ] 未执行任何未授权写操作，业务代码零改动。
- [ ] 任务完成归档并记录 journal。

## 背景事实

- 主工作区 /root/博客 不是 git 仓库，根目录无 .trellis；nobelog 是其中唯一 git 项目，Trellis 开发者 zhemed，分支 main，工作区干净。
- 用户本轮明确要求建立新任务，并已选择在 nobelog 项目内继续。

## Notes

- 本任务为轻量调查与评估任务，PRD-only 即可，无需 design.md 与 implement.md。
- 本轮新增的只读侦察由团队成员 trellis-scout 承担，团队任务为 task-2。
