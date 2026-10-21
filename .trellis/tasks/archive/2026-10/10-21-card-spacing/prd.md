# 首页文章卡片底部留白调整（评估 + 效果预览）

## Goal

用户觉得首页文章卡片「日期 + 分类」行下方留白太少、卡片之间也偏紧。评估当前间距，加一档留白，
出**前后对比截图**让用户判断效果。本阶段只到预览，是否上线/发版另行确认。

## 现状（先量再改）

- 样式落点：`vendor/patches/patch.css`（本站补丁层，最后加载；**不回改上游 `vendor/chirpy/main.css`**）。
- 相关选择器：`#post-list .card .card-body`（内边距）、`#post-list .card .card-body .post-meta`（meta 行）、
  `#post-list .card-wrapper`（卡片间距，若存在）。
- 先用浏览器量出**当前计算值**（`padding-bottom`、`.post-meta` 的 `margin-bottom`、卡片间距），
  作为「改前」基线写进报告。

## 交付

1. 在 `vendor/patches/patch.css` 的「首页文章卡片」分区追加最小补丁（注释写清为什么上游覆盖不到），
   建议一档保守值（例如卡片底部内边距 +0.5rem、卡片间距 +0.5rem；具体按量出的基线定），
   让「meta 行 → 卡片底边」有明显但不夸张的留白。
2. 出**前后对比截图**（同一视口、同一页面、明色主题）：
   - 改前：当前线上/构建产物
   - 改后：加留白后的构建产物
   - 建议对首页卡片区域做等宽裁切并排，另附整页截图
3. 报告量化差异：改前/改后的 `padding-bottom`、卡片间距、以及列表总高度变化。
4. 门禁：`cargo fmt --check && cargo test && cargo clippy --all-targets`；
   记录 CSS 产物哈希变化（`main.<hash>.css`）。
5. 若用户后续要上线：重建 release + 重启一次（另行确认）。

## 约束

- 只动 `vendor/patches/patch.css`（以及必要的注释），不重构其它部分、不动上游 CSS。
- 不重启本机 systemd 服务（用临时实例预览）。
- 文档与提交信息中不出现真实域名/IP/端口，用 `<domain>`、`<port>` 占位。

## Acceptance

- [ ] 给出改前基线计算值与改后计算值（同选择器、同视口）。
- [ ] 前后对比截图可读，留白变化肉眼可辨但不突兀。
- [ ] 门禁全绿；CSS 哈希变化已记录；临时实例用完已关闭。
- [ ] 提交（带 `[task:10-21-card-spacing]`）、journal、归档。
