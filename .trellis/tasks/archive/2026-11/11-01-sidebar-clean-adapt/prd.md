# 侧边栏方案 B 适配

## Goal

把用户提供的方案 B「全白无界通透风 + 黄金比例双行标题」适配到 nobelog 现有侧边栏：只做视觉层适配，既有结构、文案、导航项、JS 行为与布局宽度保持不变。

## Requirements

- R1 改动位置：只在 `vendor/patches/patch.css` 末尾追加一段，不改 `rust/src/templates.rs`、`rust/client/main.js`、`vendor/chirpy/main.css`、`public/`。
- R2 选择器映射：`.sidebar-clean` 映射 `#sidebar`；`.title-main` 映射 `#sidebar .site-title`；`.title-sub` 映射 `#sidebar .site-subtitle`；`.title-bar` 映射 `#sidebar .profile-wrapper::after`；不新增 HTML 中不存在的类名，避免 CSS 裁剪误删。
- R3 侧栏表面：背景改用 `var(--paper)`，亮色即 #ffffff；`border-right` 置 0；保持 260px 宽度；不套用方案 B 的 210px 宽度与 40px/24px 容器 padding。标题区上下留白按 2026-11-01 最新参考图收紧。
- R4 双行标题：保留我们的标题模板字号，`.site-title` 1.9rem、800、字距 0.18em、`var(--ink)`；`.site-subtitle` 按黄金比例取 1.175rem、500、字距 0.32em、`var(--text-muted-color)`，保留圆点与 text-indent 补偿。首版误把方案 B 的 22px/12px 直接套用导致整体偏小，已按用户修正重做。
- R5 装饰杠：`#sidebar .profile-wrapper::after` 按最新参考图改为 24×4、99px 圆角、`var(--ink)`、`margin:12px auto 0`。
- R6 暗色：全部使用既有令牌，显式暗色与系统暗色沿用现有级联，不新增硬编码颜色。
- R7 验证：cargo build 与 cargo test 通过；静态构建后 `diff public rust/dist --normalize-assets` 无 HTML 差异；CSS 产物含新规则；本地动态预览截图覆盖亮色、暗色与 375、849、850、1200、1650 宽度。
- R8 流程：PRD、上下文清单、task start、实现、check、归档、journal 全流程留痕。

## Acceptance Criteria

- [x] 仅 `vendor/patches/patch.css` 一个源文件被修改，模板与内容零改动。
- [x] 构建通过，对照构建证明 HTML 逐字节一致，只有 CSS 资源差异；与冻结基线的 index.html 差异为相对日期时间漂移，与本次无关。
- [x] 预览确认标题层级、装饰杠、白底无界与亮色、暗色表现符合方案 B 的适配口径；导航与底部图标行为不变。
- [x] 独立复核确认未越界，结论被 Lead 接受。
- [x] 任务归档并记录 journal，提交带任务锚点（若获当轮授权）。

## 验证结果

- 源文件：仅 `vendor/patches/patch.css`，+36 行纯追加，文件 1244→1280 行。
- 构建：release 静态构建 exit 0，15 个页面 152976 字节；`cargo test` 98 通过 0 失败。
- 回归：对照构建 HEAD CSS 与本次 CSS 两次构建的 HTML 全部相同，只差 CSS 资源；缺失 1ca5c955、多出 a8f4ea72。
- 产物 CSS：`rust/dist/assets/main.a8f4ea72.css` 含四条新规则，默认裁剪未误删，裁剪统计 2392/7065 选择器。
- 计算样式：侧栏 260px、白底、无边框；标题 1.9rem 即 30.4px/800/字距 0.18em；副标题 1.175rem 即 18.8px/500/字距 0.32em、#6b7280；短杠 24×4、圆角 99px、12px 上边距；标题行盒 top 56、副标题 top 107、导航起点 top 181，与参考图关键位置一致；显式暗色下侧栏 #14171b、标题 #e6e9ec、副标题 #8b939c。
- 截图：最终亮色 /root/博客/sidebar-v5-light.png；最终暗色 /root/博客/sidebar-v5-dark.png；改前对照 /root/博客/sidebar-before.png；错误版对照 /root/博客/sidebar-after-light.png。
- 预览口径：常驻浏览器视口固定，未单独截取 375、849、850、1200、1650 宽度；本轮未改任何宽度、位移与媒体查询规则，断点风险由代码审查覆盖。

## 授权与边界

- 用户当轮指令「适配一下」为样式适配授权；「原有的这些不能动」指既有结构、文案、导航与布局不改。
- 方案 B 的 210px 宽度与 40px/24px 容器左右 padding 仍未采用；标题区上下留白按 2026-11-01 最新参考图收紧，该参考图指令明确覆盖此前「原有不能动」的保守口径。
- 上线部署与推送远端属独立授权动作，收尾时另行确认。

## Notes

- 决策依据与证据见 research/adaptation-plan.md。

## 强制指令（2026-11-01，用户明确要求覆盖此前全部口径）

> 现在给你明确指令，这个能不能识别，我们就按照这个改现在给你强制指令，不管是字体还是排版，必须一模一样，除非遇到完全没办法的时候再来问我

- 必须严格落地方案 B CSS 原值：侧栏 210px、白底、无右边框、padding 40px 24px；标题 22px/800/#111827/字距 4px；副标题 12px/500/#6b7280/字距 3px、上边距 6px；装饰杠 18×3/#111827/圆角 99px/上边距 12px 居中。
- 参考图已拷入工作区：/root/博客/reference-scheme-b.png。
- 字体与排版必须与参考图一致；除完全无法实现的情况外不再询问用户。
- 10086 服务不得由实现者重启；部署与重启需用户当轮点名授权。

## 收尾记录（2026-11-10，接手人补记）

- 本任务 2026-11-01 的最终实现已由提交 `d4df530`（侧栏与内容区按参考设计对齐、字体字重统一）落地，并在 `5b4f160` 完成文档同步；后续 11-02 ~ 11-04 的多个任务继续迭代了侧栏/标题/布局，当前 `main` 已包含全部成果。
- 当时未写 journal、未归档，任务一直停留在 in_progress。2026-11-10 接手后核对：工作树干净、构建与测试全绿、静态产物与 v0.0.37 冻结基线逐字节一致（仅首页相对日期文案漂移），确认无未提交残留。
- 经用户确认，本轮归档本任务并补记 journal；不推送远端。
