#!/usr/bin/env bash
# shellcheck disable=SC2029,SC2086
# 一键部署：本地构建二进制 → 同步 → 重启 systemd 服务。
#
# 两种模式：
#   DEPLOY_MODE=dynamic（默认）—— VPS 运行时渲染，只同步 binary + content + vendor + client
#   DEPLOY_MODE=static         —— 同步 binary + public + content（需要先在本地 pnpm build）
#
# 用法：
#   DEPLOY_HOST=root@1.2.3.4 ./deploy/deploy.sh
#   DEPLOY_HOST=deploy@vps DEPLOY_MODE=static ./deploy/deploy.sh
set -euo pipefail

HOST="${DEPLOY_HOST:?请设置 DEPLOY_HOST，例如 DEPLOY_HOST=root@1.2.3.4}"
APP_DIR="${DEPLOY_DIR:-/opt/nobelog}"
SERVICE="${DEPLOY_SERVICE:-nobelog}"
PORT="${DEPLOY_PORT:-10086}"
MODE="${DEPLOY_MODE:-dynamic}"
SSH_OPTS="${DEPLOY_SSH_OPTS:-}"

cd "$(dirname "$0")/.."

case "$MODE" in
  dynamic|static) ;;
  *) echo "DEPLOY_MODE 只能是 dynamic 或 static"; exit 1 ;;
esac

echo "[1/4] 编译 release 二进制"
cargo build --release --manifest-path rust/Cargo.toml
BIN=rust/target/release/nobelog
[ -x "$BIN" ] || { echo "找不到 $BIN"; exit 1; }

if [ "$MODE" = "static" ]; then
  echo "[2/4] 静态产物构建"
  pnpm build
fi

STAMP="$(date +%Y%m%d-%H%M%S)"
echo "[3/4] 同步到 $HOST:$APP_DIR（模式 $MODE）"
ssh $SSH_OPTS "$HOST" "mkdir -p '$APP_DIR/public' '$APP_DIR/content' '$APP_DIR/vendor' '$APP_DIR/rust/client'"
rsync -az --delete $SSH_OPTS "$PWD/content/" "$HOST:$APP_DIR/content/"
if [ "$MODE" = "dynamic" ]; then
  rsync -az --delete $SSH_OPTS "$PWD/vendor/" "$HOST:$APP_DIR/vendor/"
  rsync -az $SSH_OPTS "$PWD/rust/client/main.js" "$HOST:$APP_DIR/rust/client/main.js"
else
  rsync -az --delete $SSH_OPTS "$PWD/public/" "$HOST:$APP_DIR/public/"
fi
rsync -az $SSH_OPTS "$BIN" "$HOST:$APP_DIR/nobelog.new"
ssh $SSH_OPTS "$HOST" "mv '$APP_DIR/nobelog.new' '$APP_DIR/nobelog' && chmod 755 '$APP_DIR/nobelog'"

echo "[4/4] 重启服务并健康检查"
# systemd 单元里的 --dynamic 与 MODE 不一致时，用 drop-in 覆盖启动参数
if [ "$MODE" = "dynamic" ]; then
  ssh $SSH_OPTS "$HOST" "sudo systemctl set-environment NOBELOG_FLAGS=--dynamic"
else
  ssh $SSH_OPTS "$HOST" "sudo systemctl set-environment NOBELOG_FLAGS="
fi
ssh $SSH_OPTS "$HOST" "sudo systemctl restart '$SERVICE'"
ssh $SSH_OPTS "$HOST" "for i in \$(seq 1 40); do curl -fsS -o /dev/null http://127.0.0.1:$PORT/ && break; sleep 0.5; done && curl -fsS -o /dev/null http://127.0.0.1:$PORT/"

echo "✓ 部署完成：$HOST:$APP_DIR（模式 $MODE，版本 $STAMP）"
