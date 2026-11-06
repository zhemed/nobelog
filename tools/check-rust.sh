#!/usr/bin/env bash
# 校验 Rust 工具链与仓库根 rust-toolchain.toml 一致（当前固定 1.99.1）。
#
# 为什么要这个脚本：rust-toolchain.toml 只有在 rustup 的 shim 下才生效。
# 若新机器用发行版包（apt install cargo rustc）装 Rust，版本会被悄悄换成
# 系统自带的旧版本，且 rustup 不会介入 —— 这里提前拦下并给出正确做法。
set -euo pipefail
cd "$(dirname "$0")/.."

want="$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)"
if [ -z "$want" ]; then
  echo "错误：无法从 rust-toolchain.toml 读取 channel。" >&2
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  cat >&2 <<MSG
错误：未找到 rustup。rust-toolchain.toml 只在 rustup 下生效，
      用发行版自带的 cargo/rustc 会忽略它，可能装到别的版本。

修复（新机器首次安装，工具链装在该用户 HOME 下，不污染系统）：
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  然后重新进入本目录，执行任意 cargo 命令会自动按 rust-toolchain.toml 装 ${want}
MSG
  exit 1
fi

have_rustc="$(rustc --version 2>/dev/null | awk '{print $2}' || true)"
have_cargo="$(cargo --version 2>/dev/null | awk '{print $2}' || true)"

if [ "$have_rustc" != "$want" ] || [ "$have_cargo" != "$want" ]; then
  cat >&2 <<MSG
错误：当前工具链与 rust-toolchain.toml 不一致。
  rust-toolchain.toml 要求 : ${want}
  实际 rustc              : ${have_rustc:-未找到}
  实际 cargo              : ${have_cargo:-未找到}

修复：
  rustup toolchain install ${want} --profile minimal --component rustfmt --component clippy
  （在仓库内执行时 rustup 会自动选用 ${want}，无需手动 set）
MSG
  exit 1
fi

echo "Rust 工具链 OK：rustc ${have_rustc} / cargo ${have_cargo}"
