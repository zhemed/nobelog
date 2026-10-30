#!/usr/bin/env bash
# 生成发布包：dist/nobelog-<version>-linux-x86_64.tar.gz
# 只放运行所需文件，不含安装脚本；部署方式由使用者决定。
#
# 包内的 public/ 是打包时用 SITE_URL 重建的静态产物：canonical、og:url、RSS、sitemap、
# robots 里的绝对地址全部来自这个值，因此必须显式给正式 https 域名，不允许空值、
# http、本机地址或已下线域名。VERSION 必须与 rust/Cargo.toml 的版本一致，避免
# 包名与二进制 --version 脱节。
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${VERSION:-$(git describe --tags --always --dirty | sed 's/^v//')}"
NAME="nobelog-$VERSION-linux-x86_64"
OUT="${OUT:-dist}"

SITE_URL="${SITE_URL:-}"
if [ -z "$SITE_URL" ]; then
  echo "错误：必须显式设置 SITE_URL，例如 SITE_URL=https://blog.example.com VERSION=$VERSION $0" >&2
  echo "      包内静态产物的 canonical / RSS / sitemap / robots 会使用该值。" >&2
  exit 1
fi
case "$SITE_URL" in
  https://*) ;;
  *)
    echo "错误：SITE_URL 必须以 https:// 开头，当前为：$SITE_URL" >&2
    exit 1
    ;;
esac
if [ -z "${SITE_URL#https://}" ]; then
  echo "错误：SITE_URL 缺少主机名，当前为：$SITE_URL" >&2
  exit 1
fi
case "$SITE_URL" in
  *example.com*|*127.0.0.1*)
    echo "错误：SITE_URL 不允许使用已下线域名或本机地址，当前为：$SITE_URL" >&2
    exit 1
    ;;
esac
# 与 env_url() 一致：去掉尾部斜杠，避免包内出现 https://host// 形式的绝对地址。
SITE_URL="${SITE_URL%/}"
export SITE_URL

cargo_version="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' rust/Cargo.toml | head -n 1)"
if [ -z "$cargo_version" ]; then
  echo "错误：无法从 rust/Cargo.toml 读取版本号" >&2
  exit 1
fi
if [ "$VERSION" != "$cargo_version" ]; then
  echo "错误：VERSION=$VERSION 与 rust/Cargo.toml 的版本 $cargo_version 不一致" >&2
  echo "      请先把 rust/Cargo.toml 与 rust/Cargo.lock 的版本同步，或显式传 VERSION=$cargo_version。" >&2
  exit 1
fi
echo "打包版本：$VERSION"
echo "SITE_URL：$SITE_URL"

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
