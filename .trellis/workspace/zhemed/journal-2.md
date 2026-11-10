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


## Session 84: 右侧栏宽度改为按比例
<!-- trellis-session: v=2 fp=75b9b9e19d3c3838 -->

**Date**: 2026-11-03
**Task**: 右侧栏宽度改为按比例
**Branch**: `main`

### Summary

面板 240 → clamp(240,11.7vw,300)，2560 下 300、标签不再折行；页脚与面包屑公式同步。

### Main Changes

- vendor/patches/patch.css：面板宽度与两处留白公式

### Git Commits

| Hash | Message |
|------|---------|
| `7d72897` | chore(task): archive 11-03-panel-width-proportional |

### Testing

- [OK] 1920 面板 240、留白 139；2560 面板 300、留白 255、标签单行；无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 85: 撤回右栏右对齐
<!-- trellis-session: v=2 fp=61e13dd5748d9129 -->

**Date**: 2026-11-03
**Task**: 撤回右栏右对齐
**Branch**: `main`

### Summary

右对齐试验被否决，删除 4 条 text-align:right；保留面板 clamp 比例宽度，标签单行。

### Main Changes

- vendor/patches/patch.css：删除右栏右对齐规则

### Git Commits

| Hash | Message |
|------|---------|
| `cf04af7` | chore(task): archive 11-03-revert-panel-align |

### Testing

- [OK] 2560 右侧栏 300、左对齐、标签单行；无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 86: 手机端许可与分享排成一行
<!-- trellis-session: v=2 fp=898893c5f0ca764e -->

**Date**: 2026-11-03
**Task**: 手机端许可与分享排成一行
**Branch**: `main`

### Summary

手机端尾部署名区从两行右对齐改为单行：许可左、分享右；849px 以下生效，过窄自动换行。

### Main Changes

- vendor/patches/patch.css：把 ≤576 的两行右对齐块换成 ≤849 的单行块

### Git Commits

| Hash | Message |
|------|---------|
| `4c4a8cb` | chore(task): archive 11-03-mobile-tail-row |

### Testing

- [OK] 390：许可[28,130]/分享[202,160] 同行；360：同行间距14；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 87: 手机端许可与分享垂直对齐
<!-- trellis-session: v=2 fp=673190387ef2563f -->

**Date**: 2026-11-03
**Task**: 手机端许可与分享垂直对齐
**Branch**: `main`

### Summary

许可行被主题的 1rem 上边距顶低 16px；在手机上归零后与分享图标中线对齐。

### Main Changes

- vendor/patches/patch.css：≤849 块内许可行 margin-top:0

### Git Commits

| Hash | Message |
|------|---------|
| `4cba2c1` | fix: 手机端许可行与分享图标垂直对齐 [task:11-03-mobile-tail-row] |

### Testing

- [OK] 390 下许可高 19、分享高 34，中线偏差 0；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 88: 发版 0.0.33
<!-- trellis-session: v=2 fp=f73b17b7db31c115 -->

**Date**: 2026-11-03
**Task**: 发版 0.0.33
**Branch**: `main`

### Summary

0.0.32 之后：右侧栏比例宽度、撤回右对齐、手机端许可与分享单行对齐；打包/tag/push/gh release/重启线上完成。

### Main Changes

- Cargo.toml / Cargo.lock 版本 0.0.33

### Git Commits

| Hash | Message |
|------|---------|
| `0b2243f` | chore: 发版 0.0.33 [task:11-03-release-0033] |

### Testing

- [OK] fmt/clippy/test(98)；包 sha256 通过；解包 --version=0.0.33；Release 2 附件；线上 200

### Status

[OK] **Completed**


## Session 89: 正文图片居中
<!-- trellis-session: v=2 fp=425adb7081677a14 -->

**Date**: 2026-11-03
**Task**: 正文图片居中
**Branch**: `main`

### Summary

正文图片默认靠左；改为 img 与 a.img-link 一起居中，1920 下左右各 67，手机满宽。

### Main Changes

- vendor/patches/patch.css：新增图片与图片链接的居中规则

### Git Commits

| Hash | Message |
|------|---------|
| `d07b084` | chore(task): archive 11-03-center-article-images |

### Testing

- [OK] 1920 图片 960、左右 67/67；390 满宽 334；无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 90: 正文图片撑满内容列
<!-- trellis-session: v=2 fp=b8aba58b712cba09 -->

**Date**: 2026-11-03
**Task**: 正文图片撑满内容列
**Branch**: `main`

### Summary

图片由居中改为 100% 撑满内容列；1920 放大 1.14×、2560 放大 1.5×、手机缩小，用户确认采用。

### Main Changes

- vendor/patches/patch.css：img 与 a.img-link 宽度 100%

### Git Commits

| Hash | Message |
|------|---------|
| `4d64a98` | chore(task): archive 11-03-image-fill |

### Testing

- [OK] 1920 1094/1.14×；2560 1442/1.5×；390 334；无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 91: 侧栏按比例、内容收窄
<!-- trellis-session: v=2 fp=3d202b82d0782190 -->

**Date**: 2026-11-03
**Task**: 侧栏按比例、内容收窄
**Branch**: `main`

### Summary

宽屏块统一用四个变量推导：侧栏 clamp(300,16.5vw,360)、右栏 clamp(240,11.7vw,300)、内容 62%、留白均分；页脚/面包屑同步。

### Main Changes

- vendor/patches/patch.css：重写 ≥1650 块

### Git Commits

| Hash | Message |
|------|---------|
| `6594dce` | chore(task): archive 11-03-side-content-balance |

### Testing

- [OK] 1920 侧317/内994/留185；2048 侧338/内1060/留205；2560 侧360/内1364/留268；对齐无溢出；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 92: 侧栏标题放大、与导航拉开距离
<!-- trellis-session: v=2 fp=e6f98875deab63e1 -->

**Date**: 2026-11-04
**Task**: 侧栏标题放大、与导航拉开距离
**Branch**: `main`

### Summary

对照 Chirpy 参考：站点标题 25→28px，标题块到导航 51→91px。

### Main Changes

- vendor/patches/patch.css：#sidebar .site-title/.site-subtitle/.profile-wrapper

### Git Commits

| Hash | Message |
|------|---------|
| `bb03851` | style: 侧栏标题放大、与导航拉开距离 [task:11-04-sidebar-title-scale] |

### Testing

- [OK] 2048 下标题 28px、间隔 91px；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 93: 侧栏导航放大并下移
<!-- trellis-session: v=2 fp=2818be1d459797f2 -->

**Date**: 2026-11-04
**Task**: 侧栏导航放大并下移
**Branch**: `main`

### Summary

导航字号 16→17px、单项高 41→47、节距 45→55、标题到导航 91→123px。

### Main Changes

- vendor/patches/patch.css：nav-link 字号/内边距、nav-item 间距、profile-wrapper 下边距

### Git Commits

| Hash | Message |
|------|---------|
| `4dac8e8` | style: 侧栏导航放大并下移 [task:11-04-nav-scale] |

### Testing

- [OK] 2048 下数值符合预期；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 94: 侧栏标题、短杠、竖线加大
<!-- trellis-session: v=2 fp=840d0d0383d13794 -->

**Date**: 2026-11-04
**Task**: 侧栏标题、短杠、竖线加大
**Branch**: `main`

### Summary

标题 28→32px；短杠 36×1→48×3；菜单右侧竖线 1×124→2×156。改的是末尾真正生效的那条短杠规则。

### Main Changes

- vendor/patches/patch.css：site-title 字号、profile-wrapper::after、ul.nav::after

### Git Commits

| Hash | Message |
|------|---------|
| `828c487` | style: 侧栏标题、短杠、竖线都加大 [task:11-04-sidebar-lines-scale] |

### Testing

- [OK] 计算值 32px / 48×3 / 2×156；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 95: 回滚侧栏标题/短杠/竖线加大
<!-- trellis-session: v=2 fp=7b0c9dc48a385e9a -->

**Date**: 2026-11-04
**Task**: 回滚侧栏标题/短杠/竖线加大
**Branch**: `main`

### Summary

回滚 828c487：标题 32→28px、短杠 48×3→36×1、竖线 2×156→1×124；导航放大那版保留。

### Main Changes

- vendor/patches/patch.css：三处数值回滚

### Git Commits

| Hash | Message |
|------|---------|
| `ee301b1` | revert: 回滚侧栏标题/短杠/竖线加大 [task:11-04-revert-sidebar-lines] |

### Testing

- [OK] 计算值 28px / 36×1 / 1×124，导航仍 17px；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 96: 发版 0.0.34
<!-- trellis-session: v=2 fp=d4b53b1fddb05a4e -->

**Date**: 2026-11-04
**Task**: 发版 0.0.34
**Branch**: `main`

### Summary

按比例版式（侧栏 16.5vw/右栏 11.7vw/正文 62%）、正文图片撑满、侧栏标题与导航加大；打包/tag/push/gh release/重启完成。

### Main Changes

- Cargo.toml / Cargo.lock 版本 0.0.34

### Git Commits

| Hash | Message |
|------|---------|
| `c4c3d4c` | chore: 发版 0.0.34 [task:11-04-release-0034] |

### Testing

- [OK] fmt/clippy/test(98)；包 sha256 通过；解包 0.0.34；Release 2 附件；线上 200

### Status

[OK] **Completed**


## Session 97: 侧栏导航上移
<!-- trellis-session: v=2 fp=696d645f22629ccd -->

**Date**: 2026-11-04
**Task**: 侧栏导航上移
**Branch**: `main`

### Summary

profile-wrapper 下边距 6.5→5rem，副标题到导航 123→99px。

### Main Changes

- vendor/patches/patch.css：profile-wrapper margin-bottom

### Git Commits

| Hash | Message |
|------|---------|
| `bb71fff` | style: 侧栏导航上移 [task:11-04-nav-up] |

### Testing

- [OK] 2048 下间隔 99px；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 98: 手机端侧栏整体上移
<!-- trellis-session: v=2 fp=3a595dc44ea520be -->

**Date**: 2026-11-04
**Task**: 手机端侧栏整体上移
**Branch**: `main`

### Summary

≤849px 下标题块上移：标题 84→36、导航 282→198；桌面不变。

### Main Changes

- vendor/patches/patch.css：新增 ≤849px 的 profile-wrapper 间距覆盖

### Git Commits

| Hash | Message |
|------|---------|
| `35a1ba4` | style: 手机端侧栏整体上移 [task:11-04-mobile-sidebar-up] |

### Testing

- [OK] 390 下标题 36、导航 198；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 99: 发版 0.0.35
<!-- trellis-session: v=2 fp=3648503f7d5e1f13 -->

**Date**: 2026-11-04
**Task**: 发版 0.0.35
**Branch**: `main`

### Summary

桌面侧栏导航上移（123→99px）、手机抽屉侧栏整体上移（标题84→36、导航282→198）；打包/tag/push/release/重启完成。

### Main Changes

- Cargo.toml / Cargo.lock 版本 0.0.35

### Git Commits

| Hash | Message |
|------|---------|
| `fad0e95` | chore: 发版 0.0.35 [task:11-04-release-0035] |

### Testing

- [OK] fmt/clippy/test(98)；包 sha256 通过；解包 0.0.35；Release 2 附件；线上 200

### Status

[OK] **Completed**


## Session 100: 修复系统深色下白底白字
<!-- trellis-session: v=2 fp=6fe03dc17fe17022 -->

**Date**: 2026-11-04
**Task**: 修复系统深色下白底白字
**Branch**: `main`

### Summary

根因：主题与 github-markdown 的 prefers-color-scheme:dark 分支在 auto 档位接管，只改部分变量，与浅色板混搭。修复：auto 解析为 light/dark 写 data-mode + 浅色态覆盖 github 变量。

### Main Changes

- rust/client/main.js、rust/src/templates.rs（EARLY_SCRIPT）、vendor/patches/patch.css

### Git Commits

| Hash | Message |
|------|---------|
| `9badfef` | fix: 系统深色下白底白字 [task:11-04-dark-mode-leak] |

### Testing

- [OK] 系统深色→dark 深底浅字；系统浅色→light 白底深字；系统深色+手动浅色→浅色正常；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 101: 发版 0.0.36
<!-- trellis-session: v=2 fp=d66fc3639f24a3fb -->

**Date**: 2026-11-04
**Task**: 发版 0.0.36
**Branch**: `main`

### Summary

修复系统深色下白底白字：auto 解析成 light/dark 写 data-mode、覆盖 github-markdown 深色媒体查询；打包/tag/push/release/重启完成。

### Main Changes

- Cargo.toml / Cargo.lock 版本 0.0.36

### Git Commits

| Hash | Message |
|------|---------|
| `6688cf9` | chore: 发版 0.0.36 [task:11-04-release-0036] |

### Testing

- [OK] fmt/clippy/test(98)；包 sha256 通过；解包 0.0.36；Release 2 附件；线上 200

### Status

[OK] **Completed**


## Session 102: 锁定 Rust 1.99.1
<!-- trellis-session: v=2 fp=fd40eee689fe3ea7 -->

**Date**: 2026-11-06
**Task**: 锁定 Rust 1.99.1
**Branch**: `main`

### Summary

toolchain 文件上移到仓库根，MSRV 收紧到 1.99.1，spec 同步；根目录 cargo --version 已是 1.99.1。

### Main Changes

- rust-toolchain.toml（移到根）、rust/Cargo.toml、spec

### Git Commits

| Hash | Message |
|------|---------|
| `4f64634` | chore: 锁定 Rust 1.99.1 [task:11-06-pin-rust-1-99-1] |

### Testing

- [OK] 根目录 cargo 1.99.1；fmt/clippy/test(98) 全绿

### Status

[OK] **Completed**


## Session 103: 新机器工具链自检与文档
<!-- trellis-session: v=2 fp=d52015ceb674cf7e -->

**Date**: 2026-11-06
**Task**: 新机器工具链自检与文档
**Branch**: `main`

### Summary

新增 tools/check-rust.sh 校验工具链；package.sh 去掉写死的 cargo 路径并前置自检；README 补 rustup 安装说明。

### Main Changes

- tools/check-rust.sh（新增）、tools/package.sh、README.md、spec

### Git Commits

| Hash | Message |
|------|---------|
| `4fa2bf5` | chore: 新机器工具链自检与文档 [task:11-06-rust-toolchain-guard] |

### Testing

- [OK] 自检通过；无 rustup 环境退出码 1 并给指引；打包脚本自检后正常继续

### Status

[OK] **Completed**


## Session 104: 写清工具链安装布局
<!-- trellis-session: v=2 fp=8281757d71399d84 -->

**Date**: 2026-11-06
**Task**: 写清工具链安装布局
**Branch**: `main`

### Summary

核实：入口系统级（/opt/rust + /usr/local/bin 软链），工具链按 /root/.rustup 分用户；README 与 spec 补对照表与自动下载说明。

### Main Changes

- README.md、.trellis/spec/frontend/build-serve-deploy.md

### Git Commits

| Hash | Message |
|------|---------|
| `c82a2b3` | docs: 写清工具链安装布局 [task:11-06-rust-layout-doc] |

### Testing

- [OK] rustup show 显示 /root/.rustup；模拟其他用户首次运行触发 1.99.1 自动下载

### Status

[OK] **Completed**


## Session 105: Rust 改为系统级共享安装
<!-- trellis-session: v=2 fp=470970dcc16a1503 -->

**Date**: 2026-11-06
**Task**: Rust 改为系统级共享安装
**Branch**: `main`

### Summary

/usr/local/bin 的 rust 入口改为包装脚本固定 RUSTUP_HOME=/opt/rust/rustup，加 /etc/profile.d，全机共用 1.99.1；文档同步。

### Main Changes

- 机器级：/usr/local/bin 包装脚本、/etc/profile.d/rust.sh；仓库：README、spec

### Git Commits

| Hash | Message |
|------|---------|
| `fd18786` | docs: Rust 改为系统级共享安装 [task:11-06-system-wide-rust] |

### Testing

- [OK] root/其他用户/登录 shell 均 1.99.1 且不落用户家目录；fmt/clippy/test(98)/build 全绿

### Status

[OK] **Completed**


## Session 106: 发版 0.0.37
<!-- trellis-session: v=2 fp=cfcffc4ef9a75b2f -->

**Date**: 2026-11-06
**Task**: 发版 0.0.37
**Branch**: `main`

### Summary

无功能变化：Rust 固定 1.99.1、工具链自检脚本、package.sh 走 PATH、README 工具链章节；打包/tag/push/release/重启完成。

### Main Changes

- Cargo.toml / Cargo.lock 版本 0.0.37

### Git Commits

| Hash | Message |
|------|---------|
| `aae968b` | chore: 发版 0.0.37 [task:11-06-release-0037] |

### Testing

- [OK] fmt/clippy/test(98)；打包前自检 1.99.1；包 sha256 通过；Release 2 附件；线上 200

### Status

[OK] **Completed**


## Session 107: 补记：侧边栏方案 B 适配（11-01 遗留任务）
<!-- trellis-session: v=2 fp=f683345e2b61b1c3 -->

**Date**: 2026-11-10
**Task**: 补记：侧边栏方案 B 适配（11-01 遗留任务）
**Branch**: `main`

### Summary

补记 2026-11-01 未写入 journal 的会话：按用户方案 B 与最新参考图重做侧栏与标题区，最终实现以 d4df530 落地、5b4f160 同步文档；后续 11-02~11-04 继续迭代侧栏与版式。2026-11-10 接手复核无未提交残留后归档该任务。

### Main Changes

- vendor/patches/patch.css 侧栏/标题/装饰杠与版式按参考设计重做（+431 行）
- docs/blog-tree.md 新增；tools/package.sh 发布记录域名脱敏；spec 域名说明修正
- 任务目录归档至 archive/2026-11/，PRD 补收尾记录

### Git Commits

| Hash | Message |
|------|---------|
| `d4df530` | feat(ui): 侧栏与内容区按参考设计对齐、字体字重统一；发布记录域名脱敏 [task:11-01-sidebar-clean-adapt] |
| `5b4f160` | docs: 修正脱敏后自相矛盾的域名说明（示例域名与已下线域名的语义） [task:11-01-sidebar-clean-adapt] |

### Testing

- [OK] [OK] 接手复核：fmt/clippy/test(98)/release build 全绿；diff 冻结基线仅首页相对日期漂移

### Status

[OK] **Completed**
