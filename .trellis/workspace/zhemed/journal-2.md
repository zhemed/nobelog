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
