# Research: 博客同步状态与运行现状只读核查

- Query: 本地工作区与 GitHub 远程是否对得上；博客当前具体情况
- Scope: mixed，只读覆盖本地 git 仓库、本地 systemd 服务与 10086 端口、GitHub `zhemed/nobelog`
- Date: 2026-10-30
- 核查时刻: 2026-10-30T11:06Z 前后，全部命令在仓库根 `/root/博客/nobelog` 执行
- 约束: 全程只读；未 fetch，未 commit/push/tag，未重启服务，未访问公网博客域名，未安装任何东西

## 一、结论摘要

1. 工作区与 GitHub 远程：不完全一致。本地 `main` 领先 `origin/main` 5 个提交、落后 0 个，这 5 个提交尚未推送。未推送内容全部位于 `.trellis/`，不含 `rust/`、`content/`、`vendor/`、`tools/` 改动，因此代码与内容层面与远程一致。
2. 标签与 Release：本地标签、远程标签、GitHub Releases 三方数量均为 29，名称集合与 tag 对象哈希逐条一致；最新 Release 为 v0.0.29，不是草稿也不是预发布。
3. 发布包：本地 `dist/` 只剩 v0.0.29 包与 `SHA256SUMS`，两者的字节数与 sha256 均与 GitHub Release v0.0.29 资产逐字节相同；包内二进制的 sha256 与当前运行二进制一致。
4. 运行现状：`nobelog-local.service` 自 2026-10-21T15:30:36Z 起持续运行，主进程 PID 797659，动态模式，监听 0.0.0.0:10086，二进制版本 0.0.29。v0.0.29 之后 6 个提交未改任何源码或内容，运行二进制与当前 HEAD 源码等价。
5. 内容与配置：2 篇文章、1 个页面、8 张图片；站点标题 吱吱、副标题 冷知识；`SITE_URL` 未设置，绝对链接回退 `http://127.0.0.1:10086`；Rust 工具链固定 1.99.1；Trellis 归档任务 39 个、活跃任务 1 个、journal 39 个会话。

一句话结论：**代码、内容、标签、Release、发布包与运行二进制互相对得上；唯一不同步的是本地 10-30 的 5 个 Trellis 文档与归档提交尚未推到 GitHub。**

## 二、工作区与 GitHub 对账

### 1. 本地 git 事实

| 项目 | 值 |
|---|---|
| 仓库根 | `/root/博客/nobelog` |
| 当前分支 | `main`，唯一分支，无 stash，无其它本地分支 |
| HEAD | `44a5c5d97e6bb543563f8790ce8f15fb0d43bff7` |
| HEAD 提交时间 | 2026-10-30T10:51:28Z |
| HEAD 主题 | `chore: 归档图标回退任务并记录 journal [task:10-30-revert-bear-test]` |
| 远程配置 | `origin` 指向 `https://github.com/zhemed/nobelog.git`，fetch 与 push 同址 |
| 跟踪关系 | `branch.main.remote=origin`，`branch.main.merge=refs/heads/main` |
| 工作区状态 | 仅一个未跟踪目录 `.trellis/tasks/10-30-audit-sync-github/`，无已跟踪文件改动 |

证据命令与关键输出：

```bash
$ git status --short --branch
## main...origin/main [ahead 5]
?? .trellis/tasks/10-30-audit-sync-github/

$ git status --porcelain=v2
? .trellis/tasks/10-30-audit-sync-github/

$ git diff --stat; git diff --cached --stat
# 两条命令均无输出，说明暂存区与工作区都没有已跟踪文件改动

$ git branch -vv
* main 44a5c5d [origin/main: ahead 5] chore: 归档图标回退任务并记录 journal [task:10-30-revert-bear-test]
```

判定：`ahead 5` 与远程 head 判定一致；唯一未跟踪目录是本次只读核查任务自己的任务目录，属预期。工作区在已跟踪文件意义上干净。

### 2. 远程 ref 与包含关系

| 项目 | 值 |
|---|---|
| 远程 `refs/heads/main` | `28f80ea8887c51ec7b13f52bc37aaa955c5fb972` |
| 远程 `HEAD` | 同一哈希 |
| 远程 head 本地对象 | 存在，`git cat-file -e` 成功，无需 fetch |
| 包含关系 | 远程 head 是本地 HEAD 祖先，`git merge-base --is-ancestor` 成功 |
| ahead / behind | 5 / 0 |
| `git describe --tags --always` | `v0.0.29-6-g44a5c5d` |

```bash
$ git ls-remote origin
28f80ea8887c51ec7b13f52bc37aaa955c5fb972	HEAD
28f80ea8887c51ec7b13f52bc37aaa955c5fb972	refs/heads/main
# 其余为 29 个 tag 的 ref 行与 peeled 行，摘录见第三节

$ git rev-list --count 28f80ea..HEAD
5
$ git rev-list --count HEAD..28f80ea
0
$ git merge-base --is-ancestor 28f80ea HEAD && echo ancestor
ancestor
```

未推送的 5 个提交，`git log --oneline 28f80ea..HEAD`：

```
44a5c5d chore: 归档图标回退任务并记录 journal [task:10-30-revert-bear-test]
af21ccd revert: 回退小熊线稿测试图标，恢复粉笔少女图标 [task:10-30-revert-bear-test]
8da1a69 chore: 归档小熊图标任务并记录 journal [task:10-30-icon-bear-lineart]
9157153 docs: 记录构建输出目录与 SITE_URL 验证坑，修正图标产物命名 [task:10-30-icon-bear-lineart]
b07a0cd feat: 切换站点图标为小熊线稿 [task:10-30-icon-bear-lineart]
```

净差异：

```bash
$ git diff --stat origin/main..HEAD
 .trellis/spec/frontend/build-serve-deploy.md       | 12 ++-
 .trellis/spec/frontend/directory-structure.md      |  2 +-
 .../archive/2026-10/10-30-icon-bear-lineart/*      | 归档任务 5 个文件
 .../archive/2026-10/10-30-revert-bear-test/*       | 归档任务 5 个文件
 .trellis/workspace/zhemed/index.md                 |  8 +-
 .trellis/workspace/zhemed/journal-1.md             | 93 ++++++++++++++++++++++
 14 files changed, 261 insertions(+), 6 deletions(-)
```

共 14 个文件、+261/-6，全部在 `.trellis/` 下。图标提交 `b07a0cd` 与回退提交 `af21ccd` 在净差异中抵消，`git diff --name-status origin/main..HEAD` 没有出现任何 `content/` 或 `rust/` 文件。

判定：远程没有本地缺失的提交，本地有 5 个未推送提交。远程 head 对象在本地存在，本次没有也不需要 fetch 就能判定包含关系，实际也未执行 fetch。

### 3. 远程活跃度旁证

- `gh repo view --json pushedAt` 返回 `2026-10-21T15:31:40Z`。
- `origin/main` 提交时间 2026-10-21T15:31:38Z，主题 `chore: 归档 v0.0.29 发布任务并记录 journal [task:10-21-release-29]`。
- 远程自 2026-10-21 之后无新推送；本地 2026-10-30 的 5 个提交尚未到达 GitHub。

## 三、标签与 GitHub Release 三方对账

| 对账项 | 本地 | 远程 | GitHub Releases | 判定 |
|---|---|---|---|---|
| 数量 | 29 | 29 | 29 | 一致 |
| 名称集合 | v0.0.1 至 v0.0.29 | 同 | 同 | 一致 |
| annotated tag 对象哈希 | 逐条 | 逐条 | 跟随 tag | 逐条一致 |
| peeled commit 哈希 | 逐条 | 逐条 | 跟随 tag | 逐条一致 |
| 最新版本 | v0.0.29 | v0.0.29 | v0.0.29 | 一致 |

逐条比对：

```bash
$ diff <(git ls-remote --tags origin | awk '{print $1" "$2}' | sort -k2) \
       <(for r in $(git for-each-ref --format='%(refname)' refs/tags); do
           echo "$(git rev-parse "$r") $r"
           peeled=$(git rev-parse "$r^{}")
           [ "$peeled" != "$(git rev-parse "$r")" ] && echo "$peeled $r^{}"
         done | sort -k2)
# 无输出，tag 集合与全部对象哈希完全一致

$ git tag | wc -l
29
$ git ls-remote --tags origin | grep -vc '\^{}'
29
$ gh api 'repos/zhemed/nobelog/releases?per_page=100' --jq 'length'
29
$ comm -3 <(git tag | sort) \
          <(gh api 'repos/zhemed/nobelog/releases?per_page=100' --jq '.[].tag_name' | sort)
# 无输出，tag 与 release tagName 集合完全相同
```

最新 Release 详情：

```json
{
  "tagName": "v0.0.29",
  "name": "v0.0.29：去 Node 化收尾",
  "isDraft": false,
  "isPrerelease": false,
  "isLatest": true,
  "publishedAt": "2026-10-21T15:30:03Z",
  "url": "https://github.com/zhemed/nobelog/releases/tag/v0.0.29"
}
```

v0.0.29 的提交链路：

```text
annotated tag v0.0.29 对象: 4d98ce127362b78e6fe975e0dc364becad425636
tag message:              0.0.29：去 Node 化收尾
指向 commit:              7899d5a51be5d4231a173f244ec8177e5658b206
commit 时间与主题:        2026-10-21T15:29:46Z  chore: 发布 v0.0.29（版本号同步，两源） [task:10-21-release-29]
Release target_commitish: main，GitHub 侧 tag ref 指向同一个 tag 对象 4d98ce1
```

补充事实：

- 仓库 `zhemed/nobelog` 可见性为 PUBLIC，默认分支 `main`，url 为 `https://github.com/zhemed/nobelog`。
- Release 资产数量历史分布：v0.0.29 至 v0.0.3 每版 2 个资产；v0.0.2 有 3 个资产；v0.0.1 无资产。最新版资产完整，历史差异不影响本次对账。
- `git tag -n` 与 tag 对象内容确认本地 29 个 tag 均为 annotated tag。

判定：标签、Release、远程 ref 三方对齐，无缺版、无多版、无同名不同对象。

## 四、发布包对账

本地 `dist/` 只有 2 个文件，均为 `.gitignore` 第 15 行 `dist/` 忽略，不进入版本控制：

```bash
$ ls -la dist/
-rw-r--r-- 1 root root     101 Oct 30 10:20 SHA256SUMS
-rw-r--r-- 1 root root 4900595 Oct 30 10:20 nobelog-0.0.29-linux-x86_64.tar.gz

$ cat dist/SHA256SUMS
ed3fe2e5cae03c984969fdc7fff922650c2b635236276101ee2cae303a6cd40d  nobelog-0.0.29-linux-x86_64.tar.gz

$ (cd dist && sha256sum -c SHA256SUMS)
nobelog-0.0.29-linux-x86_64.tar.gz: OK
```

与 GitHub Release v0.0.29 资产逐项比对：

| 对象 | 本地 | 远程 Release 资产 | 判定 |
|---|---|---|---|
| 压缩包名 | `nobelog-0.0.29-linux-x86_64.tar.gz` | 同名 | 一致 |
| 压缩包字节数 | 4900595 | 4900595 | 一致 |
| 压缩包 sha256 | `ed3fe2e5cae03c984969fdc7fff922650c2b635236276101ee2cae303a6cd40d` | `sha256:ed3fe2e5cae03c984969fdc7fff922650c2b635236276101ee2cae303a6cd40d` | 逐字节一致 |
| SHA256SUMS 字节数 | 101 | 101 | 一致 |
| SHA256SUMS sha256 | `51a58acf6b4134f8039bfc5de9e48c521585638279c46a706e50ec8cba44a400` | `sha256:51a58acf6b4134f8039bfc5de9e48c521585638279c46a706e50ec8cba44a400` | 逐字节一致 |

```bash
$ gh api repos/zhemed/nobelog/releases/tags/v0.0.29 \
    --jq '.assets[] | [.name, (.size|tostring), .digest] | @tsv'
nobelog-0.0.29-linux-x86_64.tar.gz	4900595	sha256:ed3fe2e5cae03c984969fdc7fff922650c2b635236276101ee2cae303a6cd40d
SHA256SUMS	101	sha256:51a58acf6b4134f8039bfc5de9e48c521585638279c46a706e50ec8cba44a400
```

本地文件 mtime 是 2026-10-30 10:20，但内容哈希与 10-21 的 Release 资产完全一致，说明 10-30 恢复的是同一份字节，不是重新打包。

包内结构抽查，共 110 个成员：

| 包内对象 | 值与判定 |
|---|---|
| `VERSION` | `0.0.29`，与 tag、Cargo 版本一致 |
| `nobelog-0.0.29-linux-x86_64/nobelog` sha256 | `976fee0fe799fffce44ad31dfd41e585e27646cf643e169e0ea5e569ddbc9204`，与运行二进制一致 |
| `content/` 12 个文件 | 与当前工作区逐个 sha256 相同，全部 SAME |
| `rust/client/main.js` | `134958bf956c42079518196643cbecb77018376a243996074c5e86c8a893fde8`，与工作区相同 |
| `public/index.html` canonical | `https://example.com/`，发布包是用真实站点域名构建的 |
| 必备成员 | 二进制、`content/`、`vendor/`、`public/`、`rust/client/main.js`、README、LICENSE、VERSION 齐全 |

```bash
$ tar -xzOf dist/nobelog-0.0.29-linux-x86_64.tar.gz \
    nobelog-0.0.29-linux-x86_64/nobelog | sha256sum
976fee0fe799fffce44ad31dfd41e585e27646cf643e169e0ea5e569ddbc9204  -
```

判定：本地发布包、tag v0.0.29、GitHub Release v0.0.29 三者互相对应，本地包与远程资产逐字节一致。历史包 v0.0.1 至 v0.0.28 在本地 `dist/` 中已不存在，这是 PRD 中记载的 10-30 事故后按用户决定只恢复 v0.0.29 的结果；远程 Release 各版资产仍在，可随时重新下载。

## 五、运行现状

### 1. 服务单元

| 项目 | 值 |
|---|---|
| 单元文件 | `/etc/systemd/system/nobelog-local.service`，enabled |
| 当前状态 | active running，SubState=running |
| 启动时刻 | 2026-10-21T15:30:36Z |
| 运行时长 | 约 8 天 19 小时，至核查时刻未重启，NRestarts=0 |
| MainPID | 797659，唯一 nobelog 进程，无多实例 |
| ExecStart | `/root/博客/nobelog/rust/target/release/nobelog serve --host 0.0.0.0 --port 10086 --root /root/博客/nobelog --dynamic` |
| WorkingDirectory | `/root/博客/nobelog` |
| Environment | 空，未设置 `SITE_URL` 或 `NOBELOG_TRUST_FORWARDED` |
| 资源限制 | MemoryMax=512M，当前用量约 28.3 MiB，Tasks=33 |
| 加固项 | NoNewPrivileges=true、PrivateTmp=true、ProtectSystem=strict、ReadOnlyPaths=/root/博客/nobelog |

```text
$ systemctl status nobelog-local --no-pager
● nobelog-local.service - nobelog blog (local, dynamic mode)
     Loaded: loaded (/etc/systemd/system/nobelog-local.service; enabled; vendor preset: enabled)
     Active: active (running) since Wed 2026-10-21 15:30:36 UTC; 1 week 1 day ago
   Main PID: 797659 (nobelog)
      Tasks: 33 (limit: 14254)
     Memory: 28.2M (max: 512.0M available: 483.7M)
        CPU: 223ms
             └─797659 /root/博客/nobelog/rust/target/release/nobelog serve --host 0.0.0.0 --port 10086 --root /root/博客/nobelog --dynamic

$ systemctl show nobelog-local -p NRestarts,MemoryCurrent,MemoryMax
NRestarts=0
MemoryCurrent=29663232
MemoryMax=536870912
```

日志显示上一次重启是 2026-10-21T15:30:36Z 的 v0.0.29 上线动作，之后连续运行：

```text
Oct 21 15:30:36 ubuntu systemd[1]: Stopping nobelog blog (local, dynamic mode)...
Oct 21 15:30:36 ubuntu systemd[1]: Started nobelog blog (local, dynamic mode).
Oct 21 15:30:36 ubuntu nobelog[797659]: 警告：未设置 SITE_URL，站点绝对链接回退到内置默认地址 http://127.0.0.1:10086；...
Oct 21 15:30:36 ubuntu nobelog[797659]: nobelog serve  http://0.0.0.0:10086  [dynamic]
```

### 2. 运行二进制与来源

| 对象 | 路径 | sha256 | 版本 |
|---|---|---|---|
| 运行进程可执行文件 | `/proc/797659/exe` 指向 `/root/博客/nobelog/rust/target/release/nobelog` | `976fee0fe799fffce44ad31dfd41e585e27646cf643e169e0ea5e569ddbc9204` | nobelog 0.0.29 |
| 磁盘上的 release 二进制 | `rust/target/release/nobelog`，mtime 2026-10-21 15:28 | 同上 | nobelog 0.0.29 |
| v0.0.29 发布包内二进制 | `dist/nobelog-0.0.29-linux-x86_64.tar.gz` 内 `nobelog` | 同上 | 0.0.29 |

```bash
$ sha256sum rust/target/release/nobelog /proc/797659/exe
976fee0fe799fffce44ad31dfd41e585e27646cf643e169e0ea5e569ddbc9204  rust/target/release/nobelog
976fee0fe799fffce44ad31dfd41e585e27646cf643e169e0ea5e569ddbc9204  /proc/797659/exe
$ ./rust/target/release/nobelog --version
nobelog 0.0.29
```

运行二进制与源码 HEAD 的关系，用提交差异判定：

```bash
$ git rev-list --count v0.0.29..HEAD
6
$ git diff --name-status v0.0.29..HEAD -- . ':!.trellis'
# 无输出，v0.0.29 之后 6 个提交只改 .trellis，没有改 rust/content/vendor/tools
$ git diff --name-status v0.0.29..HEAD
# 19 个文件全部在 .trellis/ 下，含 2 个 spec、3 个归档任务目录、workspace index 与 journal
```

判定：运行进程执行的二进制就是 v0.0.29 发布包内的二进制；v0.0.29 之后 6 个提交未触及任何源码或内容文件，所以运行二进制与当前 HEAD 的 Rust 源码等价，无需为 10-30 的图标测试与回退重新构建。该判定基于包哈希与 diff 的推断，二进制本身未嵌入 commit hash，无法用版本命令直接证明来源提交。

### 3. 只读探活

`ss -lntp` 显示唯一监听 `0.0.0.0:10086`，进程 PID 797659，绑定全部网卡：

```text
LISTEN 0 128 0.0.0.0:10086 0.0.0.0:* users:(("nobelog",pid=797659,fd=9))
```

本地只读探活结果：

| 路径 | 状态码 | 字节数 | 备注 |
|---|---|---|---|
| `/` | 200 | 9205 | 动态渲染，`cache-control: no-cache` |
| `/robots.txt` | 404 | 7574 | 动态模式没有该路由，静态产物 `public/robots.txt` 才存在 |
| `/sitemap.xml` | 200 | 954 | 动态路由 |
| `/feed.xml` | 200 | 1331 | 动态路由 |
| `/assets/main.1ca5c955.css` | 200 | 192311 | 内存资源，哈希文件名 |
| `/assets/main.ea281e5e.js` | 200 | 12082 | 内存资源 |
| `/images/site-icon.svg` | 200 | 88502 | sha256 与 `content/images`、`public/images` 相同 |
| `/images/site-icon-32-transparent.png` | 200 | 2089 | 图标兜底图 |
| `/about/` | 200 | 8596 | 页面 |
| `/archives/` | 200 | 7957 | 页面 |
| `/tags/` | 200 | 8225 | 页面 |
| `/categories/` | 200 | 9071 | 页面 |
| `/__nonexistent__` | 404 | 7574 | 404 页正常 |

绝对地址行为，属于 `SITE_URL` 现状的直接证据：

| 请求 | canonical 输出 | 说明 |
|---|---|---|
| 默认 `Host: 127.0.0.1:10086` | `http://127.0.0.1:10086/` | 回退内置默认地址 |
| 自定义 `Host: blog.example.com` | `http://blog.example.com/` | 动态模式按 Host 推断，scheme 固定 http |
| 伪造 `X-Forwarded-Host: evil.example` 与 `X-Forwarded-Proto: https` | `http://127.0.0.1:10086/` | 默认不信任 XFH，与 spec 描述一致 |

运行态首页与冻结基准 `public/index.html` 的差异只有两类：4 处域名不同，即 canonical、og:url、og:image、页脚站点链接；2 处相对日期文案不同，即 `12 天前` 对 `4 天前` 一类随构建时刻漂移的文案。正文、资源哈希、图标引用一致。

## 六、内容与配置

| 项目 | 值 | 证据 |
|---|---|---|
| 文章数 | 2 | `content/posts/server-init-commands.md`、`content/posts/typeset-test.md` |
| 页面数 | 1 | `content/pages/about.md` |
| 图片数 | 8 | `site-icon-*` 7 个加 `test-cover.svg` |
| 站点标题 | 吱吱 | `rust/src/site.rs` 的 `SITE.title` |
| 站点副标题 | 冷知识 | `SITE.subtitle` |
| 站点描述 | 吱吱的冷知识：记录有用没用的小知识。 | `SITE.description` |
| 内置站点地址 | `http://127.0.0.1:10086` | `SITE.url` |
| `SITE_URL` | 未设置 | systemd `Environment=` 为空，启动日志有告警 |
| 真实站点域名 | `https://example.com` | 仅出现在冻结基准 `public/` 与发布包产物中 |
| Cargo 版本 | 0.0.29 | `rust/Cargo.toml` 第 3 行、`rust/Cargo.lock` 第 765 至 766 行 |
| Rust 工具链 | 1.99.1，含 rustfmt 与 clippy，profile minimal | `rust/rust-toolchain.toml` |
| `public/` | 42 个文件，构建于 2026-10-21 15:28，被 `.gitignore` 第 13 行忽略 | 与发布包内 `public/` 逐字节一致，`index.html` 与 `feed.xml` 哈希相同 |
| `rust/dist/` | 42 个文件，构建于 2026-10-30 10:11 与 10:19，被 `.gitignore` 第 3 行忽略 | 小熊图标测试期的过期构建，图标与当前内容不符 |
| `dist/` | 2 个文件，被 `.gitignore` 第 15 行忽略 | 见第四节 |
| `.run/` | 仅空目录 `review/`，被 `.gitignore` 第 9 行忽略 | 无残留文件 |
| 仓库根 | 无 `.github/` 目录，未发现 CI 配置 | 根目录清单 |
| GitHub 可见性 | PUBLIC | `gh repo view` |

Trellis 状态：

| 项目 | 值 |
|---|---|
| 活跃任务 | 1 个，`.trellis/tasks/10-30-audit-sync-github/` |
| 归档任务 | 39 个，全部位于 `.trellis/tasks/archive/2026-10/` |
| journal 会话数 | 39，`journal-1.md` 有 39 个 `## Session` 标题，`index.md` 记录 Total Sessions 39 |
| 已跟踪文件 | 365 个，其中 `.trellis/` 246 个、`content/` 12 个、`rust/` 23 个 |
| 活跃任务目录状态 | 未跟踪，含 prd.md、implement.jsonl、check.jsonl、directive.md、task.json，本次新增 research/ 报告 |

```bash
$ find .trellis/tasks/archive -mindepth 2 -maxdepth 2 -type d | wc -l
39
$ grep -c '^## ' .trellis/workspace/zhemed/journal-1.md
39
$ git ls-files | wc -l
365
```

`public/` 与当前源码的一致性：`diff -rq public rust/dist` 只在 9 个文件上不同，其中 7 个图标来自过期的小熊构建，`index.html` 与 `feed.xml` 的差异仅为相对日期文案与 `lastBuildDate`。`public/` 的图标哈希与 `content/images` 完全一致，而 `public/index.html` 与 `public/feed.xml` 又与发布包内同名文件哈希一致。判定：`public/` 是 v0.0.29 的冻结基准且与当前内容匹配；`rust/dist/` 才是过期产物。

## 七、偏差与风险

1. **5 个提交未推送**：本地领先 `origin/main` 5 个提交，GitHub 上的 `main` 仍停在 28f80ea。内容全是 Trellis 文档、归档与 journal，不含代码，但本地磁盘故障会丢失这 5 条历史。是否需要推送由 Lead 与用户决定。
2. **`SITE_URL` 未设置**：canonical、og:url、og:image、RSS、sitemap 的绝对地址回退 `http://127.0.0.1:10086`，或按请求 Host 推断且 scheme 固定 http。服务绑定 `0.0.0.0:10086`，若端口对局域网或公网可达，抓取器与分享卡片会收到错误域名。生产正确值应为 `https://example.com`，发布包与 `public/` 基准都用它构建。
3. **动态模式 `/robots.txt` 返回 404**：robots.txt 由静态构建产出，当前 10086 是动态模式，没有该路由；`/sitemap.xml` 与 `/feed.xml` 则有动态路由。若该端口被外部抓取，robots 规则不会生效。
4. **本地历史发布包缺失**：`dist/` 只剩 v0.0.29 包与 SHA256SUMS，v0.0.1 至 v0.0.28 的压缩包本地不存在；远程 Release 资产齐全，需要时从 GitHub 重新下载即可。该状态与 PRD 记载的 10-30 事故后恢复决定一致。
5. **`rust/dist/` 是过期构建**：2026-10-30 10:11 构建于小熊图标测试期，图标文件与当前 `content/images` 不同。它被 git 忽略，不会提交，但若有人误把它当部署产物，会看到错误图标。当前正确产物是 `public/`，动态服务则直接读 `content/`。
6. **工作区存在未跟踪目录**：`.trellis/tasks/10-30-audit-sync-github/` 未跟踪，因此 `git status` 不是全干净；属本任务预期，归档并提交后消除。
7. **版本语义不等于历史相同**：tag v0.0.29 指向发布提交 7899d5a，远程 main 是其后的归档提交 28f80ea，本地又在 28f80ea 之上 5 个提交。三方版本号一致，但 Git 历史并不相同。
8. **只读核查边界**：未访问 `example.com` 公网域名，无法验证线上部署、DNS、TLS、CDN 与远端 nginx 配置；未 fetch，远程状态只代表 `git ls-remote` 查询时刻；运行二进制与源码的关系由包哈希与 diff 推断，不是二进制内嵌证明。

## 八、证据命令与原始输出摘录

以下命令均在 `/root/博客/nobelog` 执行，全部只读。前文已引用的输出不重复全文，这里补充关键原始片段。

### 1. 版本与同步

```bash
$ git log -1 --format='%H %cd %s' --date=iso-strict
44a5c5d97e6bb543563f8790ce8f15fb0d43bff7 2026-10-30T10:51:28+00:00 chore: 归档图标回退任务并记录 journal [task:10-30-revert-bear-test]

$ git log -1 --format='%H %ci %s' origin/main
28f80ea8887c51ec7b13f52bc37aaa955c5fb972 2026-10-21 15:31:38 +0000 chore: 归档 v0.0.29 发布任务并记录 journal [task:10-21-release-29]

$ git describe --tags --always
v0.0.29-6-g44a5c5d

$ git remote -v
origin	https://github.com/zhemed/nobelog.git (fetch)
origin	https://github.com/zhemed/nobelog.git (push)
```

### 2. 远程与 Release

```bash
$ gh repo view zhemed/nobelog --json nameWithOwner,defaultBranchRef,pushedAt,visibility,url
{"defaultBranchRef":{"name":"main"},"nameWithOwner":"zhemed/nobelog","pushedAt":"2026-10-21T15:31:40Z","url":"https://github.com/zhemed/nobelog","visibility":"PUBLIC"}

$ gh release list -R zhemed/nobelog -L 40 --json tagName,name,isLatest,isDraft,isPrerelease,publishedAt
[{"isDraft":false,"isLatest":true,"isPrerelease":false,"name":"v0.0.29：去 Node 化收尾","publishedAt":"2026-10-21T15:30:03Z","tagName":"v0.0.29"},
 ..., {"isDraft":false,"isLatest":false,"isPrerelease":false,"name":"v0.0.1 稳定版（对照标本）","publishedAt":"2026-10-17T15:09:02Z","tagName":"v0.0.1"}]
```

### 3. 服务与二进制

```bash
$ ps -o pid,lstart,etime,cmd -p 797659
    PID                  STARTED     ELAPSED CMD
 797659 Wed Oct 21 15:30:36 2026  8-19:33:46 /root/博客/nobelog/rust/target/release/nobelog serve --host 0.0.0.0 --port 10086 --root /root/博客/nobelog --dynamic

$ readlink -f /proc/797659/exe
/root/博客/nobelog/rust/target/release/nobelog

$ systemctl show nobelog-local -p ActiveEnterTimestamp,ExecMainPID,WorkingDirectory,NRestarts
ActiveEnterTimestamp=Wed 2026-10-21 15:30:36 UTC
ExecMainPID=797659
WorkingDirectory=/root/博客/nobelog
NRestarts=0
```

### 4. 内容与站点配置

```bash
$ sed -n '60,110p' rust/src/site.rs
pub fn env_url() -> Option<String> { ... }
pub static SITE: SiteConfig = SiteConfig {
    url: "http://127.0.0.1:10086",
    lang: "zh-CN",
    feed_lang: "zh-CN",
    title: "吱吱",
    subtitle: "冷知识",
    description: "吱吱的冷知识：记录有用没用的小知识。",
    author: "",
    ...
};

$ grep -n '^version' rust/Cargo.toml
3:version = "0.0.29"
$ grep -n -A2 'name = "nobelog"' rust/Cargo.lock
765:name = "nobelog"
766-version = "0.0.29"
```

### 5. 冻结基准与产物

```bash
$ grep -o 'rel="canonical" href="[^"]*"' public/index.html
rel="canonical" href="https://example.com/"
$ cat public/robots.txt
# Generated by nobelog — 手写请删除本行
User-agent: *
Allow: /

Sitemap: https://example.com/sitemap.xml

$ diff -rq public rust/dist
Files public/feed.xml and rust/dist/feed.xml differ
Files public/images/site-icon-180.png and rust/dist/images/site-icon-180.png differ
Files public/images/site-icon-192-dark.png and rust/dist/images/site-icon-192-dark.png differ
Files public/images/site-icon-192-transparent.png and rust/dist/images/site-icon-192-transparent.png differ
Files public/images/site-icon-32-dark.png and rust/dist/images/site-icon-32-dark.png differ
Files public/images/site-icon-32-transparent.png and rust/dist/images/site-icon-32-transparent.png differ
Files public/images/site-icon-512.png and rust/dist/images/site-icon-512.png differ
Files public/images/site-icon.svg and rust/dist/images/site-icon.svg differ
Files public/index.html and rust/dist/index.html differ

$ diff public/index.html rust/dist/index.html
114c114
<             <em class="">4 天前</em>
---
>             <em class="">12 天前</em>
139c139
<             <em class="">3 天前</em>
---
>             <em class="">12 天前</em>
```

### 6. 忽略规则

```bash
$ git check-ignore -v --no-index \
    dist/nobelog-0.0.29-linux-x86_64.tar.gz public/index.html \
    rust/target/release/nobelog .run/review
.gitignore:15:dist/	dist/nobelog-0.0.29-linux-x86_64.tar.gz
.gitignore:13:public/	public/index.html
.gitignore:2:rust/target/	rust/target/release/nobelog
.gitignore:9:.run/	.run/review
```

## 九、未覆盖项与不确定性

1. 线上 `example.com` 未访问，属未授权公网域名；线上部署、真实 robots、DNS、TLS、CDN、远端 nginx 拓扑均未验证。
2. 未执行 `git fetch`，远程状态仅代表 `git ls-remote` 与 `gh` 查询时刻；远程 head 对象本地已存在，本次判定不依赖 fetch。
3. 运行二进制与源码提交的关系是推断：包内二进制哈希、运行进程哈希、`rust/target` 文件哈希三者相同；v0.0.29 之后 diff 无源码或内容变化。二进制未嵌入 commit hash，无法给出密码学级来源证明。
4. 历史发布包 v0.0.1 至 v0.0.28 本地已不存在，无法逐包比对本地与远程；只能确认远程资产齐全、v0.0.29 逐字节一致。
5. 本机未发现 `rustup` 命令，PATH 中无 rustup；`rust-toolchain.toml` 固定 1.99.1 的描述来自文件本身，未运行 `cargo` 或 `rustc` 以免触发构建写入或工具链下载。
6. 未检查远端 DNS 记录、防火墙状态与 10086 端口的对外可达性，因此 `SITE_URL` 风险的实际暴露面未知。
7. 工具链与依赖锁文件未做 `cargo test` 或构建验证，属只读核查范围之外，且构建会写入 `rust/target` 与 `rust/dist`。
