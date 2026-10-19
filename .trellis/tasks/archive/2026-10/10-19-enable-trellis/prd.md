# 确认博客无残留任务并启用 Trellis

## Goal

回答"博客是否还有残留任务"，并在用户确认后于 `nobelog` 仓库内启用 Trellis、把初始化产物提交进仓库。

## 背景（已核实的现状）

- `/root/博客` 与 `/root/博客/nobelog` 此前都没有 `.trellis/`，因此不存在任何 Trellis 任务记录。
- `nobelog` 工作区干净：`main` 与 `origin/main` 同步，无未提交改动、无 stash。
- 无游离进程：10086 端口上的 `nobelog serve` 由 systemd 单元 `nobelog-local.service` 托管，属正常服务。

## Requirements

- R1 以只读方式核实仓库状态（git 工作区、分支同步、stash、残留进程）后再下结论。
- R2 按用户选择，在 `/root/博客/nobelog` 执行 `trellis init --dsh -u zhemed`。
- R3 初始化产物（`.trellis/`、`AGENTS.md`、`.agents/`、`.dsh/`）提交进仓库，提交消息带任务锚点。
- R4 仓库外遗留工作目录**不清理**（用户选择"先保留"）。

## Non-goals

- 不填充 `.trellis/spec/` 的具体规范内容（由 `00-bootstrap-guidelines` 任务承接）。
- 不清理 `.run/`、`.probe-nobelog/`、`.probe-nodeseek/`、`.backups/`、`.ref/`。
- 不推送远程，不改动站点代码与部署。

## Acceptance Criteria

- [x] 明确回答"是否有残留任务"，并给出核实依据（git 状态、进程、`.trellis/` 存在性）。
- [x] `.trellis/` 已在 `nobelog` 仓库内生成，`.trellis/.developer`、`.runtime/` 等本地状态被忽略。
- [x] 初始化产物已提交，提交消息含 `[task:10-19-enable-trellis]` 锚点。
- [ ] 收尾：`task.py finish` + 归档 + journal 留痕。

## Notes

- 全局 git 钩子（`core.hooksPath=/usr/local/share/dsh-git-hooks`）强制要求：提交必须绑定进行中任务，且任务目录内需有 `directive.md` 逐字记录用户当轮原话。
