// 站点配置：对齐 src/site.ts（内容必须逐字一致，模板会原样输出）。

pub struct SocialLink {
    pub name: &'static str,
    pub href: &'static str,
    pub icon: &'static str,
}

pub struct NavItem {
    pub name: &'static str,
    pub href: &'static str,
    pub icon: &'static str,
}

#[allow(dead_code)] // 与 src/site.ts 同构
pub struct SiteConfig {
    pub url: &'static str,
    pub lang: &'static str,
    pub feed_lang: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub description: &'static str,
    pub author: &'static str,
    pub avatar: &'static str,
    pub email: &'static str,
    pub github: &'static str,
    pub icp: &'static str,
    pub social: &'static [SocialLink],
    pub nav: &'static [NavItem],
    pub posts_per_page: usize,
    pub license_name: &'static str,
    pub license_url: &'static str,
    pub license_note: &'static str,
}

pub static SITE: SiteConfig = SiteConfig {
    url: "http://127.0.0.1:10086",
    lang: "zh-CN",
    feed_lang: "zh-CN",
    title: "吱吱",
    subtitle: "冷知识",
    description: "吱吱的冷知识：记录有用没用的小知识。",
    author: "",
    avatar: "",
    email: "",
    github: "",
    icp: "",
    social: &[
        SocialLink {
            name: "GitHub",
            href: "",
            icon: "fab fa-github",
        },
        SocialLink {
            name: "X",
            href: "",
            icon: "fab fa-x-twitter",
        },
        SocialLink {
            name: "邮件",
            href: "",
            icon: "fas fa-envelope",
        },
        SocialLink {
            name: "RSS",
            href: "",
            icon: "fas fa-rss",
        },
    ],
    nav: &[
        NavItem {
            name: "首页",
            href: "/",
            icon: "fa-fw fa-solid fa-home",
        },
        NavItem {
            name: "分类",
            href: "/categories/",
            icon: "fa-fw fas fa-bars-staggered",
        },
        NavItem {
            name: "标签",
            href: "/tags/",
            icon: "fa-fw fas fa-tag",
        },
        NavItem {
            name: "归档",
            href: "/archives/",
            icon: "fa-fw fas fa-inbox",
        },
    ],
    posts_per_page: 8,
    license_name: "CC BY 4.0",
    license_url: "https://creativecommons.org/licenses/by/4.0/",
    license_note: "除特别声明外，本站文章均按 CC BY 4.0 协议授权。",
};
