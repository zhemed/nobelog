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

## 本地运行

```bash
# 动态模式：改 md 后刷新即生效
cargo run --release --manifest-path rust/Cargo.toml -- serve --host 0.0.0.0 --port <port> --dynamic

# 生成静态产物到 public/
cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out public
```

也可以用仓库根的 pnpm 别名：`pnpm serve` / `pnpm serve:static` / `pnpm build`。
端口统一用占位符 `<port>`，实际默认值见 `package.json` 的 scripts 与 CLI。

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

对外部署时设置 `SITE_URL`（例如 `SITE_URL=https://<domain>`），sitemap、RSS、canonical、页脚与分享链接都会使用它。
未设置时，动态模式会按请求的 `Host` / `X-Forwarded-Proto` 自动推断；静态构建则回退到默认地址。

## 数据

`content/` 就是博客的全部数据：文章、单页、图片。纯 Markdown + frontmatter，
换任何部署方式都不需要迁移数据库。

## 许可

- 代码：[MIT](LICENSE)
- 文章：CC BY 4.0
