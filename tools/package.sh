#!/usr/bin/env bash
# 生成发布包：dist/nobelog-<version>-linux-x86_64.tar.gz
# 只放运行所需文件，不含安装脚本；部署方式由使用者决定。
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${VERSION:-$(git describe --tags --always --dirty | sed 's/^v//')}"
NAME="nobelog-$VERSION-linux-x86_64"
OUT="${OUT:-dist}"

/root/.cargo/bin/cargo build --release --locked --manifest-path rust/Cargo.toml
./rust/target/release/nobelog build --self-assets --out public >/dev/null

stage="$(mktemp -d)/$NAME"
mkdir -p "$stage/rust/client"
install -m 755 rust/target/release/nobelog "$stage/nobelog"
cp -a content vendor public "$stage/"
install -m 644 rust/client/main.js "$stage/rust/client/main.js"
cp README.md LICENSE "$stage/"
printf '%s\n' "$VERSION" > "$stage/VERSION"

mkdir -p "$OUT"
tar -C "$(dirname "$stage")" -czf "$OUT/$NAME.tar.gz" "$NAME"
(cd "$OUT" && sha256sum "$NAME.tar.gz" > SHA256SUMS)
echo "$OUT/$NAME.tar.gz"
