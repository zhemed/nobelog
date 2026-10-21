# 清理孤儿图标：为暗色变体补真实引用，删除无用途备用图

## Goal

消除 `public/images/` 中的孤儿图标：让每个图标文件都有明确引用点，或停止生成。

## 方案选择与取舍

**采用方案 ②（补真实引用）+ 局部方案 ①（删除真正无用途的文件）**：

| 文件 | 处置 | 理由 |
|---|---|---|
| `site-icon-32-dark.png` | **补引用**：`<link rel="icon" sizes="32x32" media="(prefers-color-scheme: dark)">` | 它的本意就是暗色模式 favicon。当前只有 SVG 内部媒体查询在切换，而 SVG favicon 在部分浏览器（如 Safari）不受支持；补 PNG 媒体查询引用既消除孤儿，又给不支持 SVG 的浏览器真实暗色兜底 |
| `site-icon-192-dark.png` | **补引用**：`<link rel="icon" sizes="192x192" media="(prefers-color-scheme: dark)">` | 同上（高分屏暗色） |
| `site-icon-transparent-512.png` | **停止生成并删除** | 它**不是**暗色用途，只是早期「备用大图」，无任何引用点；og:image 用白底 512（社交卡需要不透明），保留透明版没有场景 |

取舍说明：暗色适配走「HTML 媒体查询引用」而非仅靠 SVG 内部切换——功能更完整、可被浏览器
真实加载与验证；代价是 head 多两条 link、产物多两个文件（但这两个文件本来就是我们要保留的暗色件）。
真正无用途的 `site-icon-transparent-512.png` 直接停生成，避免继续产生孤儿产物。

## 改动

1. `rust/src/templates.rs` head 增加两条（现有 5 条不动）：
   ```html
   <link rel="icon" type="image/png" sizes="32x32" media="(prefers-color-scheme: dark)" href="/images/site-icon-32-dark.png">
   <link rel="icon" type="image/png" sizes="192x192" media="(prefers-color-scheme: dark)" href="/images/site-icon-192-dark.png">
   ```
   测试断言同步（含 media 属性）。
2. `tools/make-icons.py`：停止生成 `site-icon-transparent-512.png`（从输出清单移除）。
3. `content/images/site-icon-transparent-512.png`：删除。
4. 不改其它图标、不动 og:image、不重构无关代码。

## 验收（逐条自检）

1. 构建后 `public/images/` 中 7 个图标**每个都有引用点**（给出 `grep -rl` 命中位置）；
   `site-icon-transparent-512.png` 不再出现。
2. 现有 4 条图标声明 + `og:image` 路径全部 200 且 `content-type` 正确（实测输出）。
3. 亮/暗两种主题下**实际加载的图标文件**证据：用 CDP `Emulation.setEmulatedMedia`
   模拟 `prefers-color-scheme: dark`，记录 `link[rel="icon"]` 的 media 匹配情况与
   浏览器实际请求（Resource Timing / Network）。
4. 提供可复核命令（域名/端口用占位符 `<domain>`、`<port>`）：
   `find public -iname '*icon*'`、`grep -rl '<file>' public`、
   `curl -s -o /dev/null -w '%{http_code} %{content_type}\n' http://<domain>:<port>/images/<file>`。
5. 提交信息带 `[task:10-21-icon-orphans]`；文档与提交信息不含真实域名/IP/端口。

## 约束

- 只改图标生成与引用逻辑；不重启本机 systemd 服务（用临时实例验证），上线/发版另行确认。
