# 编写工作区 Trellis 使用指引

## Goal

在 /root/博客 工作区根目录编写 Trellis 使用指引 README.md 与 AGENTS.md：说明根目录不是 Trellis 项目、真实项目根为 nobelog、正确开工方式与常用命令、红线与收尾约定；解决首次会话在根目录找不到工作流的问题。

## Requirements

- R1 根目录新增 README.md 与 AGENTS.md，两者内容一致：README 面向人，AGENTS 面向 AI 会话。
- R2 解释根目录不是 Trellis 项目、也不是 git 仓库，SessionStart 的未启用提示属于预期；真实项目根为 /root/博客/nobelog。
- R3 给出正确开工步骤：以 nobelog 为会话工作区或先切换目录，加载 trellis-start，读 workflow 与 spec。
- R4 列出常用只读命令、任务创建命令与任务归档位置。
- R5 列出红线：不在根目录 init；不迁移或删除 .trellis；不解压 .backups 覆盖；提交需要 [task:] 锚点与 directive.md；推送与读取工作区外路径需要用户授权。
- R6 内容必须与仓库事实一致，由团队成员只读复核。
- R7 不改动 nobelog 业务代码；指引文件位于工作区根目录，不受 git 版本控制。

## Acceptance Criteria

- [ ] README.md 与 AGENTS.md 存在于 /root/博客 根目录且内容完整。
- [ ] 指引中的路径、版本、命令经团队成员只读复核，无严重事实错误。
- [ ] 指引能回答首次会话为何找不到工作流以及如何继续。
- [ ] 任务归档并记录 journal，提交带任务锚点。

## 背景事实

- 2026-10-30 首轮会话：SessionStart 报告根目录未启用 Trellis；扫描后在 nobelog/.trellis 找到工作流，用户选择在 nobelog 项目内继续。
- 放置评估结论：.trellis 维持 nobelog 项目内，不迁主工作区。

## Notes

- 轻量任务，PRD-only。
- 团队成员校验任务为 team task-3。
