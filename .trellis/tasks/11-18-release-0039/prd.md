# 发版 0.0.39

## 内容

内容修正版，无 Rust 代码 / 模板 / 样式变化：

- **文章修正**：`content/posts/server-init-commands.md` 把「装几个常用工具（含 sudo）」独立成 `##` 小节并排到「系统更新」之前，解决刚装完的 Debian 没有 sudo、第一条命令无法执行的问题；顺序为 装工具 → 系统更新。
- **流程沉淀**：`.trellis/spec/frontend/quality-guidelines.md` 新增「内容编辑（文章 / 单页）」小节，规定内容编辑只做被点名的最小改动。
- 站点其它页面、样式、Rust 代码不变。

## 流程与验证

1. 版本同步：`rust/Cargo.toml` 与 `rust/Cargo.lock`（`name = "nobelog"` 项）0.0.38 → 0.0.39；
2. 门禁：`cd rust && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --locked`（预期 98 passed）`&& cargo build --release --locked`；
3. 打包：仓库根 `VERSION=0.0.39 tools/package.sh`（不传 `SITE_URL`，默认 `https://example.com` 占位）→ `dist/nobelog-0.0.39-linux-x86_64.tar.gz` + `SHA256SUMS`；
4. 包校验：`(cd dist && sha256sum -c SHA256SUMS)`；解包确认包内 `VERSION` = 0.0.39、`./nobelog --version` = `nobelog 0.0.39`、条目数约 110；
5. 提交与 tag：版本提交带锚点 `[task:11-18-release-0039]`；annotated tag `v0.0.39`；`git push origin main && git push origin v0.0.39`（含积压的 5 个本地提交）；
6. GitHub Release：`gh release create v0.0.39 --title "v0.0.39" --notes-file <notes> dist/nobelog-0.0.39-linux-x86_64.tar.gz dist/SHA256SUMS`；
7. 上线：`cargo build --release --locked` 后 `systemctl restart nobelog-local`；冒烟 9 条路由 + 404、CSS 哈希、feed pubDate；确认线上文章为新版顺序；
8. 留痕：Trellis 归档 + journal。

## Acceptance Criteria

- [ ] `rust/Cargo.toml`、`rust/Cargo.lock`、`rust/target/release/nobelog --version`、包内 `VERSION` 均为 0.0.39。
- [ ] fmt / clippy `-D warnings` / test（98 passed）/ build `--release --locked` 全绿。
- [ ] `dist/nobelog-0.0.39-linux-x86_64.tar.gz` 与 `SHA256SUMS` 生成，`sha256sum -c` 通过；解包后 `./nobelog --version` = 0.0.39。
- [ ] 版本提交带任务锚点；annotated tag `v0.0.39` 指向该提交；main 与 tag 均已推送。
- [ ] GitHub Release `v0.0.39` 存在，附 tarball + SHA256SUMS，notes 含本版要点与 sha256。
- [ ] 本机 `nobelog-local` 重启后 9 条路由 + 404 冒烟通过，文章顺序为 装工具 → 系统更新。
- [ ] 任务归档并记录 journal。

## Out of Scope

- Rust 代码、依赖、CI 配置、模板与样式变更；`SITE_URL` 配置（本机不需要域名）。
