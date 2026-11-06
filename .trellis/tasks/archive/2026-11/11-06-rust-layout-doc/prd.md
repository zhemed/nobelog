# 写清工具链安装布局

## 背景

用户发现本机 rpm 在系统目录，担心「装错位置/装错版本」。核实后：

- 入口是**系统级**：`/opt/rust/cargo/bin/*`（rustup shim），再软链到 `/usr/local/bin/{cargo,rustc,rustup}`；
- 工具链本体按用户存放：`RUSTUP_HOME` 未设置 → 解析为 `$HOME/.rustup`，root 的在 `/root/.rustup/toolchains/1.99.1-…`；
- 换用户（模拟 `HOME=/tmp/nobodyhome`）首次在仓库跑 `cargo`，会读 `rust-toolchain.toml` 并**自动下载 1.99.1** 到该用户家目录。

## 改动

README「工具链」小节补一张对照表（每用户 vs 系统级入口），并说明「入口可共享、工具链按用户」；
`.trellis/spec/frontend/build-serve-deploy.md` 同步一句。

## 验证

- `rustup show` → rustup home `/root/.rustup`，工具链 `1.99.1-x86_64-unknown-linux-gnu`；
- `env -u RUSTUP_HOME -u CARGO_HOME HOME=/tmp/nobodyhome cargo --version` → 触发 1.99.1 自动下载（证明按用户安装且读 toolchain 文件）。
