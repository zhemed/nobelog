# 指引独立校验记录

日期：2026-10-30；校验人：团队成员 guide-verifier；方式：只读核对 /root/博客 与 nobelog 仓库，运行只读命令。

## 校验结论

- 指引能解决根目录会话找不到工作流的问题：根目录 AGENTS.md 会被 DSH 作为工作区指令自动加载，README 提供完整细节。
- 命令可执行：get_context.py、--mode phase、--mode packages、task.py current/list/list-archive 均只读运行通过；task.py create 参数与 --help 一致。
- 事实基本吻合：根目录无 .git 与 .trellis；nobelog 为唯一 git 仓库；.trellis 结构、开发者 zhemed、分支 main、journal 与 index 路径、.backups 风险、提交锚点与 directive.md 约定均属实。

## 发现与处理

- 确定错误：README 原写「根目录内容：仅 nobelog/ 与 .backups/」，未计入指引自身文件。已改为「除本指引 README.md 与 AGENTS.md 外，仅含 nobelog/ 与 .backups/」。
- 建议改进：补充 SessionStart 应答口径、AGENTS 自动加载说明、项目实际 spec 索引路径、task.py current 无任务时的退出码说明、journal 通配路径、根目录 init 的主因表述、收尾具体命令与 trellis-finish-work 技能名；以上已全部落入 README 与 AGENTS 修订。

## 未验证项

- nobelog 作为 DSH 工作区打开后 SessionStart 是否不再提示，未实测。
- 全局钩子实现与根目录 init 的真实后果在工作区外，未读取或执行。
- 推送与读取工作区外路径的授权要求来自全局规则，工作区内无独立证据。
