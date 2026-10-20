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

/// 部署时用 SITE_URL 覆盖站点绝对地址；未设置时回退到默认值。
pub fn env_url() -> Option<String> {
    std::env::var("SITE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
}

/// 启动时检查部署配置：`SITE_URL` 未设置就回退到内置默认地址（本机地址），
/// canonical / og:url / feed / sitemap 里的绝对链接会跟着指向本机。
/// 只告警一次，不 panic、不拒绝启动——开发态正依赖这个回退。
pub fn warn_if_env_url_missing() {
    if env_url().is_none() {
        eprintln!(
            "警告：未设置 SITE_URL，站点绝对链接回退到内置默认地址 {}；生产部署请设置 SITE_URL=https://你的域名。\
             动态模式默认只按 Host 头推断、不信任 X-Forwarded-*；若前面有可信反向代理，请设 NOBELOG_TRUST_FORWARDED=1。",
            SITE.url
        );
    }
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
        NavItem {
            name: "关于",
            href: "/about/",
            icon: "fa-fw fa-solid fa-circle-info",
        },
    ],
    posts_per_page: 8,
    license_name: "CC BY 4.0",
    license_url: "https://creativecommons.org/licenses/by/4.0/",
    license_note: "除特别声明外，本站文章均按 CC BY 4.0 协议授权。",
};
