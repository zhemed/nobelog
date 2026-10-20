# 安装 nginx 并验证部署文档片段

## Goal

用户授权本机直接安装缺失依赖。安装 nginx，用真实 `nginx -t` 校验文档中的两段 nginx 配置
（占位符替换后），并做最小行为验证；必要时修正文档。

## Requirements

- R1 `apt-get install -y nginx`（本机无 nginx）；**避免占用 80 端口影响现有环境**：
  安装后如 nginx.service 被自动启动，立即 stop + disable，并确认未监听 80。
- R2 用 /tmp 下的独立 conf（`-p` 独立 prefix、自定义 pid/error_log）分别测试：
  - (a) 反代片段：`/images/` location 指向 `127.0.0.1:10086`（本机应用），含 `proxy_pass`、
    `proxy_http_version 1.1`、Host/XFP、`proxy_hide_header Cache-Control`、`add_header ... always`；
    `nginx -t` 通过；若起临时实例（非 80 端口）验证 `/images/test-cover.svg` 只有一个 `Cache-Control: no-cache`。
  - (b) 静态 root 片段：`root /tmp/.../webroot` + `try_files $uri =404` + `add_header ... always`；
    `nginx -t` 通过；临时实例验证存在的文件 200 且单条 no-cache、不存在文件 404 且仍带该头（always 生效）。
- R3 验证后关闭临时实例、清理临时目录；nginx 可保留安装（用户授权）但不得常驻/自启占 80。
- R4 若实测发现文档片段错误，修正文档并重新验证。
- R5 不部署远程、不改应用代码。

## Acceptance Criteria

- [ ] `nginx -t` 对两段配置均通过（附输出）。
- [ ] 行为验证：反代/静态 root 下 `/images/**` 响应只有一条 `Cache-Control: no-cache`；缺失文件 404 仍带头。
- [ ] 80 端口未被 nginx 占用；无自启服务残留。
- [ ] 文档与实测一致；Trellis 记录与提交。
