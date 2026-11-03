# 发版 0.0.32（发布包默认 example.com 占位）

## Goal

按用户新规则发版：发布包默认使用 `https://example.com` 占位域名，使产物与具体站点解耦、他人可直接部署。

## 改动

- `tools/package.sh`：`SITE_URL` 缺省值改为 `https://example.com` 并打印提示；仍拒绝 http、本机地址、已下线域名。
- `README.md` 与 `.trellis/spec/frontend/build-serve-deploy.md`：同步该规则。
- `rust/Cargo.toml` / `Cargo.lock`：版本 0.0.31 → 0.0.32。

## 流程与验证

1. 门禁：`fmt` / `clippy` / `test`（98）全绿；
2. 打包：`VERSION=0.0.32 tools/package.sh` → `dist/nobelog-0.0.32-linux-x86_64.tar.gz` + `SHA256SUMS`；
3. `sha256sum -c SHA256SUMS` 通过；解包 `./nobelog --version` = 0.0.32；包内 `public/index.html` 的绝对地址为 `https://example.com`；
4. 提交 + annotated tag `v0.0.32` + push + `gh release create`；
5. 重启线上服务加载新二进制。
