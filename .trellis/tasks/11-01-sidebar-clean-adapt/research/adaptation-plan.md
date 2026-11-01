# 方案 B 适配调研与决策记录

调研：r2-analyst 只读取证 + Lead 复核，日期 2026-11-01。线上现状由常驻浏览器计算样式取证，源码以 nobelog 仓库为准。

## 1 现有实现

- 侧栏 HTML：`rust/src/templates.rs::sidebar()` 241-294 行生成；标记为 `aside#sidebar > header.profile-wrapper > h1.site-title > a`、`p.site-subtitle`、`nav > ul.nav > li.nav-item > a.nav-link > i + span`、`div.sidebar-bottom`。
- 副标题：`site.rs` 68 行为「冷知识」，`templates.rs::brand_subtitle()` 229-239 行逐字插入 `span.brand-dot`，渲染成「冷 · 知 · 识」。
- 样式装配：`paths.rs::STYLE_ORDER` 依次 chirpy、fonts、github-markdown、patch；patch 最后加载，天然覆盖。
- 侧栏布局：`main.css` 第 18 行 `#sidebar` 固定定位、宽 260px、`background:var(--sidebar-bg)`；≥1650px 改 300px；≤849px 抽屉位移 260px；≥850px 主栏 margin-left 260px。
- 侧栏表面：`patch.css` 385-387 行加 1px 右边框；`--sidebar-bg` 在 892、920 行映射到 `--paper-deep`，亮色 #f7f8fa、显式暗色 #101317。
- 标题区最终级联：patch.css 819-873 行与 944-976 行两段品牌块，后段覆盖前段；最终 `.site-title` 1.6rem/600/0.16em，`.site-subtitle` 1.75rem/700/0.02em，副标题比标题更大。
- 装饰短杠：`#sidebar .profile-wrapper::after` 848-855 行 36×1、渐变，973-976 行左右 auto 居中；线上实测亮色为 rgba(158,61,63,.55) 到透明的渐变。
- 线上计算样式复查：侧栏 260px、底色 rgb(247,248,250)、右边框 1px rgb(239,239,239)；标题 25.6px/600/4.096px；副标题 28px/700/0.56px；短杠 36×1 渐变、margin-left auto。

## 2 选择器映射与采用值

| 方案 B | 现有选择器 | 采用值 |
| --- | --- | --- |
| `.sidebar-clean` 背景与边框 | `#sidebar` | `background: var(--paper)`、`border-right: 0` |
| `.title-main` | `#sidebar .site-title` | 22px、800、letter-spacing 4px、`var(--ink)`、text-indent 4px |
| `.title-sub` | `#sidebar .site-subtitle` | margin-top 6px、12px、500、letter-spacing 3px、`var(--ink-muted)`、text-indent 3px |
| `.title-bar` | `#sidebar .profile-wrapper::after` | 18×3、border-radius 99px、`var(--ink)`、margin 12px auto 0 |

- 颜色走令牌的理由：`--ink` 亮色 #111418 与方案 B #111827 仅差几个色阶，`--ink-muted` 与 #6b7280 完全一致；令牌同时覆盖显式暗色，避免硬编码在暗色失效。
- text-indent 补偿：中文标题与副标题字距会在末字后留出空隙，现有副标题已有同类先例；补偿值等于字距。
- `.brand-dot` 不改：其边距用 em，副标题从 28px 降到 12px 后装饰点间距自动收紧，标记与文案保持原样。

## 3 不采纳项

- 方案 B 的 210px 宽度：会联动主栏 margin、≤849px 抽屉位移、≥1650px 断点与回到顶部公式，且底部图标行约 191.2px 在 210px 宽度下会换行；改动超出「原有不能动」边界，需另立任务授权。
- 方案 B 的容器 padding 40px 24px：侧栏容器 padding 为 0，内缩由 profile-wrapper、nav、sidebar-bottom 分别承担；直接加会双重内缩并让底部图标换行。
- 标题区顶部收紧到 40px：会移动导航起点，属布局变更，本轮不做。

## 4 验证计划

- 质量门禁：`cd rust && cargo fmt --check && cargo clippy --all-targets && cargo test`。
- 静态构建：`SITE_URL=https://example.com cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out rust/dist`。
- 结构回归：`./rust/target/release/nobelog diff public rust/dist --normalize-assets`，HTML 必须无差异，只允许资源差异；不改 `public/` 冻结基准。
- CSS 产物检查：产物 CSS 中检索 `#sidebar`、`#sidebar .site-title`、`#sidebar .site-subtitle`、`#sidebar .profile-wrapper::after` 新规则；加 `NOBELOG_CSS_REPORT=1` 观察裁剪。
- 预览：动态模式在临时端口起本地预览，常驻浏览器截图 light、dark、auto 与 375、849、850、1200、1650 宽度，重点看标题层级、装饰杠、侧栏边界、导航起点与底部图标不折行。

## 5 风险与注意

- 中文回退字体随环境不同，标题 800 与副标题 500 可能被合成；验收以本机截图加用户环境复核为准。
- auto 加系统暗色沿用既有级联偏差，不在本任务扩大改动。
- 侧栏与主栏同色后边界消失，导航 hover 与 active 的浅底仍提供层次；若想保留边界，只改标题区、不动底色。
- 新增只存在于 CSS 的类名会被 CSS 裁剪删除，因此只用现有 class 与 id。

## 6 验证结果

- 源文件改动：`git diff --numstat` 为 36/0，仅 `vendor/patches/patch.css`，纯追加；sha256 `c48fb011e847…`。
- 构建：`SITE_URL=https://example.com cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out rust/dist` 成功，15 页 152976 字节；`cargo test` 98 通过。
- 回归：对照构建 HEAD 版 CSS 与本次 CSS，两次构建的 HTML 41 项全同，只差 CSS 资源；与冻结基线 public 的 index.html 差异为相对日期漂移，对照实验同样复现。
- 产物：`rust/dist/assets/main.a8f4ea72.css` 含四条新规则，默认裁剪保留，裁剪统计 2392/7065。
- 计算样式亮色：侧栏 260px、rgb(255,255,255)、边框 0；标题 22px/800/4px；副标题 12px/500/3px、rgb(107,114,128)、上边距 6px；短杠 18×3、圆角 99px、rgb(17,20,24)、上边距 12px、左右 auto。
- 计算样式显式暗色：侧栏 rgb(20,23,27)、标题 rgb(230,233,236)、副标题 rgb(139,147,156)、短杠 rgb(230,233,236)。
- 截图：亮色 `sidebar-after-light.png`、暗色 `sidebar-after-dark.png`、改前 `sidebar-before.png`，均在 /root/博客 下。
- 未覆盖：常驻浏览器视口固定为 1600×857，375、849、850、1200、1650 宽度未单独截图；本轮未改宽度、位移与媒体查询规则，断点风险由代码审查覆盖。

## 7 用户反馈与当前状态

- 2026-11-01 用户判定本轮适配「改错了」，要求先停下等待新的修改指示。
- 当前状态：`vendor/patches/patch.css` 的 36 行追加仍在工作区，未提交、未推送、未部署；线上 10086 服务与线上样式保持原样。
- 临时预览服务已停止；截图仍保留在 /root/博客/sidebar-after-light.png、sidebar-after-dark.png、sidebar-before.png。
- 后续按用户新指示重新适配；如需回到改前状态，先还原 patch.css 再改。

## 8 按最新参考图二次重做（覆盖此前口径）

- 2026-11-01 用户给出最新参考图并要求直接按图改，明确覆盖之前指令。按图解读：标题区整体上移、主标题保持大字、副标题小字、短粗圆角杠、白底无界。
- 实现值：`#sidebar .site-title` 1.9rem/800/0.18em；`#sidebar .site-subtitle` 1.175rem/500/0.32em，保留圆点；装饰杠 24×4、圆角 99px；`#sidebar .profile-wrapper` 顶部留白收紧为 margin-top 0、padding-top 3.5rem、padding-bottom 0.625rem、margin-bottom 1.25rem。
- 实测位置：标题行盒 top 56、bottom 99；副标题 top 107；导航起点 top 181；装饰杠 24×4；均与参考图的关键位置一致。
- 构建：`SITE_URL=https://example.com cargo run --release --manifest-path rust/Cargo.toml -- build --self-assets --out rust/dist` 成功，产物 CSS `main.bba55ad7.css` 含四条最终规则。
- 截图：`/root/博客/sidebar-v5-light.png`、`/root/博客/sidebar-v5-dark.png`。
- 当前仍未提交、未推送、未部署，线上 10086 保持旧样式。

## 9 按参考图做像素测量后的最终对齐

- 将用户参考图拷入工作区做像素测量：页面区 1135×587，侧栏内各元素坐标与颜色可精确提取；标题色 #111827，副标题色 #6b7280，导航胶囊底 #f3f4f6，导航文字接近 #1f2937，短杠 #111827 且为 22×4。
- 最终修正：标题保持 1.9rem/800/0.18em；副标题 1.175rem/500/0.32em 保持圆点；短杠改 22×4；导航项间距改为 0.8rem；导航文字 100% 字号并加深为 var(--ink)；图标改 var(--ink-muted)；标题区继续按参考图收紧。
- 对照：并排图 `/root/博客/compare-target-v6.png`（左为用户参考图裁剪，右为实现截图）；最终截图 `/root/博客/sidebar-v6-light.png`、`/root/博客/sidebar-v6-dark.png`。
- 字体说明：参考图来自 Windows 字体渲染，本机截图使用文泉驿回退字体，中文字形宽高与参考图略有差异；同一 CSS 在 Windows 字体下会更接近参考图。
- 当前仍未提交、未推送、未部署。

## 10 方案 B 强制落地并上线 10086

- 用户 2026-11-01 强制指令：按方案 B 原始 CSS 严格落地，字体与排版与参考图一致。实现仅改 vendor/patches/patch.css，从 HEAD 纯追加 136 行，无删除。
- 强制值全部落地：侧栏 210px、白底、无右边框、padding 40px 24px；标题 22px/800/#111827/字距 4px；副标题 12px/500/#6b7280/字距 3px、上边距 6px；短杠 18×3/#111827/圆角 99px、上边距 12px 居中。
- 210px 联动覆盖：≤849 抽屉位移、≥850 主栏与页脚偏移、≥1400 与 ≥1650 回到顶部/宽屏规则，产物 CSS 后写覆盖，无空白缝。
- 构建通过，cargo test 98 通过；对照构建证明 HTML 逐字节一致，仅 CSS 资源变化。
- 上线：用户弹窗授权重启 nobelog-local.service；r4-operator 尝试 1 次被 S-3 child-no-authorization 拦截，Lead 兜底执行成功。重启前 PID 238909，重启后 PID 419455，ActiveEnterTimestamp 2026-11-01 07:33:29 UTC。
- 线上实测：样式表 main.86c19fcb.css；侧栏 210px、rgb(255,255,255)、边框 0、padding 40px 24px；标题 22px/800/rgb(17,24,39)/4px；副标题 12px/500/rgb(107,114,128)/3px/上边距 6px；短杠 18×3、圆角 99px、rgb(17,24,39)、上边距 12px；主栏 margin-left 210px 且左缘 210px，无空白缝。
- 截图：/root/博客/live-10086-schemeB.png；参考对照 /root/博客/compare-scheme-b.png。
- 已知差异：本机中文字体光栅与参考图 1 到 2px；导航图标字形与底部功能图标为本站既有元素，参考图未包含。

## 11 全部回退

- 用户指令：全部回退后由用户验收，验收通过再全面重改。
- 执行：`git restore vendor/patches/patch.css`，恢复到提交 70dd83a；工作树仅剩本任务目录未跟踪；未提交、未推送。
- 重启：用户当轮原文授权「授权重启 nobelog-local.service」；重启后 PID 443082，ActiveEnterTimestamp 2026-11-01 07:50:40 UTC。
- 线上核对：10086 返回 main.1ca5c955.css，方案 B 的 width:210px 规则命中 0 条；侧栏计算宽 260px、底色 rgb(247,248,250)、右侧 1px 分隔、标题 25.6px/600、副标题 28px/700，即回退前原样。
- 证据截图：/root/博客/rollback-restored.png。
- 后续：等待用户验收；全面重改前先确认基准图与范围，并改用同窗口宽度整页对比作为验收口径。

## 12 四项最小范围改动上线

- 用户确认范围仅四项：背景打通、标题、字体、保留红线；其余结构、宽度、断点、导航、底部图标、深色三态不动。
- 实现：vendor/patches/patch.css 末尾追加 54 行新分区，+54/-0，文件 1244 到 1298 行；无 @media、无宽度、无模板、无 public 改动。
- 内容：#sidebar 背景 var(--paper)、右边框 0；标题 22px/800/4px/var(--ink)、副标题 12px/500/3px/var(--ink-muted)/margin-top 6px；.profile-wrapper margin 0 0 33.3px、padding 40px 0 0；红线仅 margin-top 12px；字体栈显式加 WenQuanYi Zen Hei，无新增字体文件。
- 部署：用户点名授权后 systemctl restart nobelog-local.service，新 PID 524289，启动 2026-11-01 08:37:11 UTC；10086 返回 main.73ab2571.css，与 rust/dist 产物 sha256 一致。
- 线上实测：侧栏 260px、背景与页面同为 rgb(255,255,255)，暗色同为 rgb(20,23,27)，border-right 0；标题 22px/800/4px/rgb(17,20,24)；副标题 12px/500/3px/rgb(107,114,128)/margin-top 6px；红线 36×1 渐变保留、margin-top 12px；导航起点 139；底部图标按钮单行 top 797。
- 截图：/root/博客/live-deploy-fourfix-light.png 与 /root/博客/live-deploy-fourfix-dark.png。
- 未完：未提交、未推送；博客树 docs/blog-tree.md 未跟踪；用户需自行验收线上效果。
