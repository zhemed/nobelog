# 内容与 frontmatter

内容源只有 `content/`：文章在 `content/posts/`，单页在 `content/pages/`，图片在 `content/images/`。
解析逻辑全部在 `rust/src/content.rs`，中间表示在 `rust/src/model.rs`。

## 文章 frontmatter

```markdown
---
title: 全要素排版测试
date: 2026-10-17 22:40:00
slug: typeset-test
tags: [排版, 测试]
categories: [调试]
description: 代码块、表格、提示块、图片、长文与 TOC 的一次性回归。
cover: /images/test-cover.svg
pinned: true
draft: false
updated: 2026-10-18 09:00:00
---
```

| 字段 | 类型 | 缺省行为 |
| --- | --- | --- |
| `title` | string | 回退到 `slug`（再回退到文件名 stem） |
| `date` | string | 无法解析时回退到文件 mtime；都没有则 Unix epoch |
| `slug` | string | 回退到文件名 stem（`my-post.md` → `my-post`） |
| `tags` / `categories` | 数组或逗号串 | 空数组；字符串会按 `,` 与中文 `，` 切分 |
| `description` | string | 空则取正文第一个 `<p>` 的纯文本前 160 个 UTF-16 单元 |
| `cover` | string | `None` |
| `pinned` | bool | `false`；`true` 时排在最前 |
| `draft` | bool | `false`；`true` 的文章在 `load()` 里**直接丢弃**（构建、动态模式、RSS、sitemap 全部看不到） |
| `updated` | string | `None`；有值时文章页展示更新时间 |

解析细节（`content.rs`，改动前必读）：

- **时区统一 UTC。** 文件头注释写明：构建机 `TZ=UTC`，Node 的 `new Date("YYYY-MM-DD HH:MM:SS")` 也按 UTC 解析。
  `parse_date` 先试 RFC3339，再试 `%Y-%m-%dT%H:%M:%S` / `%Y-%m-%dT%H:%M`，最后试纯日期；空格会被换成 `T`。
  **不要改成按服务器本地时区解析**，否则日期显示整体偏移。
- frontmatter 只认文件开头 `---` 到下一行 `\n---` 之间的块；没有 frontmatter 时整篇都是正文。
- 字符串字段接受 YAML 的 string/number/bool（`get_str`），数组字段也接受数字元素。
- 文章正文先 `trim()`，再交给 `markdown::render_shared` 渲染。

## 排序与聚合

```rust
// content.rs::load —— pinned 在前；其余按发布时间倒序（稳定排序，与 JS 一致）
posts.sort_by(|a, b| match (a.pinned, b.pinned) {
    (true, false) => std::cmp::Ordering::Less,
    (false, true) => std::cmp::Ordering::Greater,
    _ => b.date.cmp(&a.date),
});
```

- 头条（首页第一页第一张卡片）就是排序后的第一篇。
- `tags` / `categories` 用 `IndexMap`：**键的顺序 = 该标签/分类首次出现的顺序**，与 JS `Map` 语义一致；
  改回 `HashMap` 会让分类页、标签页的展示顺序随机化。
- 每个分类下的文章列表按日期倒序（`build_taxonomy` 内再排一次）。

## 摘要与阅读时长

- 摘要：`description` 非空直接用；否则 `first_paragraph` 取第一个 `<p>`，`strip_tags` 去掉标签，
  再 `slice_utf16(.., 160)` 截断（按 UTF-16 单元截，和 JS `String.slice` 对齐；不是按字节/字符）。
- 阅读时长：拉丁词数 `/220` + CJK 字数 `/400`（CJK 范围 `U+4E00..=U+9FA5`），四舍五入后 `max(1)`。
  改公式会让所有卡片上的"分钟"同步变化。

## 单页

- `content/pages/*.md`：`title` / `slug` 通过 `front_line` 用正则从 frontmatter 读（**不是完整 YAML 解析**），
  缺省都用文件名 stem。
- 单页渲染时每篇各自新建 renderer（`markdown::render`），可以并行；文章之间必须串行共享 renderer，
  因为标题 id 的 `slugCount` 要跨文章累积（见 markdown-pipeline.md）。

## 反模式

- 在 frontmatter 里写 `date: 2026-10-17 22:40:00 +08:00` 这种"本地时区"意图 → 会被当 UTC 解析，显示会差 8 小时。
- 给文章加"只在这个模板用"的字段却不改进 `Post` 结构 → 解析层直接忽略，静默无效。
- 把 `draft` 判断放到模板层 → 草稿会漏进首页分页/sitemap/RSS；必须在 `load()` 阶段过滤（现状即如此）。
