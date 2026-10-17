# nobelog 0.0.1 部署包

免编译安装，目标机只需要：Linux x86_64、systemd、glibc ≥ 2.35（Ubuntu 22.04+/Debian 12+）。

## 快速安装

```bash
tar xzf nobelog-0.0.1-linux-x86_64.tar.gz
cd nobelog-0.0.1-linux-x86_64
sudo DOMAIN=blog.example.com ./install.sh
```

安装后服务监听 `127.0.0.1:10086`，由 nginx 终止 TLS 并反代。

## 参数

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `PREFIX` | `/opt/nobelog` | 安装目录 |
| `PORT` | `10086` | 本机监听端口 |
| `DOMAIN` | 空 | 站点域名；设置后写入 `SITE_URL=https://域名`，canonical/sitemap 用它 |
| `USER_NAME` | `nobelog` | 运行服务的系统用户 |
| `NO_SYSTEMD` | `0` | 设为 1 只解包，不装服务 |

## 目录结构

```
nobelog                 二进制
content/                文章与单页（改这里即可）
vendor/                 主题 CSS / 字体 / patch
rust/client/main.js     客户端脚本
deploy/                 systemd 与 nginx 模板
install.sh              安装脚本
```

## 更新内容

把新的 `content/` 覆盖到 `$PREFIX/content/`，动态模式下一次请求自动生效，无需重启。

## 纯静态模式

二进制也能直接构建静态产物：

```bash
$PREFIX/nobelog build --self-assets --out /var/www/nobelog
```

然后让 nginx 直接 `root /var/www/nobelog;`，不再需要常驻进程。

## 卸载

```bash
sudo systemctl disable --now nobelog
sudo rm -f /etc/systemd/system/nobelog.service
sudo rm -rf /opt/nobelog
sudo userdel nobelog
```
