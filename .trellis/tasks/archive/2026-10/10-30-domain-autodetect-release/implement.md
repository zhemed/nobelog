# 实施计划：域名自动识别维护与 v0.0.30 发版

## 阶段 A：实现

- [x] A1 读取任务 `prd.md`、`design.md`、`implement.jsonl` 列出的 spec 与 research 报告。
- [x] A2 修改 `rust/src/serve.rs`：peer 判定、信任维度拆分、ConnectInfo 接入、`request_base_url` 与 handler/render 传递 peer。
- [x] A3 修改 `rust/src/site.rs`：更新未设置 SITE_URL 的启动告警文案。
- [x] A4 更新 `rust/src/serve.rs` 既有 5 组转发头测试并新增 peer 矩阵、混合信任、SITE_URL 优先级、总开关兼容、注入回退用例。
- [x] A5 修改 `rust/src/build.rs`：robots 测试样例域名改中性域名，补 trim 断言。
- [x] A6 修改 `tools/package.sh`：SITE_URL 与 VERSION 强校验，打印实际值。
- [x] A7 更新文档：`README.md`、`.trellis/spec/frontend/routes-and-urls.md`、`.trellis/spec/frontend/build-serve-deploy.md`，必要时补 `rendering-templates.md` 指向。
- [x] A8 版本两源升到 0.0.30：`rust/Cargo.toml`、`rust/Cargo.lock`。
- [x] A9 运行 `export PATH=/root/.cargo/bin:$PATH` 后 `cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test`。
- [x] A10 用 `SITE_URL=https://example.com` 重建 `public/` 与 `rust/dist/`，连续两次构建哈希一致；`./rust/target/release/nobelog diff public rust/dist` 只允许预期差异。
- [x] A11 打包自测：`VERSION=0.0.30 SITE_URL=https://example.com tools/package.sh`；`cd dist && sha256sum -c SHA256SUMS`；包内 VERSION 与解包二进制 `--version` 均为 0.0.30。
- [x] A12 报告改动文件、命令输出、失败与偏差给 Lead；不执行 commit、tag、push、Release、重启。

## 阶段 B：Lead 复核与发版

- [x] B1 Lead 复核 diff 与测试证据，抽查行为矩阵与文档。
- [x] B2 Lead 提交代码、文档、脚本、版本号三个逻辑提交，均带 `[task:10-30-domain-autodetect-release]`。
- [x] B3 Lead 打 annotated tag `v0.0.30`，推送 main 与 tag。
- [x] B4 Lead 创建 GitHub Release，附包与 SHA256SUMS。
- [x] B5 成员执行重启：`cargo build --release --locked` 后 `systemctl restart nobelog-local`。
- [x] B6 冒烟：9 条路由与 404、7 个图标、canonical 默认行为、本地环回 peer 加 XFP、CSS 哈希、`--version`。
- [x] B7 Lead 归档任务、写 journal、提交并推送收尾记录。

## 阶段 C：验收命令

```bash
cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test
cd .. && SITE_URL=https://example.com cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out public
SITE_URL=https://example.com cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out rust/dist
./rust/target/release/nobelog diff public rust/dist
VERSION=0.0.30 SITE_URL=https://example.com tools/package.sh
(cd dist && sha256sum -c SHA256SUMS)
curl -sS -H 'Host: blog.example.com' -H 'X-Forwarded-Proto: https' http://127.0.0.1:10086/ | grep -o 'rel="canonical" href="[^"]*"'
```

## 回滚点

- 代码回滚：`git revert` 发版提交；
- 服务回滚：解包 `dist/nobelog-0.0.29-linux-x86_64.tar.gz` 的二进制重启，或 checkout v0.0.29 重新构建。
