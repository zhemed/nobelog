# 本机部署 v0.0.23 并重启调试

## Goal

把本机 `nobelog-local.service` 从 v0.0.22 进程切到已构建的 v0.0.23 二进制，重启一次并完成本机调试冒烟。

## 现状

- 磁盘 `rust/target/release/nobelog` 已是 v0.0.23（sha256 `a7e257c2…`，02:53 构建）。
- 服务进程 185210（02:30:54 启动）仍在跑旧 inode（v0.0.22）。
- 本机以 `--dynamic` 运行在 10086，`SITE_URL` 未设置（启动告警，canonical 用请求 Host）。

## Requirements

- R1 `cargo build --release --locked`（确认二进制与 v0.0.23 包内一致）。
- R2 `systemctl restart nobelog-local` 一次；确认 active、MainPID 变化、`/proc/<pid>/exe` sha256 = `a7e257c2…`。
- R3 本机调试冒烟：9 条路由 + 404；feed pubDate 仍 `14:40/04:15 GMT`；首页 CSS `main.1f82b0f7.css`；
  `/images/*` 为 `no-cache`（动态模式应返回 no-cache）；XFH 恶意头回退。
- R4 说明：`robots.txt` 是构建期产物，动态模式不提供该路由（本机 `/robots.txt` 仍 404 属预期）；
  如需本机预览 robots，用 `nobelog build` 产物或静态模式。
- R5 不部署远程、不改 unit 配置、不写 SITE_URL。

## Acceptance Criteria

- [ ] 服务 active，PID 变化，运行二进制 = v0.0.23。
- [ ] 冒烟 9 路由 + 404 全绿，feed/CSS 修复项未回退，`/images` no-cache。
- [ ] 记录 PID/时间、命令输出；Trellis journal + 归档。
