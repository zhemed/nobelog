# 只读审核：博客是否包含 Node.js 与 PHP

## Goal

回答用户问题：我们的博客**有没有包含 Node.js 和 PHP**。只读审核，不改任何文件。

## 需要分别界定并给出证据的四类位置

1. **源码树（git 跟踪文件）**：是否存在 `.php`、`composer.json`、`*.js/.mjs/.cjs/.ts`、
   `package.json`、`node_modules/`、`pnpm-lock.yaml` 等；分别说明「是什么用途」。
2. **构建产物 `public/`**：产物里是否出现 Node/PHP 相关文件（例如 node_modules、`.php`、`package.json`）。
3. **发布包 `dist/nobelog-*-linux-x86_64.tar.gz`**（以最新 v0.0.28 为准）：解包后逐类检查。
4. **运行时与部署面**：本机 `nobelog-local` 服务与部署文档（nginx 片段、systemd unit、README/spec）
   是否要求 Node.js 或 PHP 运行时；本机是否存在与本站相关的 Node/PHP 进程或依赖。

## 判定口径（避免含糊）

- **Node.js**：区分「(a) 需要 Node 运行时才能构建/运行」「(b) 只是浏览器端 JS（无需 Node）」
  「(c) 仓库里有 npm/pnpm 元数据但实际不参与构建」三种情况，分别下结论。
- **PHP**：区分「(a) 存在 PHP 源文件」「(b) 部署/运行需要 PHP-FPM 或 php 二进制」「(c) 仅文档提及」。

## 交付

- 每条结论都带**可复核命令 + 原始输出**（文件清单、计数、包内清单、进程/依赖查询）。
- 最后给一句话总答复：是否包含 Node.js、是否包含 PHP；若「包含」是哪种含义。
- 只读：不改文件、不装包、不重启服务、不访问远程/外网。

## Acceptance

- [ ] 四类位置逐类有证据（命令 + 输出）。
- [ ] Node.js / PHP 各给出分层结论（a/b/c）。
- [ ] 不含真实域名、IP、端口（用 `<domain>`、`<port>` 占位）。
- [ ] Trellis journal + 归档。
