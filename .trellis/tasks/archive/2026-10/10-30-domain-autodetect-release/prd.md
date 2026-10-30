# 域名自动识别维护与 v0.0.30 发版

## Goal

维护域名自动识别逻辑，修复 2.3.4 核查发现的问题：改进客户生产机反代场景的 scheme 识别、修正文档漂移、补齐测试、加固发布脚本，并以 0.0.30 发版并重启本机服务上线。

## Requirements

1. **代码 R1**：动态模式新增 peer 判定。客户端 socket 地址为环回或私网时，额外采信 `X-Forwarded-Proto` 还原 https；`X-Forwarded-Host` 仍只由总信任开关控制；`SITE_URL` 保持最高优先；`NOBELOG_TRUST_FORWARDED=1` 保留为全量信任开关，行为与 v0.0.24 一致。
2. **测试 R2**：新增 peer 判定矩阵、混合信任、`SITE_URL` 优先级、总开关兼容、注入回退等用例；更新既有 5 组测试的函数签名；`rust/src/build.rs` 中过时的示例域名改为中性域名并补 trim 断言。
3. **文档 R3**：更新 `README.md`、`routes-and-urls.md`、`build-serve-deploy.md`，修正「按 Host 与 X-Forwarded-Proto 自动推断」的过时表述；补充发布包静态产物与 `SITE_URL` 的硬规则；冻结基准域名改为 `https://example.com` 并注明 example.com 已下线。
4. **发布脚本 R4**：`tools/package.sh` 强制校验 `SITE_URL` 非空且为 https、拒绝 example.com 与 127.0.0.1、校验 `VERSION` 与 `rust/Cargo.toml` 版本一致，并打印实际取值。
5. **版本 R5**：`rust/Cargo.toml` 与 `rust/Cargo.lock` 两源升到 0.0.30。
6. **构建验证 R6**：`cargo fmt --check`、`cargo clippy --all-targets`、`cargo test` 全绿；用 `SITE_URL=https://example.com` 重建 `public/` 与 `rust/dist/`，两次构建逐字节一致，`nobelog diff` 只剩预期差异。
7. **发版 R7**：提交带任务锚点；`VERSION=0.0.30 SITE_URL=https://example.com tools/package.sh`；`sha256sum -c` 通过，包内 `VERSION` 与二进制 `--version` 均为 0.0.30；打 annotated tag `v0.0.30`，push main 与 tag，创建 GitHub Release 附包与 SHA256SUMS。
8. **上线 R8**：重新构建 release 二进制并重启本机 `nobelog-local`，冒烟覆盖 9 条路由与 404、图标、canonical 行为、本地环回 peer 加 XFP、CSS 哈希。
9. **留痕 R9**：Trellis journal 与任务归档，提交带任务锚点。

## Acceptance Criteria

- [ ] 行为矩阵：公网 peer 伪造 XFP 无效；环回或私网 peer 的 XFP https 生效；XFH 默认不采信；SITE_URL 最高优先；总开关行为与 v0.0.24 兼容。
- [ ] 新增测试全部通过，`cargo test` 用例数增加且 0 失败。
- [ ] 活跃文档中无 example.com 引用，README 与 spec 描述与新行为一致。
- [ ] `package.sh` 对空 SITE_URL、http 协议、example.com、127.0.0.1、VERSION 不匹配均 fail-fast。
- [ ] 发布包内 `public/index.html` 的 canonical 为 `https://example.com/`；包内 VERSION 与二进制 --version 均为 0.0.30。
- [ ] tag v0.0.30 与 GitHub Release 非草稿非预发布，main 与 tag 已推送。
- [ ] 本机服务重启后运行 0.0.30，冒烟全绿，NRestarts 增加 1。
- [ ] 预期产物之外的已跟踪文件无残留改动，全部提交带 `[task:10-30-domain-autodetect-release]` 锚点。

## Constraints

- 不在运行期硬编码任何生产域名；`example.com` 只用于发布包静态产物示例。
- 不改历史归档任务、旧 Release、旧 dist 发布包。
- 不修改 systemd 单元；重启前先构建 release 二进制。
- 私网内其他客户端可伪造 scheme 的取舍必须写进 spec；影响仅限协议，不扩大域名劫持面。
- 回滚路径：保留 v0.0.29 包，异常时解包旧二进制并重启。

## Notes

- 用户决策：采用私网 peer 信任方案；发布包用中性示例域名 `example.com`；发版后重启本机服务并冒烟。
- 2.3.4 指外部需求条目，本轮按能力语义维护。
- 调研依据：`.trellis/tasks/10-30-domain-autodetect-release/research/domain-maintain-plan.md` 与已归档的 `domain-autodetect.md`、`sync-audit.md`。
