# 去 Node 化收尾：删除 pnpm 别名与过期 Node 说明

## Goal

用户授权删除。把仓库里所有「Node 生态残留」清干净，让「运行时/构建/部署都不需要 Node.js」
在文件层面也成立；同时修正会误导人的过期说明与报错文案。

## 决策（Lead 定，记录取舍）

| 对象 | 处置 | 理由 |
|---|---|---|
| `package.json` | **删除** | 唯一作用是 pnpm 别名（5 条 scripts 全包装 cargo）+ 版本标记。删除后仓库无任何 Node 生态文件；代价是版本同步由「三源」变「两源」（`rust/Cargo.toml` + `rust/Cargo.lock`），需同步更新 spec 与发版流程；回滚只需 revert 一笔提交 |
| `vendor/fonts/README.md` | 删除「抓取方式（node scripts/fetch-fonts.mjs）」与 Node 版「构建集成（server/paths.ts、server/build.ts）」两节，改为现状说明（资产已入库、由 `rust/src/assets.rs` 复制） | 脚本与 TS 时代文件都已不在仓库，属过期说明 |
| `rust/data/README.md` | 把「生成方式（作者期）」改为：脚本已随 Node 版移除，fixture 为冻结产物；保留格式与未命中行为的说明 | 同上 |
| `rust/src/assets.rs` | 四处报错文案去掉「先运行 node scripts/fetch-*.mjs」，改为「仓库不完整/资产未入库」的正确提示 | 脚本不存在，提示会误导 |
| `.gitignore` | 删除 `node_modules/` 一行 | 仓库不再有 Node 生态 |
| `.trellis/spec/frontend/*` | 同步：pnpm 命令改 cargo 命令；`package.json` 说明改「已删除」；发版版本同步改两源 | 规范必须与仓库现状一致（**由 Lead 改**，成员不改 spec） |

保留项：`rust/README.md` 里「Node 源码已移除 / 不再依赖 Node」的历史对比说明（属历史，明确说明不依赖），
以及 `.trellis/spec/guides/cross-layer-thinking-guide.md` 中与本站无关的 npm 案例。

## 交付

1. 成员：`vendor/fonts/README.md`、`rust/data/README.md`、`rust/src/assets.rs`、`package.json`（删）、`.gitignore`。
2. Lead：`.trellis/spec/frontend/directory-structure.md`、`build-serve-deploy.md`、`rendering-templates.md`。
3. 门禁 `cargo fmt --check && cargo test && cargo clippy --all-targets`（`assets.rs` 文案改动不得破坏测试）。
4. 验证：
   - `git ls-files | grep -E 'package\.json|pnpm-lock|node_modules'` 应为空；
   - 全仓 grep `pnpm|npm |node_modules|fetch-fonts|fetch-theme-assets|gen-highlight-golden`
     仅剩历史/无关说明（逐条列出）；
   - 无 `package.json` 时构建链仍可用：`cargo build --release --locked` + `nobelog build --self-assets --out <tmp>`；
   - 静态构建产物不变（CSS/JS 哈希与 v0.0.28 一致）。
5. 提交（`[task:10-21-denode-cleanup]`）、journal、归档。

## 约束

- 只做与「去 Node 化」相关的删除与文案修正，不重构其它逻辑。
- 不重启本机 systemd 服务（用临时实例/构建验证）。
- 文档与提交信息不含真实域名/IP/端口，用 `<domain>`、`<port>` 占位。

## Acceptance

- [ ] 仓库无 Node 生态文件；过期 node 脚本说明与报错文案已修正。
- [ ] 门禁全绿；无 package.json 情况下构建与打包链路可用。
- [ ] spec 与仓库现状一致（pnpm 命令、版本两源、package.json 已删除）。
- [ ] 提交、journal、归档完成。
