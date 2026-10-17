// 数据模型：content → render → artifacts 的中间表示。
// 字段语义对齐 TypeScript 版 src/content.ts / server/routes.ts。

use chrono::{DateTime, Utc};
use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    pub level: usize,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // 字段与 TS 模型同构，部分仅用于内容层
pub struct Post {
    pub slug: String,
    pub title: String,
    pub date: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub description: String,
    pub cover: Option<String>,
    pub pinned: bool,
    pub draft: bool,
    pub body: String,
    pub file_path: String,
    pub html: String,
    pub toc: Vec<TocEntry>,
    pub summary: String,
    pub reading_minutes: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SitePage {
    pub slug: String,
    pub title: String,
    pub html: String,
}

/// tags / categories 用 IndexMap 保留"首次出现顺序"，与 JS Map 语义一致。
pub type Taxonomy = IndexMap<String, Vec<usize>>;

#[derive(Debug, Clone)]
pub struct Content {
    pub posts: Vec<Post>,
    pub tags: Taxonomy,
    pub categories: Taxonomy,
    pub pages: IndexMap<String, SitePage>,
}
