# 发布 v0.0.27（图标引用清理）

## Goal

按既定发版流程发布 v0.0.27：版本三源同步、打包、tag、GitHub Release（默认），并重启本机使版本一致。

## 本版变更（v0.0.26 → v0.0.27）

- 暗色图标补真实引用：head 新增两条 `media="(prefers-color-scheme: dark)"` 的 favicon 声明
  （`site-icon-32-dark.png`、`site-icon-192-dark.png`），消除「孤儿图标」。
- 删除无用途的 `site-icon-transparent-512.png`（停止生成），构建产物不再包含无引用图标。

## 流程

1. 版本三源同步 `0.0.27`（Cargo.toml / package.json / Cargo.lock）。
2. 门禁 `cargo fmt --check`、`clippy --all-targets`、`test`（92 项）。
3. `SITE_URL=https://z.miio.cc VERSION=0.0.27 tools/package.sh`；校验包内 VERSION / `--version` / robots /
   `public/images/` 图标清单（7 个，无 transparent-512）/ 包内 head 7 条声明 / `sha256sum -c`。
4. commit + annotated tag `v0.0.27`（message `0.0.27：图标引用清理`）+ push。
5. GitHub Release（默认动作）：标题 `v0.0.27：图标引用清理`，notes 写变更与 sha256，附 tar.gz + SHA256SUMS。
6. 本机 `nobelog-local`：rebuild + 重启一次，确认 `--version` 0.0.27、head 7 条声明、端点与回归通过。

## Acceptance

- [ ] 版本三源一致；包校验通过；tag 与 Release 已推送。
- [ ] 本机跑 v0.0.27；7 个图标端点 200、head 7 条声明、9 路由回归通过。
- [ ] Trellis journal + 归档。
