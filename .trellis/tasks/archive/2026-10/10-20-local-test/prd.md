# 本地全面测试验收（v0.0.23）

## Goal

在只针对本机的前提下，把 v0.0.23 的验收测完并给出逐项证据：代码门禁、静态构建、动态服务、
热更新、安全回归、发布包一致性。不涉及远程。

## 测试矩阵

### A. 代码门禁
- `cargo fmt --check`、`cargo clippy --all-targets`（0 warning）、`cargo test`（90 项）。
- `cargo build --release --locked` 成功，二进制 `--version` = 0.0.23。

### B. 静态构建
- `SITE_URL=<占位域名> nobelog build --self-assets --out /tmp/...`：15 页面、`robots.txt` 存在且 Sitemap 用 SITE_URL。
- 与冻结 `public/` 归一化 diff：只允许 feed（时区修复）、index（相对日期）、CSS 哈希三类既有差异。
- 资源确定性：有/无 `public/` 两种 root 构建 CSS 哈希一致。

### C. 本机动态服务（systemd `nobelog-local`，本机端口）
- 9 条路由 + 404；页面/feed/sitemap 200 且可解析。
- 缓存头：`/assets/**` 长缓存且 ETag 条件请求 304；`/images/**` 单一 `Cache-Control: no-cache`。
- 压缩：文本响应带 `Vary: Accept-Encoding`，br/gzip 生效；<1024 字节不压。
- Range / If-None-Match / 304；非 GET/HEAD 方法行为记录。
- XFH 注入回退；路径穿越编码变体全部 404；`/index.html`、`//` 等一致性记录。

### D. 热更新（临时实例，独立 fixture，不碰生产端口）
- 新增文章 → 1s TTL 后可见；同长度改写生效；删除 → 404 且首页移除。
- 坏内容：只报一条错 + 10s 退避，旧快照继续；修好后立即恢复。
- 符号链接文章：改目标触发重建；broken link 不崩。
- `/images` 在动态模式为 no-cache + 304。

### E. 发布包
- 解包 `dist/nobelog-0.0.23-linux-x86_64.tar.gz`：VERSION、`--version`、`public/robots.txt`、manifest CSS 哈希、
  `sha256sum -c`。

### F. 汇总
- 输出 PASS/FAIL 表 + 证据（命令、状态码、header 片段、耗时）；FAIL 项要能复现。

## 约束

- 不访问远程、不部署远程、不改远程配置；本机端口冲突时用高位端口。
- 不改仓库源码与内容；临时实例进程验证后必须按 PID 关闭。
- 不重启生产 `nobelog-local`（除非发现服务级问题需要复测，先报告）。

## Acceptance

- [ ] A–E 全部执行并有证据；无未解释的 FAIL。
- [ ] 本机生产服务保持 v0.0.23 健康（pid 不变/或仅按需重启并记录）。
- [ ] Trellis journal + 归档；提交记录。
