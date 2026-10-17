// 全站路径常量，对齐 server/paths.ts。

pub const STYLE_ORDER: [&str; 4] = [
    "vendor/chirpy/main.css",
    "vendor/fonts/fonts.css",
    "vendor/github-markdown/github-markdown.css",
    "vendor/patches/patch.css",
];

/// 主题侧固定资产：文件名被主题 CSS 以 url(/assets/fa-*.woff2) 写死。
pub const VENDORED_ASSETS: [&str; 4] = [
    "fa-brands-400.woff2",
    "fa-regular-400.woff2",
    "fa-solid-900.woff2",
    "fa-v4compatibility.woff2",
];

/// public/ 中手工维护、清理时保留的文件。
pub const PRESERVE_IN_PUBLIC: [&str; 5] =
    ["CNAME", "_headers", "_redirects", "robots.txt", ".gitkeep"];
