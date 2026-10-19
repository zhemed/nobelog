# Journal - zhemed (Part 1)

> AI development session journal
> Started: 2026-10-19

---



## Session 1: 确认无残留任务并启用 Trellis
<!-- trellis-session: v=2 fp=2fc6fa8afd2b0c6e -->

**Date**: 2026-10-19
**Task**: 确认无残留任务并启用 Trellis
**Branch**: `main`

### Summary

核实 nobelog 工作区干净、无游离进程；按用户确认在仓库内执行 trellis init --dsh -u zhemed，提交 .trellis/AGENTS.md/.agents/.dsh

### Main Changes

- 启用 Trellis 工作流

### Git Commits

| Hash | Message |
|------|---------|
| `30c82fa` | chore: 启用 Trellis 工作流（.trellis/ 与 DSH skills） [task:10-19-enable-trellis] |

### Testing

- [OK] git status 干净；nobelog-local.service 运行正常

### Status

[OK] **Completed**

### Next Steps

- 可选：处理 00-bootstrap-guidelines 填充 spec


## Session 2: 填充 .trellis/spec/frontend
<!-- trellis-session: v=2 fp=1a0be6a9380b0b25 -->

**Date**: 2026-10-19
**Task**: 填充 .trellis/spec/frontend
**Branch**: `main`

### Summary

按 Rust 静态站真实结构重写 frontend 指南：9 个文件，删除 React 类模板；记录 public/ 冻结基准、逐字节 diff、UTC 日期语义、CSS 裁剪等约束

### Main Changes

- .trellis/spec/frontend/*

### Git Commits

| Hash | Message |
|------|---------|
| `f4c1099` | docs: 填充 .trellis/spec/frontend（按 Rust 静态站真实结构） [task:00-bootstrap-guidelines] |

### Testing

- [OK] grep 校验无占位符；index 与文件集一致

### Status

[OK] **Completed**

### Next Steps

- 按 trellis-update-spec 增量维护


## Session 3: 博客全面审查（安全/渲染/构建/线上）
<!-- trellis-session: v=2 fp=4eb9be48b3ce8261 -->

**Date**: 2026-10-19
**Task**: 博客全面审查（安全/渲染/构建/线上）
**Branch**: `main`

### Summary

三路只读审查：无 P0；P1 两条（frontmatter slug 构建期路径穿越、动态模式 Host/X-Forwarded-* 注入且 base_url 未转义）；渲染侧 P1（RSS pubDate 时区偏移 8h）；构建侧 P2（裁剪依赖 public/ 导致两套 CSS）

### Main Changes

- 无（纯审查）

### Git Commits

(No commits - planning session)

### Testing

- [OK] cargo test 30 通过；clippy 0 warning；线上 8 路由 + 404 冒烟全绿；diff 冻结产物仅时间敏感差异

### Status

[OK] **Completed**

### Next Steps

- 按用户选择修复 P1 并补回归测试


## Session 4: 修复审查发现的 P1 与日期回归
<!-- trellis-session: v=2 fp=0b615bb4636d3379 -->

**Date**: 2026-10-19
**Task**: 修复审查发现的 P1 与日期回归
**Branch**: `main`

### Summary

slug/标签/分类路径穿越双层拦截、Host/XFH 白名单 + base_url 转义、RSS pubDate 时区；复审打回两轮后补 mtime 归一与空格偏移容错，52 测试全绿，未提交未部署

### Main Changes

- rust/src/{content,build,serve,templates,site}.rs

### Git Commits

(No commits - planning session)

### Testing

- [OK] cargo fmt --check / clippy --all-targets / test(52) 全绿；debug 构建 PoC 与注入实测通过；整站归一化 diff 仅 feed+index+CSS 哈希

### Status

[OK] **Completed**

### Next Steps

- 用户决定提交/部署；剩余 P2（CSS 裁剪依赖 public、动态热更新删除检测等）待排期


## Session 5: 同步 spec 并部署 P1 修复
<!-- trellis-session: v=2 fp=603e5aa57300dbb2 -->

**Date**: 2026-10-19
**Task**: 同步 spec 并部署 P1 修复
**Branch**: `main`

### Summary

spec 时区约定改为 UTC+8 挂钟存储/RSS 输出真实 UTC；提交并推送 9ad2ee3；release 重建后重启 nobelog-local（PID 830→63373），feed pubDate 与注入头实测均符合预期

### Main Changes

- .trellis/spec/frontend/*

### Git Commits

| Hash | Message |
|------|---------|
| `9ad2ee3` | fix: 修复构建期路径穿越、Host 注入与 RSS 时区（P1） [task:10-19-fix-p1-audit] |

### Testing

- [OK] 线上 8 路由 200/404；feed pubDate 14:40/04:15 GMT；XFH 注入回退；52 测试全绿

### Status

[OK] **Completed**

### Next Steps

- 剩余 P2：CSS 裁剪依赖 public/、热更新删除检测、SITE_URL、panic=unwind
