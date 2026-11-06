# 锁定 Rust 1.99.1

## Goal

项目固定使用 `rustc 1.99.1 (58cf8f9c3 2026-10-13)` / `cargo 1.99.1 (5f94df478 2026-08-27)`，非特殊原因不再变动。

## 改动

| 文件 | 改动 |
| --- | --- |
| `rust-toolchain.toml` | 从 `rust/` 移到仓库根（`git mv`），内容不变：channel 1.99.1 + rustfmt/clippy + minimal |
| `rust/Cargo.toml` | `rust-version` 由 `1.99` 收紧为 `1.99.1` |
| `.trellis/spec/frontend/build-serve-deploy.md` | 路径与 MSRV 描述同步 |

## 验证

- 仓库根执行 `cargo --version` → **cargo 1.99.1**（根目录 toolchain 文件生效）；
- `cd rust && cargo fmt --check / clippy --all-targets --locked / test --locked` → 全部通过（98 tests）；
- `rustup show` 仅安装 `1.99.1-x86_64-unknown-linux-gnu`。
