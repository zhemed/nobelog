# VPS 部署（动态模式，本地写 + 一键发布）

## 架构

```
本地：写 md → deploy.sh（编译二进制 + rsync）
                            │
                            ▼
VPS：/opt/nobelog/{nobelog, content/, vendor/, rust/client/main.js}
     systemd: nobelog serve --host 127.0.0.1 --port 10086 --dynamic
     nginx/caddy: 443 反代 → 127.0.0.1:10086
```

- **动态模式**：服务启动时在内存里构建 CSS/JS，请求时从 `content/` 渲染页面；
  改 md 后下一次请求即生效，**不需要 build、没有 public/**。
- 内容源仍是 git 里的 `content/`；VPS 上只有二进制、内容与静态素材。
- 资源：二进制 4.6 MB、content ~230 KB、vendor ~1.1 MB；常驻内存约 89 MB（syntect）。
- 传输：serve 自带 gzip/br（CSS gzip ≈28 KB、HTML ≈5.1 KB）。

## 本地常驻服务（可选）

本机/内网预览用 systemd 常驻，崩溃或 OOM 后自动拉起：

```bash
sudo cp deploy/nobelog-local.service /etc/systemd/system/
sudo systemctl daemon-reload && sudo systemctl enable --now nobelog-local
systemctl status nobelog-local        # 查看状态
journalctl -u nobelog-local -f        # 查看日志
```

单元带 `Restart=always`、`MemoryMax=512M`（博客常态约 89 MB），并限制文件系统写权限。

## 首次部署

```bash
# 1) 建用户与目录（VPS）
sudo useradd -r -s /usr/sbin/nologin nobelog
sudo mkdir -p /opt/nobelog/{content,vendor,rust/client}
sudo chown -R nobelog:nobelog /opt/nobelog

# 2) 安装 systemd 单元（deploy/nobelog.service）
sudo cp deploy/nobelog.service /etc/systemd/system/nobelog.service
sudo systemctl daemon-reload && sudo systemctl enable --now nobelog

# 3) nginx + TLS（deploy/nginx.conf.example）
sudo cp deploy/nginx.conf.example /etc/nginx/sites-enabled/nobelog.conf
sudo nginx -t && sudo systemctl reload nginx
sudo certbot --nginx -d blog.example.com
```

## 日常发布

```bash
DEPLOY_HOST=deploy@vps.example.com ./deploy/deploy.sh
```

脚本：编译 release → 同步 `content/`、`vendor/`、`rust/client/main.js`、二进制（原子替换）→
`systemctl restart` → 健康检查。内容改动无需重新构建。

静态模式（产物托管）仍然可用：

```bash
pnpm build                      # 生成 public/
DEPLOY_MODE=static DEPLOY_HOST=... ./deploy/deploy.sh
```

## 缓存策略（动态）

| 资源 | 头 |
| --- | --- |
| HTML / feed 之外的页面 | `no-cache` + 强 ETag（内容不变返回 304） |
| `/assets/main.<hash>.css|js` | `public, max-age=31536000, immutable`（哈希随内容变） |
| `/assets-manifest.json` | `no-cache` |
| 字体 / 图片 | 短缓存（由 serve 决定） |

## 回滚

- 内容：`git checkout <commit>` 后重新 `deploy.sh`（或直接在 VPS 上恢复 content/）。
- 二进制：`git checkout <commit> && ./deploy/deploy.sh`。
- 静态模式：`public/` 快照在本地生成，可随时 `DEPLOY_MODE=static` 覆盖。

## 为什么不加数据库 / 后台

- 单作者、写少读多，git 已提供版本、diff、回滚、异地备份。
- 动态模式已经做到“改文件即生效”，不需要后台；若以后要浏览器写作，优先本地编辑器 + SSH 隧道。
- 评论用 giscus、搜索用构建期索引，都不需要数据库。

## 安全清单

- 服务只监听 `127.0.0.1`，由 nginx 终止 TLS。
- VPS 不装构建工具；SSH 只允许密钥登录。
- systemd 单元启用 `NoNewPrivileges` / `ProtectSystem=strict` / `ProtectHome`。
