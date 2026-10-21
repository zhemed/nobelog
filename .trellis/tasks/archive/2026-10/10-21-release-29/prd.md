# 发布 v0.0.29（去 Node 化收尾）

## Goal

按既定发版流程发布 v0.0.29。**注意**：`package.json` 已删除，版本同步从三源变**两源**
（`rust/Cargo.toml` + `rust/Cargo.lock`），spec 已同步更新。

## 本版变更（v0.0.28 → v0.0.29）

- 删除 `package.json`（原 pnpm 别名，无依赖）与 `.gitignore` 的 `node_modules/` 行：
  仓库不再含任何 Node 生态文件。
- 清理过期说明：`vendor/fonts/README.md`（`node scripts/fetch-fonts.mjs`、TS 时代构建集成）、
  `rust/data/README.md`（`node rust/tools/gen-highlight-golden.mjs`）。
- 修正 `rust/src/assets.rs` 四处报错文案（不再指向不存在的 node 脚本）。
- README / rust/README / spec 的命令入口由 pnpm 别名改为 cargo 等价命令。

## 流程

1. 版本两源同步 `0.0.29`（`rust/Cargo.toml`、`rust/Cargo.lock`）。
2. 门禁 `cargo fmt --check`、`clippy --all-targets`、`test`（92 项）。
3. `SITE_URL=https://z.miio.cc VERSION=0.0.29 tools/package.sh`；校验包内 VERSION / `--version` / robots /
   CSS 哈希仍为 `main.1ca5c955.css` / 包内无 Node 生态文件 / `sha256sum -c`。
4. commit + annotated tag `v0.0.29`（message `0.0.29：去 Node 化收尾`）+ push。
5. GitHub Release（默认动作）：标题 `v0.0.29：去 Node 化收尾`，notes 写变更与 sha256，附 tar.gz + SHA256SUMS。
6. 本机 `nobelog-local`：rebuild + 重启一次，确认 `--version` 0.0.29、图标/head/回归通过。

## Acceptance

- [ ] 版本两源一致；包校验通过；tag 与 Release 已推送。
- [ ] 本机跑 v0.0.29；CSS 哈希、head 7 条声明、9 路由回归通过。
- [ ] Trellis journal + 归档。
