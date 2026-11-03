# Journal - zhemed (Part 2)

> Continuation from `journal-1.md` (archived at ~2000 lines)
> Started: 2026-11-02

---



## Session 59: 目录激活项取消特殊处理
<!-- trellis-session: v=2 fp=0198d444ce955d58 -->

**Date**: 2026-11-02
**Task**: 目录激活项取消特殊处理
**Branch**: `main`

### Summary

激活项从墨色+700+指示条改为与普通条目完全相同（次要灰/400/无指示条）；hover 对所有条目生效，浅灰底+墨色+700。

### Main Changes

- vendor/patches/patch.css +8/-12：激活项样式改为普通态，指示条隐藏，hover 规则后置

### Git Commits

| Hash | Message |
|------|---------|
| `7cd7de9` | style: 目录激活项取消特殊处理 [task:11-02-toc-active-flat] |

### Testing

- [OK] 激活项与普通项计算样式一致（same=true）；激活项 hover 仍有 rgb(238,241,245) 底、rgb(17,20,24)、700；HTML 产物不变；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 60: 目录激活项加 2px 竖条提示
<!-- trellis-session: v=2 fp=d339e1c392b107e5 -->

**Date**: 2026-11-02
**Task**: 目录激活项加 2px 竖条提示
**Branch**: `main`

### Summary

激活项取消特殊化后无法定位当前章节；在条目左缘加 2px 墨色圆角短竖条（垂直居中），文字保持次要灰 400；::before 需以条目为定位参照。

### Main Changes

- vendor/patches/patch.css +11/-1：.toc-link position:relative；激活项 ::before 显示 2px×1rem 墨色竖条

### Git Commits

| Hash | Message |
|------|---------|
| `11abcd7` | style: 目录激活项加 2px 竖条提示 [task:11-02-toc-active-bar] |

### Testing

- [OK] 竖条 2px×16px 墨色、垂直居中、贴条目左侧；激活项文字仍灰 400；hover 仍 #eef1f5+墨色+700 且竖条保留；HTML 产物不变；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 61: 文章标题与面包屑可点击回干净地址
<!-- trellis-session: v=2 fp=94d5591058eda113 -->

**Date**: 2026-11-02
**Task**: 文章标题与面包屑可点击回干净地址
**Branch**: `main`

### Summary

面包屑当前文章名与文章 h1 都改为指向 /posts/<slug>/ 的自链接，点一下清除 #锚点；其它页面面包屑保持纯文本。

### Main Changes

- rust/src/templates.rs：breadcrumb() 末项支持链接、文章页 crumbs 带 href、h1 包自链接；vendor/patches/patch.css：标题自链接样式

### Git Commits

| Hash | Message |
|------|---------|
| `b2089cb` | feat: 文章标题与面包屑可点击回干净地址 [task:11-02-post-title-link] |

### Testing

- [OK] 点击两处均清空 location.hash；/archives/ 等页面面包屑仍纯文本；归一化后仅 2 个文章页产物变化；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**

### Next Steps

- 模板变化使静态产物与旧 public 基线不再逐字节一致；如需保持基线，需要重建基准目录


## Session 62: 正文行内链接恢复强调蓝
<!-- trellis-session: v=2 fp=484ea5d92c4548d2 -->

**Date**: 2026-11-02
**Task**: 正文行内链接恢复强调蓝
**Branch**: `main`

### Summary

全站链接统一墨色后，正文行内链接失去可辨识度；把 .markdown-body a 恢复为 --accent（亮 #3b5b7a / 暗 #8fb3d9），导航类链接保持墨色。

### Main Changes

- vendor/patches/patch.css +3/-3：.markdown-body a 颜色由 --link-color 改为 --accent

### Git Commits

| Hash | Message |
|------|---------|
| `ffe7033` | fix: 正文行内链接恢复强调蓝 [task:11-02-inline-link-accent] |

### Testing

- [OK] 正文行内链接亮色 rgb(59,91,122)、暗色 rgb(143,179,217)；面包屑/目录/归档仍墨色；无下划线；产物无新增结构性差异；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 63: 正文链接改用更醒目的蓝色
<!-- trellis-session: v=2 fp=1de1277f196ba293 -->

**Date**: 2026-11-02
**Task**: 正文链接改用更醒目的蓝色
**Branch**: `main`

### Summary

强调蓝 #3b5b7a 对比不足；新增 --link-body（亮 #0969da / 暗 #58a6ff）用于正文行内链接，导航类链接与 --accent 其它用途不变。

### Main Changes

- vendor/patches/patch.css +3/-1：两个令牌块新增 --link-body，.markdown-body a 改用之

### Git Commits

| Hash | Message |
|------|---------|
| `2a65f11` | style: 正文链接改用更醒目的蓝色 [task:11-02-inline-link-bright] |

### Testing

- [OK] 正文链接亮 rgb(9,105,218)、暗 rgb(88,166,255)；面包屑/目录/归档不变；仅 CSS 哈希变化；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 64: 修复置顶图钉渲染成方块
<!-- trellis-session: v=2 fp=23019997c8f9eded -->

**Date**: 2026-11-02
**Task**: 修复置顶图钉渲染成方块
**Branch**: `main`

### Summary

meta 字重规则把 fas(solid) 压到 400，退回不含图钉字形的 Regular 字体；为 .post-meta 内 fas/fa-solid 恢复 900 并覆盖 #post-list 的高特异度规则。

### Main Changes

- vendor/patches/patch.css +9 行：图标字重恢复规则

### Git Commits

| Hash | Message |
|------|---------|
| `fc18749` | fix: 修复置顶图钉图标渲染成方块 [task:11-02-pin-icon-weight] |

### Testing

- [OK] 图钉计算字重 900、渲染正常；日历仍 400；meta 文字与图标颜色不变；仅 CSS 哈希变化；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 65: 超宽屏内容贴左、右栏贴右
<!-- trellis-session: v=2 fp=e54b07db3e2f6eff -->

**Date**: 2026-11-03
**Task**: 超宽屏内容贴左、右栏贴右
**Branch**: `main`

### Summary

≥1650px 改为容器左右各 56px、行内 space-between：内容贴侧栏、右栏贴屏幕右缘，多余空间落在两者之间；主栏保持 1060 上限。

### Main Changes

- vendor/patches/patch.css +19 行：新增 @media (min-width:1650px) 容器/行/面板规则

### Git Commits

| Hash | Message |
|------|---------|
| `8285a70` | style: 超宽屏内容贴左、右栏贴右 [task:11-03-wide-layout-edges] |

### Testing

- [OK] 2555/1920/1650/1264 四档实测：容器左缘 356、右栏右缘=屏宽−56、主栏≤1060、无横向溢出；仅 CSS 哈希变化；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**

### Next Steps

- 2555 下主栏与右栏间隔达 843px，如觉得空可考虑提高主栏 1060 上限


## Session 66: 回滚超宽屏布局改动
<!-- trellis-session: v=2 fp=5f9667a2063a290c -->

**Date**: 2026-11-03
**Task**: 回滚超宽屏布局改动
**Branch**: `main`

### Summary

撤销 8285a70（≥1650 内容贴左、右栏贴右），恢复三段留白均分；revert 因任务文件移动冲突，改为手工移除 18 行 CSS，保留其它改动。

### Main Changes

- vendor/patches/patch.css −18 行：删除 @media (min-width:1650px) 规则块

### Git Commits

| Hash | Message |
|------|---------|
| `fa2387d` | revert: 回滚超宽屏布局改动 [task:11-03-revert-wide-layout] |

### Testing

- [OK] 2555 左右留白各 318、1920 各 107，均分恢复；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 67: 右侧栏贴屏幕右缘（内容位置不变）
<!-- trellis-session: v=2 fp=7197684eb6cb9cd4 -->

**Date**: 2026-11-03
**Task**: 右侧栏贴屏幕右缘（内容位置不变）
**Branch**: `main`

### Summary

超宽屏下左侧栏贴左缘而右栏距右缘 318px；把容器右边界收到屏宽−56、行内 space-between，把右栏推到右缘，同时保持内容块左留白与主栏位置不变。

### Main Changes

- vendor/patches/patch.css +19 行：新增 @media (min-width:1650px) 容器右边界/space-between/面板最小间隔

### Git Commits

| Hash | Message |
|------|---------|
| `4293bb3` | style: 右侧栏贴屏幕右缘，内容位置不变 [task:11-03-panel-edge-right] |

### Testing

- [OK] 2555：容器 [618,1881,2499]、右栏 [2259,2499]、左留白 318 不变、右缘 56；1920：左 107/右 56；1650/1264 正常；无溢出；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 68: 回滚右侧栏贴右缘改动
<!-- trellis-session: v=2 fp=e9da69ce4b5b8cbd -->

**Date**: 2026-11-03
**Task**: 回滚右侧栏贴右缘改动
**Branch**: `main`

### Summary

按用户要求恢复三段留白均分布局，撤销 4293bb3，仅删除其新增的 ≥1650px 规则块。

### Main Changes

- vendor/patches/patch.css −18 行

### Git Commits

| Hash | Message |
|------|---------|
| `7e0e5c9` | revert: 回滚右侧栏贴右缘改动 [task:11-03-revert-panel-edge] |

### Testing

- [OK] 2555 左右留白各 318 相等；1264 布局与之前一致；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 69: 恢复右侧栏贴右缘
<!-- trellis-session: v=2 fp=978342086e0c4441 -->

**Date**: 2026-11-03
**Task**: 恢复右侧栏贴右缘
**Branch**: `main`

### Summary

澄清后重新应用 4293bb3：≥1650 内容位置不变，容器右边界收至屏宽−56，space-between 把右栏推到右缘。

### Main Changes

- vendor/patches/patch.css +19 行：@media (min-width:1650px) 容器/行/面板规则

### Git Commits

| Hash | Message |
|------|---------|
| `00e4ea4` | style: 恢复右侧栏贴右缘 [task:11-03-restore-panel-edge] |

### Testing

- [OK] 2555：容器 [618,1881,2499]、右栏 [2259,2499]、左留白 318、右缘 56；1264 不变；无溢出；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 70: 主栏上限放宽到 1320（≥1650）
<!-- trellis-session: v=2 fp=8b9cade5bcb632f9 -->

**Date**: 2026-11-03
**Task**: 主栏上限放宽到 1320（≥1650）
**Branch**: `main`

### Summary

超宽屏主栏从 1060 提到 1320，右栏继续贴右，中间空白 581→323；1920 下主栏随空间增长到 1161。

### Main Changes

- vendor/patches/patch.css +5 行：@media (min-width:1650px) 内追加 main max-width:1320

### Git Commits

| Hash | Message |
|------|---------|
| `9e61327` | style: ≥1650 主栏上限放宽到 1320 [task:11-03-widen-main-1320] |

### Testing

- [OK] 2559：主栏 1320、间隔 323、右缘 56；1920：主栏 1161、间隔 56；1264 不变；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**

### Next Steps

- 验证截图统一 1264×809，不再混用其它分辨率


## Session 71: 布局改为比例流体（1264↔1920 等比，封顶居中）
<!-- trellis-session: v=2 fp=c2412c3652aaca4a -->

**Date**: 2026-11-03
**Task**: 布局改为比例流体（1264↔1920 等比，封顶居中）
**Branch**: `main`

### Summary

用 clamp+vw 把侧栏/间隔/右栏按 1264 基准等比放大到 1920，主栏吃剩余；≥1921 封顶并按视口居中。替换原三段留白公式与 ≥1650 补丁。

### Main Changes

- vendor/patches/patch.css +50/-24：删除两段旧布局块，新增 ≥1264 流体块与 ≥1921 居中块

### Git Commits

| Hash | Message |
|------|---------|
| `2834047` | style: 布局改为比例流体（1264↔1920 等比，封顶居中）[task:11-03-fluid-layout] |

### Testing

- [OK] 1264 截图 sha256 与改动前一致；1440 侧栏296/间隔64、1920 侧栏395/间隔85/主栏906、2559 封顶居中；无横向溢出；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**

### Next Steps

- 字号不随屏宽放大，超宽屏仍是定字号；如需整体缩放需浏览器缩放而非 CSS


## Session 72: 回滚比例流体布局
<!-- trellis-session: v=2 fp=eaa36fe8c13508a9 -->

**Date**: 2026-11-03
**Task**: 回滚比例流体布局
**Branch**: `main`

### Summary

比例流体方案不符合预期，恢复到 2834047 之前：三段留白 + ≥1650 右栏贴右 + 主栏 1320。

### Main Changes

- vendor/patches/patch.css 恢复到 2834047^

### Git Commits

| Hash | Message |
|------|---------|
| `b0ed524` | revert: 回滚比例流体布局 [task:11-03-revert-fluid-layout] |

### Testing

- [OK] 1264 几何与之前一致；2559 容器 [620,2503]、主栏 1320、右栏 [2263,2503]；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 73: 内容块整体居中、两侧留白相等
<!-- trellis-session: v=2 fp=f427b45ea7fd07c2 -->

**Date**: 2026-11-03
**Task**: 内容块整体居中、两侧留白相等
**Branch**: `main`

### Summary

≥1650 容器宽度封顶 1457 并用 auto 居中，主栏吃满（1161），右栏紧随 56：1920 左右各 82、2560 左右各 399，完全对称。

### Main Changes

- vendor/patches/patch.css：替换 ≥1650 块（居中容器 + 主栏取消上限 + 右栏 56）

### Git Commits

| Hash | Message |
|------|---------|
| `f3cf94f` | style: 内容块整体居中、两侧留白相等 [task:11-03-center-block] |

### Testing

- [OK] 1920 容器[382,1457]、左82/右82；2554 左399/右399；1440 与 1264 不变；无溢出；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**

### Next Steps

- 设计基准改为 1920×1080；后续截图统一用它


## Session 74: 大屏内容撑满（左右各 56）
<!-- trellis-session: v=2 fp=2d107381abea828b -->

**Date**: 2026-11-03
**Task**: 大屏内容撑满（左右各 56）
**Branch**: `main`

### Summary

取消大屏容器封顶，内容撑满：1920 主栏 1212、2555 主栏 1847，左右各 56，右侧空白消除。

### Main Changes

- vendor/patches/patch.css ≥1650 块：容器 width calc(100%-112) + margin 56，max-width none

### Git Commits

| Hash | Message |
|------|---------|
| `385e4a6` | style: 大屏内容撑满，左右各留 56 [task:11-03-fill-wide] |

### Testing

- [OK] 1920 主栏1212/左右56；2555 主栏1847/左右56；无溢出；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**

### Next Steps

- 用户截图仍显示旧样式，需强刷后确认


## Session 75: 页脚对齐内容与侧栏两端
<!-- trellis-session: v=2 fp=8696c9cdd3ea0f99 -->

**Date**: 2026-11-03
**Task**: 页脚对齐内容与侧栏两端
**Branch**: `main`

### Summary

页脚因 #tail-wrapper 是 col-xl-9 只有 75% 宽而缩在中间；≥1200 改为占满容器宽、左右内边距 28px，两端与内容/侧栏对齐。

### Main Changes

- vendor/patches/patch.css +17 行：≥1200 的 #tail-wrapper 与 footer 内边距

### Git Commits

| Hash | Message |
|------|---------|
| `03ba77b` | fix: 页脚对齐内容与侧栏两端 [task:11-03-footer-align] |

### Testing

- [OK] 1920 页脚 [356,1864]，左文字 384、右文字止于 1836；1264 页脚 [316,1208]；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 76: 页脚改为与正文列同宽
<!-- trellis-session: v=2 fp=f283be0dcf0b5816 -->

**Date**: 2026-11-03
**Task**: 页脚改为与正文列同宽
**Branch**: `main`

### Summary

纠正上一版：页脚不应延伸到侧栏右边，而应与正文列同宽（容器宽 − 296）。1920 页脚 [356,1568]、1264 [316,912]。

### Main Changes

- vendor/patches/patch.css +4/-3：#tail-wrapper 宽度 calc(100% - 296px)

### Git Commits

| Hash | Message |
|------|---------|
| `77c9740` | fix: 页脚改为与正文列同宽 [task:11-03-footer-content-width] |

### Testing

- [OK] 1920 页脚与正文列等宽 [356,1568]，右文字止于 1540；1264 同宽 [316,912]；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 77: 大屏版式试靠右
<!-- trellis-session: v=2 fp=657a271be061810a -->

**Date**: 2026-11-03
**Task**: 大屏版式试靠右
**Branch**: `main`

### Summary

内容块改为贴右：容器封顶 1457、右边距 56、左边距 auto，空白移到侧栏与内容之间（2555 下 742）。

### Main Changes

- vendor/patches/patch.css +2/-2：≥1650 容器 max-width 1457 + margin auto/56

### Git Commits

| Hash | Message |
|------|---------|
| `3a8fef3` | style: 大屏版式试靠右（空白移到左侧）[task:11-03-align-right-trial] |

### Testing

- [OK] 1920 容器[407,1864]；2555 容器[1042,2499] 左侧空白 742；1264 不变；无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**

### Next Steps

- 靠右属于试验版，用户对比后再定是否保留


## Session 78: 两侧留白相等，版心居中
<!-- trellis-session: v=2 fp=d389fc88e45d4317 -->

**Date**: 2026-11-03
**Task**: 两侧留白相等，版心居中
**Branch**: `main`

### Summary

靠右试验导致左742/右56偏侧；容器右外边距改回 auto，两侧均分：1920 各82，2558 各401。

### Main Changes

- vendor/patches/patch.css +1/-1：margin-right auto

### Git Commits

| Hash | Message |
|------|---------|
| `3e11493` | style: 两侧留白相等，版心居中 [task:11-03-sym-margins] |

### Testing

- [OK] 1920 左82/右82；2558 左401/右401；1264 不变；页脚仍与正文列同宽；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 79: 右栏贴屏幕右缘（内容不动）
<!-- trellis-session: v=2 fp=d8b7be69faabe1aa -->

**Date**: 2026-11-03
**Task**: 右栏贴屏幕右缘（内容不动）
**Branch**: `main`

### Summary

按指令只把右栏推到屏幕右缘（右缘间隔 0），内容位置与宽度锁定不变；6px 横向溢出用 overflow-x: clip 裁掉。

### Main Changes

- vendor/patches/patch.css：右栏负右边距 + 行 space-between；内容 flex 固定宽；页脚同步；main-wrapper overflow-x: clip

### Git Commits

| Hash | Message |
|------|---------|
| `92d7cef` | style: 右栏贴屏幕右缘，内容位置与宽度不动 [task:11-03-anchor-edges] |

### Testing

- [OK] 1920 内容[356,1568]不变、右栏[1680,1920]、右缘0、溢出0；2559 内容[490,1940]不变、右栏[2319,2559]、右缘0、溢出0；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 80: 左侧留白加宽，与中间间隔等宽
<!-- trellis-session: v=2 fp=b812e36507f01464 -->

**Date**: 2026-11-03
**Task**: 左侧留白加宽，与中间间隔等宽
**Branch**: `main`

### Summary

容器占满侧栏右侧宽度，主栏与右栏各用 auto 外边距平分剩余：左、中两处留白等宽，右栏仍贴屏幕右缘，页脚与正文列对齐。

### Main Changes

- vendor/patches/patch.css：≥1650 块（容器 100%、主栏 min(1450,100%-352) + margin-left auto、右栏 margin-left auto）、页脚宽度与偏移

### Git Commits

| Hash | Message |
|------|---------|
| `4bc9384` | style: 左侧留白加宽，与中间间隔等宽 [task:11-03-equal-gaps] |

### Testing

- [OK] 1920 左56/中56/右0；2559 左285/中285/右0；页脚与内容列对齐；无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 81: 内容宽度按比例（适配 125% 缩放）
<!-- trellis-session: v=2 fp=f1536bdec826dc28 -->

**Date**: 2026-11-03
**Task**: 内容宽度按比例（适配 125% 缩放）
**Branch**: `main`

### Summary

125% 缩放下实际 CSS 宽 2048；内容宽改为 min(1450,68%)，左右留白均分：2048 → 160/160，2560 → 285/285，右栏仍贴右缘。

### Main Changes

- vendor/patches/patch.css：主栏 width 与页脚 width/max-width/margin-left

### Git Commits

| Hash | Message |
|------|---------|
| `c196a88` | style: 内容宽度改为按比例，适配 125% 缩放 [task:11-03-proportional-width] |

### Testing

- [OK] 2048 内容1189/留白160/右缘0；2560 内容1450/留白285/右缘0；页脚对齐；无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 82: 面包屑跟随内容列左缘
<!-- trellis-session: v=2 fp=227637847c4e041a -->

**Date**: 2026-11-03
**Task**: 面包屑跟随内容列左缘
**Branch**: `main`

### Summary

内容列按比例后，面包屑与内容左缘错开；给 #topbar 加同一份留白公式的 padding-left，2048/2560 均对齐。

### Main Changes

- vendor/patches/patch.css：≥1650 块新增 #topbar padding-left

### Git Commits

| Hash | Message |
|------|---------|
| `f9185af` | chore(task): archive 11-03-breadcrumb-align |

### Testing

- [OK] 2048 面包屑 460 = 内容 460；2560 585 = 585；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 83: 发版 0.0.32（打包默认 example.com）
<!-- trellis-session: v=2 fp=d1ffa460c59b5dd3 -->

**Date**: 2026-11-03
**Task**: 发版 0.0.32（打包默认 example.com）
**Branch**: `main`

### Summary

改发版规则：package.sh 默认 https://example.com 占位，发布包与站点解耦；版本升到 0.0.32，打包/tag/push/gh release/重启线上完成。

### Main Changes

- tools/package.sh、README、spec 发版流程、Cargo.toml 与 Cargo.lock 版本号

### Git Commits

| Hash | Message |
|------|---------|
| `bc58bfc` | chore: 发版 0.0.32，打包默认 example.com 占位 [task:11-03-release-0032] |

### Testing

- [OK] fmt/clippy/test(98) 全绿；dist 包 sha256 通过；解包 --version=0.0.32；包内绝对地址为 example.com；Release 2 个附件；线上 200

### Status

[OK] **Completed**
