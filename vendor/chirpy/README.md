# vendor/chirpy — halo-theme-chirpy 编译产物（vendored）

本目录的 CSS 与字体**不是**我们手写的样式，而是直接取用户自有站点 `airbozh.cn` 所用主题
（`halo-theme-chirpy`）的编译产物，按原字节入库（只重写资源路径前缀，见 §3）。
目的：让 nobelog 的构建产物与线上站点用**同一份样式表**，从而外观一致
（见 `.trellis/tasks/10-16-port-chirpy-theme/design.md` §1、§2 C1）。

## 1. 来源与抓取时间

| 项 | 值 |
| --- | --- |
| 来源站 | `https://www.airbozh.cn` |
| 资源路径 | `/themes/halo-theme-chirpy/assets/dist/<file>?v=1.4.9` |
| 抓取时间 | **2026-10-16 20:48:48 UTC** |
| 抓取方式 | `node scripts/fetch-theme-assets.mjs --force`（内部 `curl -sSL --compressed`，需带 `-L`） |
| 上游仓库 | <https://github.com/AirboZH/halo-theme-chirpy>（主题作者 **AirboZH**） |
| 版本参数 | `?v=1.4.9`（主题静态资源版本号，与站点 HTML 中引用的一致） |

**上游仓库核验**（本地只读 clone：`/root/博客/.ref/theme`，未联网）：

- remote：`https://github.com/AirboZH/halo-theme-chirpy.git`
- HEAD：commit `a224b2b0ebaadafe50cc23de619856311450922a`（2026-06-17，`chore: bump version to 1.5.0`），tag `v1.5.0`
- ⚠️ **版本错位**：clone 的 HEAD 是 **1.5.0**，而本目录入库的是站点当前部署的 `?v=1.4.9` 编译产物
  （该 clone 是浅克隆、没有 1.4.9 的 ref）。
- 上游**不提交**编译产物（`templates/assets/dist/**` 由 `pnpm build` → `vite build` 生成，不在 clone 里），
  因此本目录的字节无法从该 clone 重新推导，只能按上游 URL 重抓（见 §6）。
- 上游 `vite.config.js` 写了 `base: '/themes/halo-theme-chirpy/assets/dist/'` —— 这正是编译出来的
  `main.css` 用绝对路径引用字体的原因；§3 的路径重写是把它换成我们自己的 `/assets/` 布局。

入库文件**自带的**版权头（原样保留，未改动）标明其上游组成：

- `Chirpy v6.1.0` — <https://github.com/cotes2020/jekyll-theme-chirpy>，© 2019 Cotes Chung，MIT
- `Bootstrap v5.3.2` — Copyright 2011-2023 The Bootstrap Authors，MIT
- `Font Awesome Free 6.4.2` — © 2023 Fonticons, Inc.，Icons CC BY 4.0 / Fonts SIL OFL 1.1 / Code MIT
- 主题外壳（Halo 适配、配色、自定义块）来自 `halo-theme-chirpy`（AirboZH）

## 2. 文件清单

字节数与 gzip 均为实测（`gzip -9 -c | wc -c`）；SHA-256 是入库文件的哈希。

| 文件 | 字节数 | gzip -9 | SHA-256（入库） | SHA-256（上游原始） |
| --- | ---: | ---: | --- | --- |
| `main.css` | 415654 | 67330 | `75979a3e4ea75d2d2c4c45252ad6e11c82852241c70d00af972b7a17533ab90a` | `8e07870d0f11bd297da5ad5b02cc1e4c96892325175c2af2cf19286b49c168b9`（重写前，416254 B） |
| `fa-brands-400.woff2` | 109808 | 109846 | `faae6fc0aa94cc5bde5076647c817a23206096a1cbeda10d1c6f3d89d6163ed1` | 同左（未改动） |
| `fa-regular-400.woff2` | 24488 | 24507 | `9169d8be7a8177e5a92a4d04b6de7f6504b938573bf4da5889871c4f376d3849` | 同左（未改动） |
| `fa-solid-900.woff2` | 150020 | 150063 | `886c86112a804ef1ddd1cb206af4c8c40e34b73c26652ca231404aa35a6b30d9` | 同左（未改动） |
| `fa-v4compatibility.woff2` | 4568 | 4591 | `335fd7d9af58c44df12132fc9fd760b04af779b6d8580beb74ab5672f4526e77` | 同左（未改动） |
| **合计（5 个）** | **704538** | **356337** | | |

（`main.css` 重写前 gzip = 67360，重写后 67330：路径变短，压缩后省 30 B 左右。）

`main.js` **没有** vendored：它捆绑 jQuery/tocbot/clipboard/magnific-popup，且依赖 Halo 的 DOM 结构与插件 API，
交互由我们自己的 `src/client/main.ts` 按相同类名重写（design.md §2 C1）。

## 3. URL 重写（唯一改动）

主题编译产物用绝对路径 `/themes/halo-theme-chirpy/assets/dist/…` 引用字体，而我们的站点没有这个路径，
因此入库时把该前缀改写为 `/assets/`（构建侧会把字体拷到 `public/assets/`）：

```diff
- src:url(/themes/halo-theme-chirpy/assets/dist/fa-solid-900.woff2) format("woff2")
+ src:url(/assets/fa-solid-900.woff2) format("woff2")
- src:url(/themes/halo-theme-chirpy/assets/dist/fa-solid-900.ttf) format("truetype")
+ src:url(/assets/fa-solid-900.ttf) format("truetype")
```

**实测重写处数是 20 处，不是任务书里写的 4 处**：`main.css` 有 10 个 `@font-face`
（`Font Awesome 6 Brands/Free`、`Font Awesome 5 Brands/Free`、旧别名 `FontAwesome` 各一套），
每个 block 同时写 `woff2` + `ttf` 两个 `src`：

- `.woff2` 引用 10 处：`fa-brands-400` ×3、`fa-regular-400` ×3、`fa-solid-900` ×3、`fa-v4compatibility` ×1
- `.ttf` 引用 10 处：分布同上

除这 20 处前缀替换外**没有任何字节改动**（版权头原样保留）。脚本提供两种模式：

- `--rewrite=all`（默认，入库用的就是它）：20 处全改 → 文件里 `themes/halo-theme-chirpy` 出现 0 次；
- `--rewrite=woff2`：只改 10 处 `.woff2` 引用，保留 `.ttf` 的主题路径（对应"只改 4 处"的字面读法，
  但会让 `grep themes/halo-theme-chirpy` 命中 10 次）。

> **2026-10-16 Lead 裁定：保持 `--rewrite=all`（20 处全改、零残留）。** 理由：留下那 10 处指向
> `airbozh.cn` 的绝对路径，等于把字体变成**跨域依赖**（对方有 WAF，将来换域名/路径就全丢）；
> 改成 `/assets/fa-*.ttf` 即使文件不存在，失败也发生在我们自己的域内。

> ⚠️ **ttf 未 vendored，属有意为之**：本目录只入库 4 个 `.woff2`（见 §2），CSS 里那 10 处 `/assets/fa-*.ttf`
> 在构建产物里**不存在**。现代浏览器按 `src` 列表顺序只取第一个可用格式（woff2），**根本不会请求 ttf**；
> 只有不支持 woff2 的老引擎会拿到**本地 404**，随后按 CSS 规则**回退到下一个 `src`**。
> 若将来确实要兼容老引擎，再把这 4 个 `.ttf` 也 vendor 进来（或删掉 ttf 回退条目）即可。

## 4. 许可（**已裁定：采用 MIT** —— 2026-10-17 由站主定案）

核对方式（**本地只读 clone，未联网**）：`/root/博客/.ref/theme`
（remote `https://github.com/AirboZH/halo-theme-chirpy.git`，commit `a224b2b0ebaadafe50cc23de619856311450922a`）：

| 上游文件 | 内容 | 结论 |
| --- | --- | --- |
| `LICENSE`（1074 B，sha256 `20a19418738a87ea37f222aa4a577572cf35d05dbed0de4c305bc5a8a5e273fb`） | `The MIT License (MIT)` / `Copyright (c) 2023 AirboZH` | **MIT** |
| `package.json` 第 23 行 | `"license": "GPL-3.0"`（同文件 `"name": "halo-theme-chirpy"`、`"version": "1.5.0"`、`"author": { "name": "AirboZH" }`） | **GPL-3.0** |

**裁定（2026-10-17，站主）**：以仓库根的 **`LICENSE` 文件（MIT）**为授权依据 —— 它是许可的**授予文本**本身；
`package.json` 里的 `"license": "GPL-3.0"` 视为与该文件冲突的**元数据遗留**，不作为分发依据。
补充证据：编译产物 banner 明示上游血统为 **MIT**（`Chirpy v6.1.0 © 2019 Cotes Chung — MIT Licensed`，Bootstrap 亦 MIT）。

**残余风险（已接受）**：若上游作者后续明确认定该主题为 GPL-3.0，则本仓库需按 GPL 重新评估分发方式。
当前处于开发初期且为站主自用，按用户要求采用宽松许可（MIT）。
上文 §1 列出的内嵌第三方组件则各自遵循其自身许可：Chirpy MIT、Bootstrap MIT、
Font Awesome Free 6.4.2（Icons CC BY 4.0、Fonts SIL OFL 1.1、Code MIT）。

## 5. 构建侧集成要点（Lead 侧）

1. **字体必须按原名拷到 `public/assets/`**：`fa-brands-400.woff2` 等 4 个文件名**不能改**、不能加内容哈希，
   因为 `main.css` 里写死了 `/assets/fa-*.woff2`（design.md §4.2：vendored 资产不参与哈希）。
2. `main.css` 按 `STYLE_ORDER` 参与拼接（最终仍是单一 `main.<hash>.css`），它里面的字体 URL 保持 `/assets/…` 不变。
3. 旧的 `src/styles/{base,layout,post-list,article,syntax}.css` 与主题样式并存会互相打架（design.md §4.3）。

## 6. 复现与校验

```bash
node scripts/fetch-theme-assets.mjs              # 幂等：文件已存在且哈希匹配 → 跳过下载
node scripts/fetch-theme-assets.mjs --force      # 强制重新抓取（需要网络）
node scripts/fetch-theme-assets.mjs --verify-only
node scripts/fetch-theme-assets.mjs --spans      # 打印每一处重写的前后片段
node scripts/fetch-theme-assets.mjs --rewrite=woff2 --force   # 另一种重写模式（见 §3）
```

脚本内置指纹锚点（`RAW_PINS` / `OUT_PINS`）：上游字节若发生变化会**当场报错**而不是静默入库；
改造后的 `main.css` 也会与 `OUT_PINS.all` 比对，重写逻辑漂移同样会报错。

### WAF 说明

站点（阿里云 ESA）对静态资源也会下发 JS 挑战（`<html><script>var arg1=…`），裸 `curl` 会拿到挑战页或
被 `http_auto_ratelimit` 限流。过挑战的办法是用本机常驻浏览器取一次 cookie，再交给脚本：

```bash
bw open 'https://www.airbozh.cn/themes/halo-theme-chirpy/assets/dist/main.css?v=1.4.9'
bw eval 'document.cookie'      # 取 acw_sc__v2=…
node scripts/fetch-theme-assets.mjs --force --cookie 'acw_sc__v2=…; language=zh-CN'
```

（同一出口 IP，浏览器拿到的 cookie 可直接给 curl 用；脚本也支持 `--cookie-file` / `AIRBOZH_COOKIE` 环境变量。）

### 独立复核（不依赖脚本自身逻辑）

```bash
# 1) 主题路径必须为 0
grep -c 'themes/halo-theme-chirpy' vendor/chirpy/main.css        # → 0

# 2) 本地字体引用 20 处（grep -c 数的是行数，全在一行 → 1；用 -o 数出现次数）
grep -o 'url(/assets/fa-[a-z0-9.-]*' vendor/chirpy/main.css | sort | uniq -c

# 3) 只有前缀被改：用 sed 重演替换，应与入库文件逐字节相同
curl -sSL --compressed 'https://www.airbozh.cn/themes/halo-theme-chirpy/assets/dist/main.css?v=1.4.9' \
  | sed 's|/themes/halo-theme-chirpy/assets/dist/|/assets/|g' | cmp - vendor/chirpy/main.css

# 4) 哈希
sha256sum vendor/chirpy/main.css vendor/chirpy/*.woff2
```

2026-10-16 入库时上述复核全部通过（`cmp` 无差异、`grep -c` = 0、5 个文件哈希与 §2 表格一致）。
