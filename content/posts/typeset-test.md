---
title: 全要素排版测试
date: 2026-10-17 22:40:00
slug: typeset-test
tags: [排版, 测试]
categories: [调试]
description: 代码块、表格、提示块、图片、长文与 TOC 的一次性回归。
---

这是一篇用于本地调试的测试文章，覆盖博客常用的全部元素。上线前会删掉。

## 二级标题：文字样式

正文里有 **粗体**、*斜体*、`行内代码`、[普通链接](https://example.com)、~~删除线~~，以及中英文混排 Mixed Chinese and English with numbers 123456。

> 这是一段引用。引用应该和正文有明显区分，但又不至于抢戏。
>
> 第二行引用，用来测试多段落。

### 三级标题：列表

- 无序列表第一项
- 第二项，带 `inline code`
  - 嵌套项 A
  - 嵌套项 B
- 第三项，带链接 [吱吱](https://example.com)

1. 有序列表第一步
2. 第二步
3. 第三步

#### 四级标题：任务列表

- [x] 已完成的任务
- [ ] 未完成的任务

## 二级标题：代码块

```js
// JavaScript：测试行号、横向滚动与复制按钮
export function greet(name) {
  const message = `Hello, ${name}! 这是一行很长的中文注释，用来测试代码块在窄屏下是否会横向滚动而不是撑破布局。`;
  console.log(message);
  return message;
}
```

```rust
/// Rust：测试泛型与长行
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

```bash
# Shell：测试命令高亮
cargo build --release --locked
systemctl restart nobelog-local
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:10086/
```

```python
# Python
def fib(n: int) -> int:
    return n if n < 2 else fib(n - 1) + fib(n - 2)
```

```powershell
# 一个没有高亮支持的语言，应该安全退化为纯文本
Get-ChildItem -Recurse | Where-Object { $_.Length -gt 1MB }
```

## 二级标题：表格

| 项目 | 状态 | 说明 |
| --- | :---: | --- |
| 代码块 | ✅ | syntect 高亮 + 复制按钮 |
| 表格 | ✅ | 窄屏可横向滚动 |
| 提示块 | ✅ | tip / info / warning / danger |
| 图片 | ✅ | 点击放大 |
| RSS | ✅ | `/feed.xml` |

一个很宽的表格，用来测试横向滚动：

| 列一 | 列二 | 列三 | 列四 | 列五 | 列六 | 列七 | 列八 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 数据 | 数据 | 数据 | 数据 | 数据 | 数据 | 数据 | 数据 |
| 很长很长很长很长很长很长的内容 | 内容 | 内容 | 内容 | 内容 | 内容 | 内容 | 内容 |

## 二级标题：提示块

::: tip
提示块：适合放“建议这样做”的内容。
:::

::: info
信息块：补充说明，不打断阅读。
:::

::: warning
警告块：注意这里有坑。
:::

::: danger
危险块：会造成数据丢失的操作。
:::

## 二级标题：图片

![测试封面图](/images/test-cover.svg)

图片下方是一段说明文字，用来检查图片与正文的间距。

## 二级标题：长文与锚点

这是一段比较长的正文，用来测试阅读节奏和行高。排版的目标不是“好看”，而是让读者在长时间阅读时不容易疲劳：行宽不要过宽，行高要略大于字号，段间距要稳定，标题和正文之间要有明确的层级差。

这是第二段。中英文混排时，标点、数字和汉字之间的间距会直接影响观感。比如 “Rust 1.99.1” 和 “2026 年 10 月 17 日”，以及 10086 这个端口号。

### 三级标题：折叠上下文

- 列表中的代码：`cargo test --locked`
- 列表中的链接：[GitHub](https://github.com/zhemed/nobelog)

### 三级标题：结尾

如果上面这些都正常，说明排版管线是健康的；如果有哪里不对，就在这一篇里改。
