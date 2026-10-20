# 核对残留任务与仓库状态

## Goal

回答用户"是否还有残留任务"，给出 Trellis 任务、团队任务板、成员状态与仓库工作区四项证据。

## Acceptance

- [x] `task.py list` 无其它活动任务（仅本轮临时任务）。
- [x] 团队任务板全部 completed。
- [x] 所有 teammate inactive。
- [x] `git status` 干净、与 origin 同步。
- [x] 本轮临时任务归档，不留下新的"残留"。
