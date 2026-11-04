# 侧栏导航上移

## Goal

导航距标题过远，用户要求整体往上挪。

## 改动

`#sidebar .profile-wrapper { margin-bottom: 6.5rem → 5rem }`，其余不动。

## 验证结果（2048 视口）

| 项目 | 改前 | 改后 |
| --- | --- | --- |
| 副标题 → 导航 | 123px | **99px** |

`fmt` / `clippy` / `test` 98 全绿。
