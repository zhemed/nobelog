# 部署文档：nginx 建议双拓扑化（反代 / 静态 root）

## Goal

把仓库部署文档里的 nginx 建议改为「反代 (a)」与「静态 root (b)」双版本，显式标注拓扑；
新增部署拓扑一节；全量修正 `try_files` / `location` / `Cache-Control` / `add_header` 片段；
文档中不出现真实域名/IP/端口/主机名/私密路径。**只改文档，不改代码。**

## 三种拓扑（文档必须按此表述）

| 代号 | 拓扑 | TLS | 静态文件 | 压缩 | 缓存头 |
| --- | --- | --- | --- | --- | --- |
| (a) | nginx 反代 → 应用（应用托管 `public/`） | nginx | 应用 | 应用或 nginx（运维确认） | 通常应用下发，nginx 可覆盖 |
| (b) | nginx `root`/`alias` 托管静态目录 | nginx | nginx | nginx | nginx |
| (c) | 动态模式直连（无 nginx 或仅透传） | 应用/其它边缘 | 应用 | 应用 | 应用 |

## Requirements

- R1 新增「部署拓扑」章节：表格 + 如何判断自己属于哪种（看 `proxy_pass` 是否存在、看 `root/alias`、
  看响应头是否与 `PAGE_CACHE`/`ASSET_CACHE` 一致、看 `Server` 头等），并说明"改缓存头应落在哪一层"。
- R2 `/images/**` 给出两版片段：
  - (a) `proxy_pass` + `proxy_hide_header Cache-Control` + `add_header ... always` + `proxy_set_header Host/X-Forwarded-Proto`；
  - (b) `try_files $uri =404` + `add_header ... always`。
- R3 反代通用注意事项：`proxy_hide_header` 的必要性（避免双 Cache-Control）、`add_header` 的继承规则
  （location 内出现 add_header 后不再继承上层）、CDN/共享缓存下 `no-cache` 只保证回源验证（purge 由运维决定）。
- R4 全量排查文档内所有 nginx 片段（`grep try_files|location |add_header|Cache-Control`），统一双拓扑标注，不留单拓扑片段。
- R5 占位符化：`<domain>`、`<port>`、`<app_port>`、`<app_upstream>`、`<cert_path>`；不得出现真实域名/IP/端口/主机名。
  远端/生产配置改动一律标注「需运维授权，由运维执行」；文档只描述"应该长什么样"。
- R6 范围：`.trellis/spec/frontend/build-serve-deploy.md` 为主；顺带把 `.trellis/spec/frontend/routes-and-urls.md`
  与 `README.md`、`rust/README.md` 中的真实默认端口/回环地址改为占位符或引用（见备注）。
- R7 保留原有章节结构与术语，只替换/新增；不改代码、不改构建产物。

## 备注（范围界定）

- `content/posts/*.md` 是已发布文章内容，不改。
- `.trellis/workspace/*` 与 `.trellis/tasks/archive/*` 是历史记录，不改（回复中说明）。
- 本机未安装 nginx，`nginx -t` 无法本地执行；需在回复中如实说明，并给出可执行的验证方式（运维侧或授权安装后）。

## Acceptance Criteria

- [ ] 文档中任一 nginx 片段都带 (a)/(b) 拓扑标签；无裸 `try_files`。
- [ ] `/images/**` 两种拓扑各一段，反代段含三件套（proxy_pass / proxy_hide_header / add_header always）。
- [ ] 文档内无真实域名/IP/端口（`grep` 证据）。
- [ ] 新增拓扑表与判断方法。
- [ ] 只改文档；`git diff --stat` 仅 `.md` 文件。
- [ ] 提交消息带 `[task:10-20-nginx-topologies]`。
