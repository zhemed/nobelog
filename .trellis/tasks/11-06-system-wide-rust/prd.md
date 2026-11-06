# Rust 改为系统级共享安装

## 背景

本机 `/usr/local/bin/{cargo,rustc,rustup…}` 原是指向 `/opt/rust/cargo/bin/*` 的软链，
但 `RUSTUP_HOME` 未设置 → 实际落在各用户 `$HOME/.rustup`，工具链并非全机共享。

## 改动（机器级，不入库）

1. `/usr/local/bin` 下 14 个 rust 入口改为**包装脚本**：先把 `RUSTUP_HOME` 固定为 `/opt/rust/rustup`，再 `exec /opt/rust/cargo/bin/<name>`；
2. 新增 `/etc/profile.d/rust.sh`：登录 shell 也导出同一个 `RUSTUP_HOME`；
3. 原软链备份在 `/opt/rust/bin-backup-20261106/`。

## 仓库内改动

README「工具链」小节与 `.trellis/spec/frontend/build-serve-deploy.md` 改为描述系统级共享方案。

## 验证

| 场景 | 结果 |
| --- | --- |
| 干净环境 root：`cargo --version` | 1.99.1，`rustup show` 显示 home = `/opt/rust/rustup` |
| 模拟其他用户（空 HOME，无缓存） | 1.99.1，且**未**在其家目录下载工具链 |
| 模拟其他用户 `sh -l` | `RUSTUP_HOME=/opt/rust/rustup`，同样 1.99.1 |
| 仓库内 `fmt` / `clippy` / `test`（98）/ `build --release` | 全部通过 |
