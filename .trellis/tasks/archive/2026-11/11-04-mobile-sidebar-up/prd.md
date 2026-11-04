# 手机端侧栏整体上移

## Goal

手机抽屉里标题块偏下，标题与导航占掉了上半屏，用户要求整体往上。

## 改动（仅 `≤849px` 生效）

```css
@media all and (max-width: 849px) {
  #sidebar .profile-wrapper {
    margin-top: 1rem !important;
    padding-top: 1.25rem !important;
    margin-bottom: 2.75rem !important;
  }
}
```

## 验证结果（390 视口）

| 项目 | 改前 | 改后 |
| --- | --- | --- |
| 标题块顶部 | 40px | **16px** |
| 标题顶部 | 84px | **36px** |
| 导航顶部 | 282px | **198px** |

桌面端（≥850）不受影响。`fmt` / `clippy` / `test` 98 全绿。
