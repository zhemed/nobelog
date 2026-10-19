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
        let root_url = crate::site::env_url().unwrap_or_else(|| SITE.url.to_string());
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

    pub fn page(&self, pathname: &str, assets: &PageAssets, base_url: &str) -> RenderedPage {
        let path = if pathname.is_empty() { "/" } else { pathname };
        let recent = self.recent_links();
        let top_tags = self.top_tag_links();
        let ctx = Ctx {
            assets,
            base_url,
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
                    html: templates::post_page(&ctx, post, &related),
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

        // 通用单页 /about/：路径段解码后直接查 pages 表，与 collect_paths 里的 encode 对称。
        // 以前这里只放行 `[A-Za-z0-9-]`，中文 / 下划线 slug 的单页虽然进了 sitemap 却永远 404；
        // 现在按同一套编码规则反解，命中与否只取决于 pages 表本身。
        // 保留名（固定路由 tags/categories/page/posts/archives，输出根文件 feed.xml/sitemap.xml/…）
        // 不走这里：避免遮挡上面的固定路由，也让出与构建产物同名的路径。
        if let Some(raw_slug) = path.strip_prefix('/').and_then(|r| r.strip_suffix('/'))
            && !raw_slug.is_empty()
            && !raw_slug.contains('/')
        {
            let slug = decode(raw_slug);
            if !content::is_reserved_page_slug(&slug)
                && let Some(page) = self.content.pages.get(&slug)
            {
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
                html: templates::feed_xml(&refs, base_url, SITE.description),
                kind: PageKind::Feed,
            };
        }

        if path == "/sitemap.xml" {
            let paths = self.collect_paths();
            return RenderedPage {
                status: 200,
                path: path.to_string(),
                html: templates::sitemap_xml(&paths, base_url),
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
            result.push(self.page(&path, assets, &self.root_url));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SitePage;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static SEQ: AtomicUsize = AtomicUsize::new(0);

    /// 独立临时 root（content/posts 与 content/pages 已建好）。
    fn temp_root(tag: &str) -> PathBuf {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "nobelog-routes-test-{}-{tag}-{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("content/posts")).unwrap();
        fs::create_dir_all(root.join("content/pages")).unwrap();
        root
    }

    fn write_file(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn assets() -> PageAssets {
        PageAssets {
            css: "main.css".to_string(),
            js: "main.js".to_string(),
        }
    }

    const BASE: &str = "https://example.com";

    /// sitemap（collect_paths）里的每个条目都必须能被 page() 200 命中。
    /// 以前 collect_paths 用 `encode(slug)` 收录单页，page() 的单页分支却只放行
    /// `[A-Za-z0-9-]`：中文 / 下划线 slug 的单页进了 sitemap，线上却是 404 死链。
    #[test]
    fn every_collected_path_renders_200() {
        let root = temp_root("sitemap");
        write_file(
            &root,
            "content/posts/one.md",
            "---\ntitle: 一篇文章\nslug: one\ndate: 2026-10-17 22:40:00\ntags: [运维]\ncategories: [备忘]\n---\n\n正文。\n",
        );
        write_file(
            &root,
            "content/pages/about.md",
            "---\ntitle: 关于\nslug: about\n---\n\n关于页。\n",
        );
        write_file(
            &root,
            "content/pages/cn.md",
            "---\ntitle: 关于我\nslug: 关于我\n---\n\n中文 slug 单页。\n",
        );
        write_file(
            &root,
            "content/pages/under.md",
            "---\ntitle: 归档页\nslug: 存档_归档\n---\n\n下划线 slug 单页。\n",
        );

        let site = Site::new(&root).unwrap();
        let assets = assets();
        let paths = site.collect_paths();
        for path in &paths {
            assert_eq!(
                site.page(path, &assets, BASE).status,
                200,
                "collect_paths 收录的 {path} 必须 200，否则 sitemap 指向死链"
            );
        }
        // 重复 slug 会让同一个 URL 在 sitemap 里出现两次；content 层已 fail-fast，
        // 这里再从 sitemap 侧守住"没有重复条目"。
        let mut unique = paths.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(
            unique.len(),
            paths.len(),
            "collect_paths 出现了重复路由：{paths:?}"
        );

        // 每个单页都必须真的走单页分支（而不是被同名固定路由或 404 吃掉）。
        for (slug, page) in &site.content.pages {
            let path = format!("/{}/", crate::url::encode(slug));
            assert!(
                paths.contains(&path),
                "单页路由 {path} 必须被 collect_paths 收录"
            );
            let rendered = site.page(&path, &assets, BASE);
            assert_eq!(rendered.status, 200, "单页 {path} 必须 200");
            assert_eq!(
                rendered.kind,
                PageKind::About,
                "单页 {path} 必须渲染成单页，而不是被固定路由抢走"
            );
            assert!(
                rendered.html.contains(&page.title),
                "单页 {path} 的标题应出现在产物里"
            );
        }
        assert_eq!(site.content.pages.len(), 3, "三个单页都应加载");
        let _ = fs::remove_dir_all(&root);
    }

    /// 保留路由（tags / categories / page / posts / archives）不会被单页抢走：
    /// content 层已拒绝这类 slug，这里再验证路由层的优先级（纵深防御）。
    #[test]
    fn reserved_routes_win_over_pages() {
        let root = temp_root("reserved");
        write_file(
            &root,
            "content/posts/one.md",
            "---\ntitle: 一篇文章\nslug: one\ntags: [运维]\n---\n\n正文。\n",
        );
        let mut content = content::load(&root).unwrap();
        // 绕过 content 层的 fail-fast，直接塞一个保留路由名，验证 page() 不会让它遮住固定路由。
        content.pages.insert(
            "tags".to_string(),
            SitePage {
                slug: "tags".to_string(),
                title: "冒充标签页".to_string(),
                html: "<p>冒充标签页正文</p>".to_string(),
            },
        );
        let site = Site {
            content,
            home_pages: vec![vec![0]],
            root_url: BASE.to_string(),
        };
        let assets = assets();

        let rendered = site.page("/tags/", &assets, BASE);
        assert_eq!(
            rendered.kind,
            PageKind::Taxonomy,
            "/tags/ 只能是标签索引，不能被同名单页遮住"
        );
        assert!(
            !rendered.html.contains("冒充标签页正文"),
            "保留路由被单页抢走了"
        );
        assert_eq!(
            site.page("/archives/", &assets, BASE).kind,
            PageKind::Archive
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// 放宽单页匹配后，未知 slug 与多段路径依然必须 404（不能把任意路径都当单页）。
    #[test]
    fn unknown_and_multi_segment_page_paths_still_404() {
        let root = temp_root("unknown-page");
        write_file(
            &root,
            "content/pages/about.md",
            "---\ntitle: 关于\nslug: about\n---\n\n关于页。\n",
        );
        let site = Site::new(&root).unwrap();
        let assets = assets();
        for path in [
            "/nope/",
            "/about/extra/",
            "/关于我没写/",
            "/posts/",
            "/page/",
            "/page/0/",
            "/page/999/",
        ] {
            assert_eq!(
                site.page(path, &assets, BASE).status,
                404,
                "{path} 不应命中任何路由"
            );
        }
        assert_eq!(site.page("/about/", &assets, BASE).status, 200);
        let _ = fs::remove_dir_all(&root);
    }
}
