# 工作区残留盘点证据

取证时间：2026-10-30 23:51 Asia/Shanghai，UTC 15:51。取证方式：nobelog 仓库内只读命令与文件读取；未改动任何被审计对象。

## 1 结论

- 活跃树没有历史残留任务：.trellis/tasks 下非 archive 目录只有本轮审计任务 10-30-audit-residual-tasks。
- 归档任务 44 个，全部 status=completed 且 completedAt 齐全。
- 工作树仅本轮任务目录未跟踪；git stash 为空；main 与 origin/main 同步，ahead/behind 为 0/0，HEAD 为 ef464fd。
- /root/博客 根目录只有 nobelog、.backups、AGENTS.md、README.md；根目录没有 .trellis，也没有 .git，属预期设计。
- .backups 内 5 个 tar.gz 校验 5/5 通过；其中 3 个含 2026-10-17 旧世代 .trellis 快照与未归档任务，但只存在于压缩包内，不参与当前工作流。
- workspace journal 共 44 个 Session，Last Active 为 2026-10-30；最近 Next Steps 记录的业务待决项属于用户决策事项，不是任务残留。
- 运行期残留为 .trellis/.runtime 的 2 个 update-check marker 与 16 个 .pyc 缓存，均被 .trellis/.gitignore 忽略，不影响任务状态。

## 2 证据

### 2.1 活跃与归档任务

```bash
find .trellis/tasks -mindepth 1 -maxdepth 1 -type d
# .trellis/tasks/archive
# .trellis/tasks/10-30-audit-residual-tasks

find .trellis/tasks/archive -name task.json | wc -l
# 44
find .trellis/tasks/archive -name task.json -exec python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));print(d.get("status"), bool(d.get("completedAt")))' {} \; | sort | uniq -c
# 44 completed True
```

- 归档目录 2026-10 共 44 个任务，task.py list-archive 显示 2026-10 共 44 个。
- 与上一轮盘点相比增加 2 个：10-30-trellis-residue-placement、10-30-workspace-trellis-guide，均为 completed。

### 2.2 Git 与仓库

```bash
git rev-parse --short HEAD          # ef464fd
git status --short                  # ?? .trellis/tasks/10-30-audit-residual-tasks/
git stash list | wc -l              # 0
git rev-list --left-right --count origin/main...HEAD   # 0	0
```

- 工作树没有业务文件改动；唯一未跟踪项是本轮审计任务目录，收尾后会归档并产生流程提交。
- 无遗留分支与 worktree：仅 main 与 remotes/origin/main，worktree 只有 /root/博客/nobelog。

### 2.3 根目录与备份

```bash
ls -1 /root/博客                   # AGENTS.md README.md nobelog；另有隐藏 .backups
ls -d /root/博客/.trellis /root/博客/.git   # 均不存在
du -sh /root/博客/.backups         # 11M
cd /root/博客/.backups && sha256sum -c SHA256SUMS   # 5/5 OK
```

旧世代快照明细，只读 tar 列表与 task.json 读取，未解压覆盖：

| 备份包 | 内含未归档任务 | 状态 | 说明 |
|--------|----------------|------|------|
| nobelog-src-20261017-020319.tar.gz | 10-17-freeze-backup、00-bootstrap-guidelines | in_progress | 旧世代任务在 tasks 根而非 archive |
| nobelog-rs-src-20261017-030023.tar.gz | 10-17-rust-default、10-17-rust-toolchain-pin、00-bootstrap-guidelines | planning、planning、in_progress | 旧世代 Rust 化前快照 |
| nobelog-pre-node-removal-20261017-032548.tar.gz | 10-17-rust-toolchain-pin、00-bootstrap-guidelines | planning、in_progress | 去 Node 化前快照 |
| nobelog-build-20261017-020319.tar.gz | 无 | 无 | 仅构建产物 |
| public-node-baseline-20261017-030023.tar.gz | 无 | 无 | 仅 public 基线 |

- 3 个含旧 .trellis 的备份时间均为 2026-10-17，早于当前世代启用与归档记录；当前 archive 中的同名 00-bootstrap-guidelines 已是 completed 的另一世代。
- 复核补充：3 个含 .trellis 的包内非归档 task.json 共 7 个目录快照、4 个唯一任务 id，状态为 in_progress 2 个、planning 2 个，创建日期 2026-10-16 与 2026-10-17，completedAt 均为 null。
- 复核补充：nobelog-build-20261017-020319.tar.gz 与 public-node-baseline-20261017-030023.tar.gz 的 sha256 完全相同，属同内容冗余备份。
- 历史决策见 10-30-trellis-residue-placement/research/audit-findings.md：旧备份保留、禁止解压覆盖；如需清理或迁移另立任务并授权。

### 2.4 Journal 与待决事项

```bash
grep -n 'Last Active\|Total Sessions' .trellis/workspace/zhemed/index.md
# Total Sessions: 44；Last Active: 2026-10-30
grep -c '^## Session ' .trellis/workspace/zhemed/journal-1.md
# 44
wc -l .trellis/workspace/zhemed/journal-1.md
# 1545
```

journal 最近会话 Next Steps 中的待决项，按用户决策事项归类，不是任务残留：

- Session 42：systemd 未注入 SITE_URL，本机 canonical 仍为 http://127.0.0.1:10086/；公网反代需设置 SITE_URL 或 NOBELOG_TRUST_FORWARDED=1 并透传 Host；私网内可伪造 scheme 属已记录的安全取舍；2.3.4 若为外部需求条目需用户提供原文。
- Session 43：后续会话工作区使用 /root/博客/nobelog；如需处理旧备份中的未闭环任务，另立任务并授权。
- Session 44：后续会话优先以 /root/博客/nobelog 作为工作区；根目录指引需更新时直接编辑 README.md 与 AGENTS.md。

- 复核补充：.trellis/workspace/index.md 主索引仍为模板空表，Active Developers 显示 none yet，与 zhemed/index.md 不一致；属文档一致性小项，不是任务残留。
- 复核补充：归档区 00-bootstrap-guidelines 缺 implement.jsonl 与 check.jsonl；归档区共 76 个零字节 jsonl 模板文件；均为历史模板产物，不影响状态。

### 2.5 运行期残留

```bash
find .trellis/.runtime -maxdepth 2 -name '*.marker' -o -name sessions   # 2 个 marker 与 sessions 目录
find .trellis/scripts -name '*.pyc' | wc -l                            # 16
git check-ignore -v .trellis/.runtime/sessions .trellis/scripts/common/__pycache__
# .trellis/.gitignore:8:.runtime/
# .trellis/.gitignore:31:**/__pycache__/
```

- 均为 git 忽略项；2 个 update-check marker 时间为 13:46 与 14:52，来自往次会议，当前会话尚无 marker。
- .trellis/.runtime/sessions 下 1 个 session json，current_task 指向本审计任务；无 .current-task、无 .plan-log、无 *.tmp 或 *.new、无符号链接。
- 如需清理属可选维护动作，需授权。

## 3 独立复核

复核员 r3-verifier，团队任务 task-2；取证快照 2026-10-30 15:49–15:55 UTC；全程只读，未写改删文件，未联网。

Lead 结论：接受。复核员用独立命令复现全部关键数字，并确认归档 44/44 完整、225 个归档文件全部纳入 git、2026-10-19 以来 77 个提交均带 task 锚点。

- 确认项：活跃树仅本审计任务；归档 44 个全部 completed 且 completedAt 齐全；工作树 6 个未跟踪文件都在本任务目录；stash 为空；仅 main 分支、无未合并分支、无额外 worktree；根目录四项且无 .trellis 与 .git；备份 5/5 校验通过；旧世代任务只在压缩包内；journal 44 会话与 Last Active 一致；运行期残留均被忽略。
- 修正项：远端同步仅在本地引用层面确认，本轮未联网 fetch，远端实时状态未验证；主 workspace/index.md 仍为模板空表；归档区 00-bootstrap-guidelines 缺两份 jsonl 清单；运行期 marker 来自往次会议而非当前会话；两个备份 sha256 相同；证据文件 §2.1 原为占位命令，已在本轮补为可复跑命令。

## 4 建议

- 维持现状：活跃树没有需要处理的任务残留，无需清理或迁移。
- 旧备份保持原样，禁止解压覆盖当前 .trellis；如确需清理旧世代任务目录，另立任务并取得当轮授权。
- journal 中的业务待决项按用户决策事项跟踪，需要时可立项处理，不计入任务残留。
- 运行期残留可选清理，不影响任务状态；主 workspace/index.md 模板可在后续会话顺手回填。
- 如需严格确认远端实时同步，取得授权后 git fetch 再比对。
