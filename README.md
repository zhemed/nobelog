# 吱吱

[![CI](https://github.com/zhemed/nobelog/actions/workflows/ci.yml/badge.svg)](https://github.com/zhemed/nobelog/actions/workflows/ci.yml)

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

两种装法都行，行为一致：

| 装法 | 入口（`cargo`/`rustc`） | 工具链本体 |
| --- | --- | --- |
| 默认（每用户） | `$HOME/.cargo/bin` | `$HOME/.rustup/toolchains/` |
| 系统级（本机现况） | `/usr/local/bin/*` 包装脚本 | **`/opt/rust/rustup`（全机共享）** |

本机采用系统级方案：`/usr/local/bin/{cargo,rustc,rustup…}` 是包装脚本，会先把
`RUSTUP_HOME` 固定为 `/opt/rust/rustup` 再转交给 `/opt/rust/cargo/bin/*`，因此**任何用户、
任何进程**（含非登录 shell、同时不受 `~/.rustup` 影响）拿到的都是同一套 1.99.1。
登录 shell 另有 `/etc/profile.d/rust.sh` 兜底。

也就是说：入口可以共享，工具链一定是**按用户各自下载**的 —— 换用户或换机器后第一次在仓库里
跑 `cargo`，rustup 会照 `rust-toolchain.toml` 把 1.99.1 拉进那个用户的家目录，不会串版本。

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

## 验证与基线

提交前在本地跑完整门禁（CI 在 push 到 `main` / PR 时跑同样的命令，但不跑 `diff`）：

```bash
cd rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked && cargo build --release --locked
cd .. && SITE_URL=https://example.com ./rust/target/release/nobelog build --self-assets --out rust/dist
```

`nobelog diff` 以仓库根的 `public/` 作为逐字节冻结基准；它是 gitignored 的构建产物，全新 clone 里没有。
先还原基线再对比：

```bash
tools/fetch-baseline.sh            # 默认取 GitHub 最新 release
tools/fetch-baseline.sh v0.0.37    # 或固定某个 release tag
SITE_URL=https://example.com ./rust/target/release/nobelog diff public rust/dist
```

脚本只写 `public/`（从 release 包内还原，校验 sha256 与包内 `VERSION`），不执行任何 git 操作；
`NOBELOG_REPO` 可覆盖仓库。`diff` 允许的差异只有首页卡片相对日期（`N 天前`）这类时间漂移；
改模板 / Markdown 输出后必须看 `diff`。

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
