# 用户当轮原话（逐字记录）

> # 任务：清理构建产物中的「孤儿图标」，或为它们补上真实引用
>
> ## 一、先接受这些事实（实测结论，勿臆测）
> - 构建产物 `public/images/` 下共 **8 个**图标文件。
> - 其中 **5 个已被引用**，不算孤儿：
>   - `site-icon.svg`（`<link rel="icon" type="image/svg+xml">`）
>   - `site-icon-32-transparent.png`（`<link rel="icon" sizes="32x32">`）
>   - `site-icon-192-transparent.png`（`<link rel="icon" sizes="192x192">`）
>   - `site-icon-180.png`（`<link rel="apple-touch-icon" sizes="180x180">`）
>   - `site-icon-512.png`（被 `<meta property="og:image">` 引用，**不是孤儿**）
> - **3 个无任何引用**（在 `public/` 全目录内按文件名检索无命中，CSS 中也没有 `url(...)` 引用）：
>   - `site-icon-192-dark.png`
>   - `site-icon-32-dark.png`
>   - `site-icon-transparent-512.png`
> - 页面 HTML 里出现的 `dark` 字样来自 **JS 主题模式**，**不是图标引用** —— 不要据此认为暗色图标已经生效。
> - 因此这 3 个文件**每次构建都会被发布，却没有任何引用点**，属「孤儿产物」。
>
> ## 二、你要做的（二选一，并在 PRD 中说明取舍）
> **方案 ①：确认不需要 → 停止生成/提交**
> - 从图标生成脚本、源图与模板/资源清单中移除这 3 个文件，确保构建产物不再出现它们。
> - 取舍：产物最干净；代价是将来若要做暗色适配需重新生成。
>
> **方案 ②：本意是暗色模式图标 → 补上真实引用并验证**
> - 为暗色变体补 `<link rel="icon" ... media="(prefers-color-scheme: dark)">`，或在 SVG 内用媒体查询处理，
>   确保亮/暗两种系统主题下标签页图标都正确。
> - 若 `site-icon-transparent-512.png` 并非暗色用途，请明确其真实用途并补引用，或一并删除。
> - 取舍：功能更完整；但**必须实测**两种主题，不能只加标签了事。
>
> ## 三、约束（必须遵守）
> - **不得破坏**现有 4 条图标声明与 `og:image`；改完逐条验证路径仍 200 且 `content-type` 正确。
> - 若选方案 ②，**必须实测**两种主题（至少用开发者工具模拟 `prefers-color-scheme: dark`）。
> - 遵循仓库 Trellis 流程：**先建任务并写 PRD**；提交信息带 `[task:<slug>]`；不要先改后补文档。
> - 文档与提交信息中**不得出现真实域名、IP、端口**，用占位符 `<domain>`、`<port>`。
> - 只改与本任务相关的图标生成与引用逻辑，不要顺手重构其它部分。
>
> ## 四、验收标准（改完逐条自检并说明）
> 1. 构建后 `public/images/` 中**不存在无引用的图标**：要么不再生成，要么每个文件都有明确引用点（给出位置）。
> 2. 现有 4 条图标声明与 `og:image` 的路径全部 200，`content-type` 正确（给出实测输出）。
> 3. 若做了暗色适配：给出亮/暗两种主题下**实际加载的图标文件**证据。
> 4. 给出可复核的验证命令与结果（域名写成占位符）：`find public -iname '*icon*'`、
>    `grep -rl '<图标文件名>' public`、`curl -s -o /dev/null -w '%{http_code} %{content_type}\n' https://<domain>/images/<图标文件名>`。
> 5. 回复中列出：改了哪些文件、选择哪个方案及理由、每个图标的引用点（或"已停止生成"）。
