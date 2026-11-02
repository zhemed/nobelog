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
