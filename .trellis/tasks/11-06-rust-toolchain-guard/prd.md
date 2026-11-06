# 新机器工具链自检与文档

## 背景

在新服务器 clone 后，`rust-toolchain.toml` 只有在 **rustup shim** 下才生效：
用发行版包（`apt install cargo rustc`）装的 Rust 会无视它，静默用旧版本。
另外 `tools/package.sh` 里写死了 `/root/.cargo/bin/cargo`，别的用户/机器路径不同会直接失败。

## 改动

| 文件 | 改动 |
| --- | --- |
| `tools/check-rust.sh`（新增） | 读 `rust-toolchain.toml` 的 channel，对比 `rustc/cargo --version`；无 rustup 或版本不符时报错并给出修复命令 |
| `tools/package.sh` | 开头调用 `./tools/check-rust.sh`；`/root/.cargo/bin/cargo` → `cargo`（走 PATH） |
| `README.md` | 新增「工具链」小节：rustup 安装命令、为什么不能用发行版包、自检脚本 |
| `.trellis/spec/frontend/build-serve-deploy.md` | 补自检脚本与 package.sh 前置检查的说明 |

## 验证

- `./tools/check-rust.sh` → `Rust 工具链 OK：rustc 1.99.1 / cargo 1.99.1`；
- 模拟无 rustup 环境（`env -i PATH=/usr/bin:/bin`）→ 退出码 1 且输出安装指引；
- `VERSION=0.0.36 OUT=/tmp/pkgtest tools/package.sh` → 自检先通过，随后正常进入打包流程。
