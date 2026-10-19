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

- **日期统一存「UTC+8 挂钟」。** `Post::date` 里装的是北京时间墙钟值（`Utc` 只是承载体，不代表时区）：
  - 裸写 `2026-10-17 22:40:00`、`2026-10-17T22:40:00+08:00`、`2026-10-17T14:40:00Z` 是同一时刻，
    解析后都等于挂钟 `2026-10-17 22:40:00`（RFC3339 分支先换算真实 UTC，再 `+8h` 转挂钟）。
  - 无 `date` 时回退文件 mtime，同样经 `wall_clock_from_system_time` 换算成挂钟（不要直接 `DateTime::from(mtime)`）。
  - `parse_date` 的顺序：RFC3339 → `%Y-%m-%dT%H:%M:%S` / `%Y-%m-%dT%H:%M` → 纯日期；
    `normalize_date` 把首个空白换成 `T` 并删除其余空白，所以 `2026-10-17 22:40:00 +08:00` 也能解析。
  - 展示层（`format_date` / `format_date_time` / `card_date`）直接读挂钟分量；**RSS 输出前再减 8 小时**
    还原真实 UTC（`templates::rss_pub_date`）。
  - **不要改成按服务器本地时区解析**，也不要新增"直接用 `DateTime<Utc>` 序列化"的输出点，
    否则日期会整体偏移 8 小时。
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

- 给文章加"只在这个模板用"的字段却不改进 `Post` 结构 → 解析层直接忽略，静默无效。
- 把 `draft` 判断放到模板层 → 草稿会漏进首页分页/sitemap/RSS；必须在 `load()` 阶段过滤（现状即如此）。
- 把挂钟值当真实 UTC 直接用（例如新增 JSON-LD、直接序列化 `Post::date`）→ 全站时间偏 8 小时；
  要 UTC 时先减 8 小时，或复用 `rss_pub_date` 的换算。
- 写 `date: 2026-10-18 09:00:00 UTC` 这类非 RFC3339 的时区后缀 → 会落到 naive 分支被当挂钟，语义不符预期。
