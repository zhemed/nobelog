# 侧栏导航放大并下移

## Goal

用户觉得侧栏导航偏小、且离标题太近，要求「再往下一点、再大一点」。

## 改动

```css
#sidebar .profile-wrapper { margin-bottom: 6.5rem !important; }
#sidebar .nav-link { font-size: 1.0625rem !important; padding: .75rem 0 !important; }
#sidebar .nav-link i { font-size: 1.0625rem !important; }
#sidebar li.nav-item { margin-bottom: .25rem; }
```

## 验证结果（2026-11-04，2048 视口）

| 项目 | 改前 | 改后 |
| --- | --- | --- |
| 导航字号 | 16px | **17px** |
| 单项高度 | 41px | **47px** |
| 项间距（节距） | 45px | **55px** |
| 标题块到导航 | 91px | **123px** |

`fmt` / `clippy` / `test` 98 全绿。
