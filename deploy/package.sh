#!/usr/bin/env bash
# 生成免编译部署包：dist/nobelog-<version>-linux-x86_64.tar.gz
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${VERSION:-$(git describe --tags --always --dirty | sed 's/^v//')}"
NAME="nobelog-$VERSION-linux-x86_64"
OUT="${OUT:-dist}"

/root/.cargo/bin/cargo build --release --locked --manifest-path rust/Cargo.toml

stage="$(mktemp -d)/$NAME"
mkdir -p "$stage/rust/client" "$stage/deploy"
install -m 755 rust/target/release/nobelog "$stage/nobelog"
cp -a content vendor "$stage/"
install -m 644 rust/client/main.js "$stage/rust/client/main.js"
install -m 755 deploy/install.sh "$stage/install.sh"
install -m 644 deploy/README-DEPLOY.md "$stage/README-DEPLOY.md"
cp deploy/nobelog.service deploy/nginx.conf.example "$stage/deploy/"
cp README.md LICENSE "$stage/"
printf '%s\n' "$VERSION" > "$stage/VERSION"

mkdir -p "$OUT"
tar -C "$(dirname "$stage")" -czf "$OUT/$NAME.tar.gz" "$NAME"
(cd "$OUT" && sha256sum "$NAME.tar.gz" > SHA256SUMS)
echo "$OUT/$NAME.tar.gz"
