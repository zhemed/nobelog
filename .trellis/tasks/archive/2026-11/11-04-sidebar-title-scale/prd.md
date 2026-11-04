# 侧栏标题放大、与导航拉开距离

## Goal

对照 Chirpy 参考站：我们的站点标题偏小、标题块与导航贴得太近。

## 改动

```css
#sidebar .site-title { font-size: 1.75rem !important; }
#sidebar .site-subtitle { font-size: 1rem !important; }
#sidebar .profile-wrapper { margin-bottom: 4.5rem !important; }
```

## 验证结果（2026-11-04，2048 视口）

| 项目 | 改前 | 改后 |
| --- | --- | --- |
| 标题字号 | 25px | **28px** |
| 副标题字号 | 16px | 16px（保持） |
| 副标题到导航 | 51px | **91px** |

`fmt` / `clippy` / `test` 98 全绿。
