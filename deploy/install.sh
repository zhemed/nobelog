#!/usr/bin/env bash
# nobelog 一键安装（dynamic 模式：systemd 常驻 + 本机 127.0.0.1:PORT）
# 用法：
#   sudo DOMAIN=blog.example.com ./install.sh
#   sudo PREFIX=/opt/nobelog PORT=10086 ./install.sh
#   sudo NO_SYSTEMD=1 PREFIX=/tmp/try ./install.sh   # 只解包，不装服务（调试用）
set -euo pipefail

PREFIX="${PREFIX:-/opt/nobelog}"
PORT="${PORT:-10086}"
DOMAIN="${DOMAIN:-}"
USER_NAME="${USER_NAME:-nobelog}"
SERVICE="${SERVICE:-nobelog}"
NO_SYSTEMD="${NO_SYSTEMD:-0}"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m错误:\033[0m %s\n' "$*" >&2; exit 1; }

[ "${1:-}" = "--help" ] && { sed -n '2,9p' "$0"; exit 0; }
[ "$(id -u)" = 0 ] || die "请用 root 运行（sudo）"
command -v systemctl >/dev/null || die "需要 systemd"

SRC="$(cd "$(dirname "$0")" && pwd)"
[ -x "$SRC/nobelog" ] || die "包内找不到 nobelog 二进制"

log "创建用户 $USER_NAME（已存在则跳过）"
id "$USER_NAME" >/dev/null 2>&1 || useradd -r -s /usr/sbin/nologin "$USER_NAME"

log "安装到 $PREFIX"
mkdir -p "$PREFIX/rust/client" "$PREFIX/deploy"
install -m 755 "$SRC/nobelog" "$PREFIX/nobelog"
rm -rf "$PREFIX/content" "$PREFIX/vendor"
cp -a "$SRC/content" "$PREFIX/content"
cp -a "$SRC/vendor" "$PREFIX/vendor"
rm -rf "$PREFIX/public"
if [ -d "$SRC/public" ]; then cp -a "$SRC/public" "$PREFIX/public"; fi
install -m 644 "$SRC/rust/client/main.js" "$PREFIX/rust/client/main.js"
install -m 644 "$SRC/deploy/nginx.conf.example" "$PREFIX/deploy/nginx.conf.example"
chown -R "$USER_NAME:$USER_NAME" "$PREFIX"

if [ "$NO_SYSTEMD" = 1 ]; then
  log "已跳过 systemd（NO_SYSTEMD=1）。手动运行："
  echo "  sudo -u $USER_NAME $PREFIX/nobelog serve --host 127.0.0.1 --port $PORT --root $PREFIX --dynamic"
  exit 0
fi

log "写入 /etc/systemd/system/$SERVICE.service"
{
  echo "[Unit]"
  echo "Description=nobelog blog (dynamic)"
  echo "After=network-online.target"
  echo "Wants=network-online.target"
  echo ""
  echo "[Service]"
  echo "Type=simple"
  echo "User=$USER_NAME"
  echo "WorkingDirectory=$PREFIX"
  [ -n "$DOMAIN" ] && echo "Environment=SITE_URL=https://$DOMAIN"
  echo "ExecStart=$PREFIX/nobelog serve --host 127.0.0.1 --port $PORT --root $PREFIX --dynamic"
  echo "Restart=always"
  echo "RestartSec=2"
  echo "NoNewPrivileges=true"
  echo "PrivateTmp=true"
  echo "ProtectSystem=strict"
  echo "ProtectHome=true"
  echo "ReadOnlyPaths=$PREFIX"
  echo ""
  echo "[Install]"
  echo "WantedBy=multi-user.target"
} > "/etc/systemd/system/$SERVICE.service"

systemctl daemon-reload
systemctl enable --now "$SERVICE"
sleep 1

log "健康检查"
curl -fsS -o /dev/null "http://127.0.0.1:$PORT/" || die "服务没起来，看 journalctl -u $SERVICE -n 50"
log "本机服务正常：http://127.0.0.1:$PORT/"

cat <<NEXT

下一步（对外提供访问，可选）：
  sudo cp $PREFIX/deploy/nginx.conf.example /etc/nginx/sites-enabled/nobelog.conf
  sudo nano /etc/nginx/sites-enabled/nobelog.conf   # 改 server_name 与证书路径
  sudo nginx -t && sudo systemctl reload nginx
  sudo certbot --nginx -d ${DOMAIN:-你的域名}

查看日志：journalctl -u $SERVICE -f
NEXT
