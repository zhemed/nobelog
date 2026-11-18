# 修正服务器初始化文章：调换 sudo 安装与系统更新的顺序

## Goal

`content/posts/server-init-commands.md` 在刚装完的 Debian 上无法按顺序执行：第一条命令用了 `sudo`，但 sudo 尚未安装。只需把「装工具（含 sudo）」与「系统更新」两个代码块调换顺序，让 sudo 先装上，其它一律不动。

## Requirements

- R1 仅调换「系统更新」小节内两个代码块的顺序：先「装几个常用工具」（`apt update -y && apt install -y curl wget socat sudo unzip`），后「`sudo apt update && sudo apt upgrade -y`」。
- R2 不新增/删除任何文字、说明、提示块、步骤；不改其它段落、frontmatter、标题、日期、slug、tags、categories。
- R3 不改模板、样式、站点配置、Rust 代码。
- R4 验证：静态构建通过；渲染产物中两个代码块顺序正确；动态服务刷新后可见。

## Acceptance Criteria

- [x] 相对原文的 diff 只有两个代码块的位置互换（4 增 4 删），无任何新增文字。
- [x] 渲染 HTML 中先出现装工具命令、后出现 `sudo apt update`。
- [x] 站点构建通过；其它文章/页面产物不受影响。
- [x] 提交带任务锚点 `[task:11-18-init-sudo-order]`；不推送远端。
- [x] 「内容编辑只做最小改动」的流程已写入 spec。

## 复盘（2026-11-18）

- 首版实现过度：加了 Debian/Ubuntu 差异说明块、`adduser`/`usermod` 建用户步骤、`deluser` 的 Ubuntu 语境说明，被用户指出「本来就调换一下顺序就行了，你在这啰啰嗦嗦瞎改瞎添加什么呢」。
- 已全部撤销，只保留两个代码块互换；教训写入 `.trellis/spec/frontend/quality-guidelines.md` 的「内容编辑」小节。

## Notes

- 用户原话与边界见 `directive.md`。

## 验证结果（2026-11-18）

- 相对原文 diff：仅两个代码块互换，4 增 4 删，无任何新增文字。
- 渲染顺序：装工具（`apt update -y && apt install -y curl wget socat sudo unzip`）在前，`sudo apt update && sudo apt upgrade -y` 在后。
- 静态构建通过（15 页 155511 字节）；动态服务热更新后顺序正确，HTTP 200。
- 阅读时长回到 1~2 分钟。
- spec 规则已写入 `.trellis/spec/frontend/quality-guidelines.md` 的「内容编辑（文章 / 单页）」小节。
- 未推送远端。
