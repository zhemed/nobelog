# Journal - zhemed (Part 1)

> AI development session journal
> Started: 2026-10-19

---



## Session 1: 确认无残留任务并启用 Trellis
<!-- trellis-session: v=2 fp=2fc6fa8afd2b0c6e -->

**Date**: 2026-10-19
**Task**: 确认无残留任务并启用 Trellis
**Branch**: `main`

### Summary

核实 nobelog 工作区干净、无游离进程；按用户确认在仓库内执行 trellis init --dsh -u zhemed，提交 .trellis/AGENTS.md/.agents/.dsh

### Main Changes

- 启用 Trellis 工作流

### Git Commits

| Hash | Message |
|------|---------|
| `30c82fa` | chore: 启用 Trellis 工作流（.trellis/ 与 DSH skills） [task:10-19-enable-trellis] |

### Testing

- [OK] git status 干净；nobelog-local.service 运行正常

### Status

[OK] **Completed**

### Next Steps

- 可选：处理 00-bootstrap-guidelines 填充 spec


## Session 2: 填充 .trellis/spec/frontend
<!-- trellis-session: v=2 fp=1a0be6a9380b0b25 -->

**Date**: 2026-10-19
**Task**: 填充 .trellis/spec/frontend
**Branch**: `main`

### Summary

按 Rust 静态站真实结构重写 frontend 指南：9 个文件，删除 React 类模板；记录 public/ 冻结基准、逐字节 diff、UTC 日期语义、CSS 裁剪等约束

### Main Changes

- .trellis/spec/frontend/*

### Git Commits

| Hash | Message |
|------|---------|
| `f4c1099` | docs: 填充 .trellis/spec/frontend（按 Rust 静态站真实结构） [task:00-bootstrap-guidelines] |

### Testing

- [OK] grep 校验无占位符；index 与文件集一致

### Status

[OK] **Completed**

### Next Steps

- 按 trellis-update-spec 增量维护


## Session 3: 博客全面审查（安全/渲染/构建/线上）
<!-- trellis-session: v=2 fp=4eb9be48b3ce8261 -->

**Date**: 2026-10-19
**Task**: 博客全面审查（安全/渲染/构建/线上）
**Branch**: `main`

### Summary

三路只读审查：无 P0；P1 两条（frontmatter slug 构建期路径穿越、动态模式 Host/X-Forwarded-* 注入且 base_url 未转义）；渲染侧 P1（RSS pubDate 时区偏移 8h）；构建侧 P2（裁剪依赖 public/ 导致两套 CSS）

### Main Changes

- 无（纯审查）

### Git Commits

(No commits - planning session)

### Testing

- [OK] cargo test 30 通过；clippy 0 warning；线上 8 路由 + 404 冒烟全绿；diff 冻结产物仅时间敏感差异

### Status

[OK] **Completed**

### Next Steps

- 按用户选择修复 P1 并补回归测试


## Session 4: 修复审查发现的 P1 与日期回归
<!-- trellis-session: v=2 fp=0b615bb4636d3379 -->

**Date**: 2026-10-19
**Task**: 修复审查发现的 P1 与日期回归
**Branch**: `main`

### Summary

slug/标签/分类路径穿越双层拦截、Host/XFH 白名单 + base_url 转义、RSS pubDate 时区；复审打回两轮后补 mtime 归一与空格偏移容错，52 测试全绿，未提交未部署

### Main Changes

- rust/src/{content,build,serve,templates,site}.rs

### Git Commits

(No commits - planning session)

### Testing

- [OK] cargo fmt --check / clippy --all-targets / test(52) 全绿；debug 构建 PoC 与注入实测通过；整站归一化 diff 仅 feed+index+CSS 哈希

### Status

[OK] **Completed**

### Next Steps

- 用户决定提交/部署；剩余 P2（CSS 裁剪依赖 public、动态热更新删除检测等）待排期


## Session 5: 同步 spec 并部署 P1 修复
<!-- trellis-session: v=2 fp=603e5aa57300dbb2 -->

**Date**: 2026-10-19
**Task**: 同步 spec 并部署 P1 修复
**Branch**: `main`

### Summary

spec 时区约定改为 UTC+8 挂钟存储/RSS 输出真实 UTC；提交并推送 9ad2ee3；release 重建后重启 nobelog-local（PID 830→63373），feed pubDate 与注入头实测均符合预期

### Main Changes

- .trellis/spec/frontend/*

### Git Commits

| Hash | Message |
|------|---------|
| `9ad2ee3` | fix: 修复构建期路径穿越、Host 注入与 RSS 时区（P1） [task:10-19-fix-p1-audit] |

### Testing

- [OK] 线上 8 路由 200/404；feed pubDate 14:40/04:15 GMT；XFH 注入回退；52 测试全绿

### Status

[OK] **Completed**

### Next Steps

- 剩余 P2：CSS 裁剪依赖 public/、热更新删除检测、SITE_URL、panic=unwind


## Session 6: 修复 P2：CSS 确定性、热更新可靠性、内容一致性
<!-- trellis-session: v=2 fp=ccc8f24ec59bfa5d -->

**Date**: 2026-10-19
**Task**: 修复 P2：CSS 确定性、热更新可靠性、内容一致性
**Branch**: `main`

### Summary

CSS 裁剪与 public 解耦（静态/动态同哈希 1f82b0f7）、动态热更新指纹/TTL/退避/图片 no-cache、非法 YAML/重复 slug/摘要双转义/提示块围栏/page slug 一致性；80 测试全绿，两路复核均接受，未提交未部署

### Main Changes

- rust/src/{assets,build,content,markdown,routes,serve}.rs、rust/tools/css_prune.rs

### Git Commits

(No commits - planning session)

### Testing

- [OK] cargo fmt/clippy/test 全绿；有/无 public 构建同 CSS；fixture 前后对比；reviewer-security 与 reviewer-render 均接受

### Status

[OK] **Completed**

### Next Steps

- 用户决定提交与部署；P3（符号链接、.mdx 扫描、保留名大小写）待排期


## Session 7: 修复 P3 并部署 P2+P3
<!-- trellis-session: v=2 fp=30c07a0d6c0fff27 -->

**Date**: 2026-10-19
**Task**: 修复 P3 并部署 P2+P3
**Branch**: `main`

### Summary

P3：符号链接指纹跟随+link target、.mdx CSS 扫描、保留名小写且补 11 项、updated 非法 fail-fast；87 测试全绿；release 重建后重启 nobelog-local（PID 63373→97813），线上 CSS 1f82b0f7、feed 时区与注入回退均保持

### Main Changes

- rust/src/{serve,assets,content}.rs

### Git Commits

| Hash | Message |
|------|---------|
| `e12cfd0` | fix: P3 修复（符号链接指纹、.mdx 扫描、保留名大小写、updated 校验） [task:10-19-fix-p3] |

### Testing

- [OK] cargo fmt/clippy/test 87；部署后 9 路由冒烟、CSS 确定性 cmp、pubDate、/images no-cache、XFH 回退

### Status

[OK] **Completed**

### Next Steps

- SITE_URL 待域名；可选图片哈希化；P3 剩余（broken link 语义、date fail-fast）需产品决策


## Session 8: 发布 v0.0.22
<!-- trellis-session: v=2 fp=901c8bf08e4ab456 -->

**Date**: 2026-10-20
**Task**: 发布 v0.0.22
**Branch**: `main`

### Summary

版本三源同步 0.0.22；cargo build --release --locked 通过；package.sh 产出 dist/nobelog-0.0.22-linux-x86_64.tar.gz（3960053B，sha256 e2320054…），包内容与 VERSION/--version 校验通过；annotated tag v0.0.22 已推送

### Main Changes

- rust/Cargo.toml、package.json、rust/Cargo.lock

### Git Commits

| Hash | Message |
|------|---------|
| `adc05c3` | chore: 发布 v0.0.22（版本号同步） [task:10-20-release] |

### Testing

- [OK] cargo test 87；sha256sum -c OK；包内 --version=0.0.22；远程 tag 存在

### Status

[OK] **Completed**

### Next Steps

- 可选：重启服务使版本号生效；SITE_URL 待域名


## Session 9: 上线 v0.0.22
<!-- trellis-session: v=2 fp=ca14e2e80bc16334 -->

**Date**: 2026-10-20
**Task**: 上线 v0.0.22
**Branch**: `main`

### Summary

重启 nobelog-local 上线 v0.0.22：MainPID 97813→185210，运行二进制 sha256 5467c921…（与包内一致，--version 0.0.22）；9 路由与 feed/CSS 冒烟全绿

### Main Changes

- 无（仅服务重启）

### Git Commits

| Hash | Message |
|------|---------|
| `24fc8d4` | chore: 归档 v0.0.22 发布任务并记录 journal [task:10-20-release] |

### Testing

- [OK] is-active、/proc/<pid>/exe sha256、CSS main.1f82b0f7.css、pubDate 14:40/04:15 GMT

### Status

[OK] **Completed**

### Next Steps

- SITE_URL 待域名


## Session 10: 默认维护 GitHub Releases + 补 v0.0.22 发布单
<!-- trellis-session: v=2 fp=84a6dcd41d98ef07 -->

**Date**: 2026-10-20
**Task**: 默认维护 GitHub Releases + 补 v0.0.22 发布单
**Branch**: `main`

### Summary

用户长期授权 Releases 默认更新；补建 GitHub Release v0.0.22（2 附件、Latest、标题对齐历史风格）；把发版流程（含 gh release 默认步骤）写入 spec

### Main Changes

- .trellis/spec/frontend/build-serve-deploy.md

### Git Commits

| Hash | Message |
|------|---------|
| `adc05c3` | chore: 发布 v0.0.22（版本号同步） [task:10-20-release] |

### Testing

- [OK] gh release view 核对 tagName/assets/url；资产 digest 与本地 sha256 一致；gh release list Latest

### Status

[OK] **Completed**

### Next Steps

- 下一版按新流程执行；SITE_URL 待域名


## Session 11: 核对远程 z.miio.cc 修复状态
<!-- trellis-session: v=2 fp=e834060d2cca8e92 -->

**Date**: 2026-10-20
**Task**: 核对远程 z.miio.cc 修复状态
**Branch**: `main`

### Summary

只读核对远程：feed pubDate 时区、确定性 CSS 1f82b0f7、canonical/sitemap 指向 z.miio.cc、XFH 不污染、404 正常；本地同 SITE_URL 重建产物与远程逐字节一致，确认为 v0.0.22。静态部署下 /images 走 PAGE_CACHE（非 dynamic 的 no-cache）

### Main Changes

- 无（只读核对）

### Git Commits

(No commits - planning session)

### Testing

- [OK] curl GET/HEAD 证据；cmp index/feed/sitemap/404/CSS IDENTICAL

### Status

[OK] **Completed**

### Next Steps

- 可选：nginx 对 /images 加 no-cache；补 robots.txt


## Session 12: 备好 robots.txt 与 nginx 建议
<!-- trellis-session: v=2 fp=d4bb7f8c0c43b0a6 -->

**Date**: 2026-10-20
**Task**: 备好 robots.txt 与 nginx 建议
**Branch**: `main`

### Summary

构建生成 robots.txt（含来源标记，SITE_URL 变化会覆盖自动产物、手写文件保留）；spec 增加静态部署 nginx /images 缓存建议；90 测试全绿。未部署远程

### Main Changes

- rust/src/build.rs、.trellis/spec/frontend/build-serve-deploy.md

### Git Commits

(No commits - planning session)

### Testing

- [OK] cargo fmt/clippy/test 90；同 out 两次构建 A→B 覆盖；手写保留；diff 仅多 robots.txt

### Status

[OK] **Completed**

### Next Steps

- 随下次发版部署；远程改 nginx 需授权


## Session 13: 发布 v0.0.23
<!-- trellis-session: v=2 fp=a47a28ae53034d18 -->

**Date**: 2026-10-20
**Task**: 发布 v0.0.23
**Branch**: `main`

### Summary

版本三源 0.0.23；SITE_URL=https://z.miio.cc 打包（robots.txt 含正确 Sitemap）；tag v0.0.23 已推送；GitHub Release 已创建（2 附件，Latest）

### Main Changes

- rust/Cargo.toml、package.json、rust/Cargo.lock

### Git Commits

| Hash | Message |
|------|---------|
| `95768e6` | chore: 发布 v0.0.23（版本号同步） [task:10-20-release-23] |

### Testing

- [OK] cargo test 90；包 sha256 084cd534…；gh release view 核对 assets=2

### Status

[OK] **Completed**

### Next Steps

- 远程部署新 public/ 后 robots.txt 生效（仅只读授权，需用户安排）


## Session 14: 本机部署 v0.0.23 并重启
<!-- trellis-session: v=2 fp=32b6638c7cab665c -->

**Date**: 2026-10-20
**Task**: 本机部署 v0.0.23 并重启
**Branch**: `main`

### Summary

本机 nobelog-local 从 v0.0.22 切到 v0.0.23：重启一次（PID 185210→197053，/proc/exe sha256 a7e257c2…）；冒烟 9 路由+404、feed pubDate 14:40/04:15、CSS 1f82b0f7、/images no-cache、XFH 回退均正常

### Main Changes

- 无（仅服务重启）

### Git Commits

| Hash | Message |
|------|---------|
| `173360b` | chore: 归档 v0.0.23 发布任务并记录 journal [task:10-20-release-23] |

### Testing

- [OK] systemctl active、exe sha、bw/curl 冒烟、条件请求 304

### Status

[OK] **Completed**

### Next Steps

- 远程部署仍由用户执行；可选让动态模式也提供 /robots.txt


## Session 15: 部署文档 nginx 双拓扑化
<!-- trellis-session: v=2 fp=cd2a71838487c475 -->

**Date**: 2026-10-20
**Task**: 部署文档 nginx 双拓扑化
**Branch**: `main`

### Summary

build-serve-deploy 新增部署拓扑章节（a反代/b静态root/c动态直连）与判定方法；/images 双片段（反代三件套/静态 try_files）；add_header 继承坑与 CDN 说明；占位符化真实域名/IP/端口；README 等 4 个 md 同步

### Main Changes

- .trellis/spec/frontend/{build-serve-deploy,routes-and-urls}.md、README.md、rust/README.md

### Git Commits

(No commits - planning session)

### Testing

- [OK] grep 占位符 0 命中；本机 /images 单条 no-cache、/assets 长缓存、页面 feed 200；nginx -t 未执行（本机无 nginx）

### Status

[OK] **Completed**

### Next Steps

- 如需 nginx -t 实测，需授权 docker pull nginx 或安装 nginx


## Session 16: 安装 nginx 并实测双拓扑片段
<!-- trellis-session: v=2 fp=5f1c859cc2aa5ae9 -->

**Date**: 2026-10-20
**Task**: 安装 nginx 并实测双拓扑片段
**Branch**: `main`

### Summary

用户授权本机装依赖：安装 nginx 1.18（避免占 80，已 stop+disable）；(a)反代与(b)静态root 两段配置均 nginx -t 通过；起临时实例实测 /images 单条 no-cache、缺失 404 仍带头；文档更新验证状态

### Main Changes

- .trellis/spec/frontend/build-serve-deploy.md

### Git Commits

| Hash | Message |
|------|---------|
| `02b5dbc` | docs: nginx 建议改为反代/静态 root 双拓扑，新增部署拓扑章节 [task:10-20-nginx-topologies] |

### Testing

- [OK] nginx -t 两段通过；curl 单条 Cache-Control；404 always；端口已关闭

### Status

[OK] **Completed**

### Next Steps

- 运维侧按占位符落地；nginx 保留安装但未自启


## Session 17: 核对残留任务
<!-- trellis-session: v=2 fp=cff0c6621021dc98 -->

**Date**: 2026-10-20
**Task**: 核对残留任务
**Branch**: `main`

### Summary

核对：Trellis 无其它活动任务；团队任务板 27 项全部 completed；teammate 全部 inactive；git 工作区干净且与 origin 同步

### Main Changes

- 无

### Git Commits

(No commits - planning session)

### Testing

- [OK] task.py list / team_task_list / list_agents / git status

### Status

[OK] **Completed**

### Next Steps

- 无


## Session 18: 本地全面测试验收 v0.0.23
<!-- trellis-session: v=2 fp=f5bb1b2861cb1e67 -->

**Date**: 2026-10-20
**Task**: 本地全面测试验收 v0.0.23
**Branch**: `main`

### Summary

A/B/C/E 全 PASS（90 测试、静态 15 页+robots、服务缓存/压缩/Range/穿越、包校验）；D 热更新 PASS；安全发现 1 项 FAIL：SITE_URL 未设置时合法 XFH 仍可改写 canonical/feed（本机 10086 与临时实例复现）

### Main Changes

- 无（纯测试）

### Git Commits

(No commits - planning session)

### Testing

- [OK] PASS/FAIL 表与 curl 证据在 .run/review/local-{ops,security}/

### Status

[OK] **Completed**

### Next Steps

- 决定修复方式：代码默认忽略 XFH 或本机设置 SITE_URL


## Session 19: 发布 v0.0.24 并上线本机
<!-- trellis-session: v=2 fp=f5967b10a7440d28 -->

**Date**: 2026-10-20
**Task**: 发布 v0.0.24 并上线本机
**Branch**: `main`

### Summary

安全补丁版 v0.0.24：默认不信任 X-Forwarded-*（SITE_URL 优先/NOBELOG_TRUST_FORWARDED 开关）；版本三源 0.0.24、包 sha256 c6cff2bc…、tag 与 GitHub Release 已推送；本机重启（PID 213193→216337，exe sha 6bd9317b…），XFH 矩阵与回归全 PASS

### Main Changes

- rust/Cargo.toml、package.json、rust/Cargo.lock

### Git Commits

| Hash | Message |
|------|---------|
| `d4215a6` | chore: 发布 v0.0.24（版本号同步） [task:10-20-release-24] |

### Testing

- [OK] cargo test 92；包 sha256sum -c；本机 XFH 矩阵 canonical 不被污染；9 路由 + 缓存回归

### Status

[OK] **Completed**

### Next Steps

- 远程部署由用户负责；反代场景需设 SITE_URL 或 NOBELOG_TRUST_FORWARDED=1
