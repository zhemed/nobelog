# 吱吱

一个用 Rust 写的个人博客：Markdown 即内容源，构建成纯静态页面，也支持服务端实时渲染。

## 写文章

文章放在 `content/posts/`，文件名即 slug：

```markdown
---
title: 标题
date: 2026-10-17 12:00:00
slug: my-post
tags: [随笔]
categories: [生活]
description: 一句话摘要
---

正文用 Markdown。
```

单页放在 `content/pages/`（如 `about.md`）。

## 工具链

Rust 版本固定为 **1.99.1**，由仓库根的 `rust-toolchain.toml` 声明，`rust/Cargo.toml` 的
`rust-version` 也在 1.99.1 —— 两处一致，不要单独改其中一处。

**新机器首次准备（工具装在当前用户 HOME 下，不污染系统）：**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
# 然后进入仓库执行任意 cargo 命令，rustup 会按 rust-toolchain.toml 自动装 1.99.1
```

注意：必须用 **rustup** 装的 Rust。发行版包（`apt install cargo rustc`）不会读取
`rust-toolchain.toml`，会静默使用系统自带的旧版本。

任何一步可用自检脚本确认：

```bash
./tools/check-rust.sh   # 期望输出：Rust 工具链 OK：rustc 1.99.1 / cargo 1.99.1
```

## 本地运行

```bash
# 动态模式：改 md 后刷新即生效
cargo run --release --manifest-path rust/Cargo.toml -- serve --host 0.0.0.0 --port <port> --dynamic

# 生成静态产物到 public/
cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out public
```

静态服务用同一条 cargo 命令（去掉 `--dynamic` 即静态模式），端口用 `<port>` 占位，
默认值见 `cargo run --manifest-path rust/Cargo.toml -- serve --help`。

## 目录

| 路径 | 用途 |
| --- | --- |
| `content/posts/` | 文章（Markdown + frontmatter） |
| `content/pages/` | 单页（关于、我） |
| `content/images/` | 站内图片 |
| `rust/` | 生成器与服务（content → render → artifacts） |
| `vendor/` | 主题样式与字体 |
| `public/` | 构建产物（不提交） |

## 部署提示

对外部署时优先设置 `SITE_URL`（例如 `SITE_URL=https://<domain>`），sitemap、RSS、canonical、页脚与分享链接都会使用它。
未设置时，动态模式按请求的 `Host` 推断域名，scheme 默认 `http`；只有客户端来自本机或私网地址时，才额外采信
`X-Forwarded-Proto` 还原 https。`X-Forwarded-Host` 与其它场景的转发头仍需显式设置 `NOBELOG_TRUST_FORWARDED=1` 后才采信。
静态构建不参与请求推断，直接回退到默认地址。

发布包用 `SITE_URL=https://<domain> VERSION=<x.y.z> tools/package.sh` 打包，并用 `SITE_URL` 重建包内 `public/`
的静态产物。**不传 `SITE_URL` 时默认用占位域名 `https://example.com`**，发布包与具体站点解耦，其他人可以直接部署；
部署方拿到包后用自己的域名重新构建即可。脚本仍会拒绝 http、本机地址与已下线域名。

## 数据

`content/` 就是博客的全部数据：文章、单页、图片。纯 Markdown + frontmatter，
换任何部署方式都不需要迁移数据库。

## 许可

- 代码：[MIT](LICENSE)
- 文章：CC BY 4.0
