# 核查 2.3.4 域名自动识别是否完善

## Goal

查清用户所指「2.3.4」是哪条需求或文档条目，核对「自动识别客户生产机域名」能力是否已经完善，并给出可复核结论。

## Requirements

1. **定位 2.3.4**：在仓库内搜索所有可能出处，包括 `.trellis/tasks` 归档与活跃任务、`.trellis/spec`、`.trellis/workflow.md`、journal、README、`content/`、`rust/src`、git 提交历史与提交消息；明确它属于哪个文档、任务或验收项。若存在多义，列出候选并给出最可能解释与依据。
2. **核对实现**：梳理当前域名识别链条，包括 `SITE_URL` 环境变量优先级、动态模式按 Host 推断、`X-Forwarded-*` 信任开关 `NOBELOG_TRUST_FORWARDED`、scheme 处理、静态构建回退 `SITE.url`，以及 canonical、og:url、og:image、RSS、sitemap、页脚等使用点。
3. **行为验证**：对本地 10086 服务做只读请求，覆盖默认 Host、自定义 Host、伪造 `X-Forwarded-Host` 与 `X-Forwarded-Proto`、无 `SITE_URL` 等组合，比对 canonical 等绝对地址输出；不重启服务、不新起实例。
4. **测试覆盖**：查找 `rust/src` 内与 SITE_URL、Host、转发头、canonical 相关的单元测试，说明覆盖了什么、没覆盖什么。
5. **结论**：明确回答 2.3.4 是否完成或完善；完善给证据，部分完善列出缺口与实际影响，重点说明客户生产机位于 TLS 反代后且未设 `SITE_URL` 的场景。
6. **报告落盘**：写入 `.trellis/tasks/10-30-check-domain-autodetect/research/domain-autodetect.md`。

## Acceptance Criteria

- [ ] 报告给出 2.3.4 的出处与原文摘录，或明确说明仓库内未找到并列出已检索范围。
- [ ] 报告给出实现链条的代码位置与关键行号。
- [ ] 报告给出本地行为验证矩阵与原始输出摘要。
- [ ] 报告给出单元测试覆盖情况。
- [ ] 报告给出明确结论：完善、部分完善或未完成，以及缺口清单。
- [ ] 全程只读：除 research 报告外无文件改动，无 git 写操作，无服务重启。

## Constraints

- 只读调查；禁止改代码、配置、spec 与任务文件。
- 不重启 `nobelog-local`，不新起服务实例，不安装依赖。
- 不访问未授权公网域名；本地 10086 与 GitHub 只读查询为授权范围。
- 报告写 `research/` 目录。

## Notes

- 用户原话：2.3.4是不是已经完善了，你去查一下，我记得已经好了呀，会自动识别客户生产机的域名。
- 前一轮只读核查曾把 `SITE_URL` 未设置列为风险，本轮重点核对自动识别是否已覆盖该风险。
