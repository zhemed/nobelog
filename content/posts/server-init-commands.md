---
title: 常用服务器初始化命令
date: 2026-10-18 12:15:00
slug: server-init-commands
tags: [运维, Linux]
categories: [备忘]
description: 新机器到手后的常用命令：系统更新、Docker 与 SSH 设置。
---

新机器到手，先跑一遍这些。内容主要是 Debian/Ubuntu 系，个别是 OpenWrt。

## 系统更新

```bash
sudo apt update && sudo apt upgrade -y
```

装几个常用工具：

```bash
apt update -y && apt install -y curl wget socat sudo unzip
```

## 开启 root 登录

```bash
sudo passwd root
sudo nano /etc/ssh/sshd_config
```

把这一行：

```text
#PermitRootLogin prohibit-password
```

改成：

```text
PermitRootLogin yes
```

保存后重启 SSH：

```bash
sudo service ssh restart
```

::: warning 安全提示
允许 root 直接登录会显著放大暴力破解的风险。至少配合密钥登录、改端口或 fail2ban 使用；更稳妥的做法是保留普通用户 + sudo。
:::

## 删除默认用户

```bash
sudo deluser --remove-home ubuntu
```

## 重启与关机

```bash
shutdown -r now   # 重启
shutdown -h now   # 关机
```

## 安装 Docker

```bash
bash <(curl -sSL https://gitee.com/SuperManito/LinuxMirrors/raw/main/DockerInstallation.sh)
chmod +x /usr/local/bin/docker-compose

docker --version
docker-compose --version
```

## OpenWrt 备注

路由器上第一次 SSH 断开的问题，需要补 sftp server：

```bash
opkg update
opkg install openssh-sftp-server
```

## 其他

- 重启 Web UI（OpenWrt）：`/etc/init.d/uhttpd restart`
- 上面这些只是速查，具体环境还是要先确认系统版本和防火墙后端。
