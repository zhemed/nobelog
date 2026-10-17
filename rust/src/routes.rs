// 路由表 + 静态生成。每条路由都是纯函数：给定路径产出完整 HTML。
// 对齐 server/routes.ts。

use crate::content;
use crate::model::{Content, Post};
use crate::site::SITE;
use crate::templates::{self, Breadcrumb, Ctx, Link, PageAssets, RelatedLink, TagLink};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageKind {
    Home,
    Post,
    Archive,
    Taxonomy,
    About,
    Feed,
    Sitemap,
    NotFound,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // kind 供构建统计/测试读取
pub struct RenderedPage {
    pub status: u16,
    pub path: String,
    pub html: String,
    pub kind: PageKind,
}

pub struct Site {
    pub content: Content,
    pub home_pages: Vec<Vec<usize>>,
    pub root_url: String,
}

impl Site {
    pub fn new(root: &Path) -> Result<Self> {
        let content = content::load(root)?;
        let per = SITE.posts_per_page.max(1);
        let mut home_pages: Vec<Vec<usize>> = Vec::new();
        let mut current: Vec<usize> = Vec::new();
        for index in 0..content.posts.len() {
            current.push(index);
            if current.len() == per {
                home_pages.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            home_pages.push(current);
        }
        if home_pages.is_empty() {
            home_pages.push(Vec::new());
        }
        let root_url = std::env::var("SITE_URL")
            .unwrap_or_else(|_| SITE.url.to_string())
            .trim_end_matches('/')
            .to_string();
        Ok(Self {
            content,
            home_pages,
            root_url,
        })
    }

    fn refs(&self, indices: &[usize]) -> Vec<&Post> {
        indices.iter().map(|i| &self.content.posts[*i]).collect()
    }

    fn recent_links(&self) -> Vec<Link> {
        self.content
            .posts
            .iter()
            .take(5)
            .map(|p| Link {
                title: p.title.clone(),
                url: templates::post_url(p),
            })
            .collect()
    }

    fn top_tag_links(&self) -> Vec<TagLink> {
        let mut entries: Vec<(&String, &Vec<usize>)> = self.content.tags.iter().collect();
        entries.sort_by_key(|(_, list)| std::cmp::Reverse(list.len()));
        entries
            .into_iter()
            .take(10)
            .map(|(name, list)| TagLink {
                name: name.clone(),
                url: templates::tag_url(name),
                count: list.len(),
            })
            .collect()
    }

    pub fn page(&self, pathname: &str, assets: &PageAssets) -> RenderedPage {
        let path = if pathname.is_empty() { "/" } else { pathname };
        let recent = self.recent_links();
        let top_tags = self.top_tag_links();
        let ctx = Ctx {
            assets,
            path,
            recent: &recent,
            top_tags: &top_tags,
        };
        let not_found = |p: &str| RenderedPage {
            status: 404,
            path: p.to_string(),
            html: templates::not_found_page(&ctx, p),
            kind: PageKind::NotFound,
        };

        if path == "/" {
            let refs = self.refs(&self.home_pages[0]);
            return RenderedPage {
                status: 200,
                path: path.to_string(),
                html: templates::home_page(&ctx, &refs, 1, self.home_pages.len()),
                kind: PageKind::Home,
            };
        }

        if let Some(rest) = path
            .strip_prefix("/page/")
            .and_then(|r| r.strip_suffix('/'))
        {
            if let Ok(index) = rest.parse::<usize>()
                && index >= 1
                && index <= self.home_pages.len()
            {
                let refs = self.refs(&self.home_pages[index - 1]);
                return RenderedPage {
                    status: 200,
                    path: path.to_string(),
                    html: templates::home_page(&ctx, &refs, index, self.home_pages.len()),
                    kind: PageKind::Home,
                };
            }
            return not_found(path);
        }

        if let Some(slug) = path
            .strip_prefix("/posts/")
            .and_then(|r| r.strip_suffix('/'))
        {
            let slug = decode(slug);
            if let Some(post) = self.content.posts.iter().find(|p| p.slug == slug) {
                let mut chronological = self.content.posts.iter().collect::<Vec<_>>();
                chronological.sort_by_key(|p| p.date);
                let at = chronological.iter().position(|p| p.slug == slug);
                let (prev, next) = match at {
                    Some(i) => (
                        if i > 0 {
                            Some(chronological[i - 1])
                        } else {
                            None
                        },
                        chronological.get(i + 1).copied(),
                    ),
                    None => (None, None),
                };
                let related: Vec<RelatedLink> = self
                    .content
                    .posts
                    .iter()
                    .filter(|p| {
                        p.slug != post.slug
                            && p.categories.iter().any(|c| post.categories.contains(c))
                    })
                    .take(3)
                    .map(|p| RelatedLink {
                        title: p.title.clone(),
                        url: templates::post_url(p),
                        date: p.date,
                        summary: p.summary.clone(),
                    })
                    .collect();
                return RenderedPage {
                    status: 200,
                    path: path.to_string(),
                    html: templates::post_page(&ctx, post, &related, prev, next),
                    kind: PageKind::Post,
                };
            }
            return not_found(path);
        }

        if path == "/archives/" {
            let refs: Vec<&Post> = self.content.posts.iter().collect();
            return RenderedPage {
                status: 200,
                path: path.to_string(),
                html: templates::archive_page(&ctx, &refs),
                kind: PageKind::Archive,
            };
        }

        if path == "/tags/" {
            return RenderedPage {
                status: 200,
                path: path.to_string(),
                html: templates::taxonomy_index(
                    &ctx,
                    "tags",
                    &self.content.tags,
                    &self.content.posts,
                ),
                kind: PageKind::Taxonomy,
            };
        }

        if path == "/categories/" {
            return RenderedPage {
                status: 200,
                path: path.to_string(),
                html: templates::taxonomy_index(
                    &ctx,
                    "categories",
                    &self.content.categories,
                    &self.content.posts,
                ),
                kind: PageKind::Taxonomy,
            };
        }

        if let Some(name) = path
            .strip_prefix("/tags/")
            .and_then(|r| r.strip_suffix('/'))
        {
            let name = decode(name);
            if let Some(list) = self.content.tags.get(&name) {
                let refs = self.refs(list);
                let crumb = [
                    Breadcrumb {
                        name: "标签".into(),
                        href: Some("/tags/".into()),
                    },
                    Breadcrumb {
                        name: name.clone(),
                        href: None,
                    },
                ];
                return RenderedPage {
                    status: 200,
                    path: path.to_string(),
                    html: templates::list_page(&ctx, &format!("标签：{name}"), &refs, &crumb),
                    kind: PageKind::Taxonomy,
                };
            }
            return not_found(path);
        }

        if let Some(name) = path
            .strip_prefix("/categories/")
            .and_then(|r| r.strip_suffix('/'))
        {
            let name = decode(name);
            if let Some(list) = self.content.categories.get(&name) {
                let refs = self.refs(list);
                let crumb = [
                    Breadcrumb {
                        name: "分类".into(),
                        href: Some("/categories/".into()),
                    },
                    Breadcrumb {
                        name: name.clone(),
                        href: None,
                    },
                ];
                return RenderedPage {
                    status: 200,
                    path: path.to_string(),
                    html: templates::list_page(&ctx, &format!("分类：{name}"), &refs, &crumb),
                    kind: PageKind::Taxonomy,
                };
            }
            return not_found(path);
        }

        // 通用单页 /about/、/me/
        if let Some(raw_slug) = path.strip_prefix('/').and_then(|r| r.strip_suffix('/'))
            && !raw_slug.is_empty()
            && raw_slug
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            let slug = decode(raw_slug);
            if let Some(page) = self.content.pages.get(&slug) {
                return RenderedPage {
                    status: 200,
                    path: path.to_string(),
                    html: templates::about_page(&ctx, &page.title, &page.html),
                    kind: PageKind::About,
                };
            }
        }

        if path == "/feed.xml" {
            let refs: Vec<&Post> = self.content.posts.iter().collect();
            return RenderedPage {
                status: 200,
                path: path.to_string(),
                html: templates::feed_xml(&refs, &self.root_url, SITE.description),
                kind: PageKind::Feed,
            };
        }

        if path == "/sitemap.xml" {
            let paths = self.collect_paths();
            return RenderedPage {
                status: 200,
                path: path.to_string(),
                html: templates::sitemap_xml(&paths, &self.root_url),
                kind: PageKind::Sitemap,
            };
        }

        not_found(path)
    }

    pub fn collect_paths(&self) -> Vec<String> {
        let mut paths = vec![
            "/".to_string(),
            "/archives/".to_string(),
            "/tags/".to_string(),
            "/categories/".to_string(),
        ];
        for i in 2..=self.home_pages.len() {
            paths.push(format!("/page/{i}/"));
        }
        for post in &self.content.posts {
            paths.push(templates::post_url(post));
        }
        for tag in self.content.tags.keys() {
            paths.push(format!("/tags/{}/", encode(tag)));
        }
        for cat in self.content.categories.keys() {
            paths.push(format!("/categories/{}/", encode(cat)));
        }
        for slug in self.content.pages.keys() {
            paths.push(format!("/{}/", encode(slug)));
        }
        paths
    }

    pub fn all_routes(&self, assets: &PageAssets) -> Vec<RenderedPage> {
        let mut result: Vec<RenderedPage> = Vec::new();
        let mut paths = self.collect_paths();
        paths.push("/feed.xml".to_string());
        paths.push("/sitemap.xml".to_string());
        for path in paths {
            result.push(self.page(&path, assets));
        }
        result
    }
}

fn decode(value: &str) -> String {
    crate::url::decode(value)
}

fn encode(value: &str) -> String {
    crate::url::encode(value)
}
