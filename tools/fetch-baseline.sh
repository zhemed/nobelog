#!/usr/bin/env bash
# 从 GitHub Release 还原 public/ 冻结基线，让全新 clone 也能复现 `nobelog diff`。
#
# 用法：tools/fetch-baseline.sh [tag]
#   - 不传 tag：查询 GitHub 最新 release（需要 python3 解析 API JSON）。
#   - 传 tag：如 v0.0.37；也可用环境变量 NOBELOG_BASELINE_TAG 指定默认 tag。
#   - NOBELOG_REPO 覆盖仓库，默认 zhemed/nobelog。
#
# 行为：下载 nobelog-<version>-linux-x86_64.tar.gz 与 SHA256SUMS 到临时目录，
# 校验 sha256 与包内 VERSION 后，用包内 public/ 覆盖仓库根的 public/。
# 只写 public/（gitignored 构建产物，是 diff 的冻结基准，覆盖是预期行为），
# 不碰源码、不执行任何 git 操作。
set -euo pipefail
cd "$(dirname "$0")/.."

REPO="${NOBELOG_REPO:-zhemed/nobelog}"

die() {
  echo "错误：$*" >&2
  exit 1
}

usage() {
  cat <<'EOF'
用法：tools/fetch-baseline.sh [tag]

从 GitHub Release 还原 public/ 冻结基线（diff 基准）。

  tag                    release tag，如 v0.0.37；省略则取 GitHub 最新 release
  NOBELOG_REPO           覆盖仓库，默认 zhemed/nobelog
  NOBELOG_BASELINE_TAG   省略 tag 参数时的默认 tag（固定版本/离线场景）

只写仓库根的 public/，不执行 git 操作。
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac
if [ "$#" -gt 1 ]; then
  die "参数过多：最多接受一个 tag（当前：$*）"
fi

TAG="${1:-${NOBELOG_BASELINE_TAG:-}}"

for tool in curl tar sha256sum; do
  command -v "$tool" >/dev/null 2>&1 \
    || die "缺少依赖命令 $tool（Debian/Ubuntu 可 apt install $tool；macOS 的 sha256sum 在 coreutils 里）"
done

if [ -z "$TAG" ]; then
  command -v python3 >/dev/null 2>&1 \
    || die "未指定 tag 且缺少 python3（用于解析 GitHub API JSON）；可显式传 tag：$0 v0.0.37"
  echo "未指定 tag，查询 GitHub 最新 release：$REPO" >&2
  api="https://api.github.com/repos/$REPO/releases/latest"
  json="$(curl -fsSL --retry 3 -H 'Accept: application/vnd.github+json' "$api")" \
    || die "查询最新 release 失败：$api（检查网络；或显式指定 tag：$0 v0.0.37）"
  TAG="$(printf '%s' "$json" | python3 -c 'import json, sys; print(json.load(sys.stdin).get("tag_name") or "")')" \
    || die "解析 GitHub API 响应失败：$api"
  [ -n "$TAG" ] || die "GitHub API 未返回 tag_name：$api；可显式指定 tag：$0 v0.0.37"
fi

case "$TAG" in
  v[0-9]*) VER="${TAG#v}" ;;
  [0-9]*) VER="$TAG" ;;
  *) die "tag 格式不认识：$TAG（期望形如 v0.0.37）" ;;
esac

ASSET="nobelog-${VER}-linux-x86_64.tar.gz"
BASE_URL="https://github.com/$REPO/releases/download/$TAG"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "基线 tag：$TAG（仓库 $REPO，资产 $ASSET）"
echo "下载：$BASE_URL/$ASSET"
curl -fsSL --retry 3 -o "$TMP/$ASSET" "$BASE_URL/$ASSET" \
  || die "下载 $ASSET 失败：$BASE_URL/$ASSET（tag/release 或资产名是否存在？）"
curl -fsSL --retry 3 -o "$TMP/SHA256SUMS" "$BASE_URL/SHA256SUMS" \
  || die "下载 SHA256SUMS 失败：$BASE_URL/SHA256SUMS"

# SHA256SUMS 每行 "<hash>  <file>"（兼容二进制模式的 "*file"）。
awk -v f="$ASSET" '{ name = $2; sub(/^\*/, "", name); if (name == f) found = 1 } END { exit !found }' \
  "$TMP/SHA256SUMS" \
  || die "SHA256SUMS 中找不到 $ASSET（发布资产命名变了？）"
( cd "$TMP" \
  && awk -v f="$ASSET" '{ name = $2; sub(/^\*/, "", name); if (name == f) print }' SHA256SUMS \
    | sha256sum -c - ) \
  || die "$ASSET 的 sha256 校验失败"

mkdir -p "$TMP/unpack"
tar -xzf "$TMP/$ASSET" -C "$TMP/unpack" || die "解包 $ASSET 失败"
PKG="$TMP/unpack/nobelog-${VER}-linux-x86_64"
[ -d "$PKG/public" ] || die "包内缺少 public/：$PKG/public（发布包布局变了？）"
[ -f "$PKG/VERSION" ] || die "包内缺少 VERSION：$PKG/VERSION"
PKG_VER="$(tr -d '[:space:]' < "$PKG/VERSION")"
[ "$PKG_VER" = "$VER" ] || die "包内 VERSION=$PKG_VER 与 tag 推导版本 $VER 不一致"

TARGET="$(pwd)/public"
echo "覆盖冻结基线：$TARGET"
rm -rf "$TARGET"
cp -a "$PKG/public" "$TARGET"
COUNT="$(find "$TARGET" -type f | wc -l | tr -d '[:space:]')"
echo "已从 $TAG 还原 $COUNT 个文件到 public/"
echo "下一步：SITE_URL=https://example.com ./rust/target/release/nobelog diff public rust/dist"
