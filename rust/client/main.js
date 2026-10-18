// rust/client/main.js
// src/client/main.ts 的直接翻译（去掉类型标注，语义逐行对应，零依赖）。
// 构建时由 rust/src/assets.rs 去除注释与缩进后输出为 assets/main.<hash>.js。
//   1. 亮暗三态（light / dark / 跟随系统）+ data-mode + localStorage
//   2. 移动端侧栏（sidebar-display 属性 + #mask 遮罩）
//   3. 回到顶部（#back-to-top）
//   4. 代码复制（.code-header > button）
//   5. 目录跟随 + 折叠（.toc-link / is-active-li / is-active-link）
//   6. 分享里的“复制链接”（#copy-link）

const doc = document.documentElement;
const body = document.body;

/* --- 1. 亮暗三态 ------------------------------------------------------- */
const MODE_KEY = "nobelog-mode";

const modeToggle = document.querySelector("#mode-toggle");

function storedMode() {
  try {
    const saved = localStorage.getItem(MODE_KEY);
    if (saved === "light" || saved === "dark") return saved;
  } catch {
    /* 隐私模式下 localStorage 可能不可用 */
  }
  return "auto";
}

function applyMode(mode) {
  // auto 时不写 data-mode，让主题 CSS 的 prefers-color-scheme 规则接管
  if (mode === "auto") doc.removeAttribute("data-mode");
  else doc.setAttribute("data-mode", mode);
  if (modeToggle) {
    modeToggle.title =
      mode === "dark" ? "暗色模式（点击：跟随系统）" : mode === "light" ? "亮色模式（点击：暗色）" : "跟随系统（点击：亮色）";
  }
}

applyMode(storedMode());

modeToggle?.addEventListener("click", () => {
  const now = storedMode();
  const next = now === "light" ? "dark" : now === "dark" ? "auto" : "light";
  try {
    if (next === "auto") localStorage.removeItem(MODE_KEY);
    else localStorage.setItem(MODE_KEY, next);
  } catch {
    /* 存不下就只在本次会话生效 */
  }
  applyMode(next);
});

/* --- 2. 移动端侧栏 ----------------------------------------------------- */
const closeSidebar = () => body.removeAttribute("sidebar-display");

document.querySelector("#sidebar-trigger")?.addEventListener("click", () => {
  if (body.hasAttribute("sidebar-display")) closeSidebar();
  else body.setAttribute("sidebar-display", "");
});
document.querySelector("#mask")?.addEventListener("click", closeSidebar);
document.querySelectorAll("#sidebar a").forEach((link) => link.addEventListener("click", closeSidebar));
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") closeSidebar();
});

/* --- 3. 回到顶部 ------------------------------------------------------- */
const backToTop = document.querySelector("#back-to-top");
if (backToTop) {
  const syncBackToTop = () => {
    backToTop.style.display = window.scrollY > 50 ? "block" : "none";
  };
  syncBackToTop();
  window.addEventListener("scroll", syncBackToTop, { passive: true });
  backToTop.addEventListener("click", () => window.scrollTo({ top: 0, behavior: "smooth" }));
}

/* --- 4. 代码复制 ------------------------------------------------------- */
function codeTextOf(button) {
  const scope = button.closest(".highlight, .highlighter-rouge, figure, pre") ?? button.parentElement;
  if (!scope) return null;
  // 新结构：.highlight > pre > code（旧的 rouge 结构仍兼容：td.rouge-code 优先）
  const node = scope.querySelector("td.rouge-code, pre code, code");
  return node ? node.innerText.replace(/\n$/, "") : null;
}

/**
 * 复制到剪贴板。
 * 注意：navigator.clipboard 只在**安全上下文**（https / localhost）存在；
 * 通过 http://<局域网IP>:10086 访问时它是 undefined —— 必须降级到 execCommand，
 * 否则就是“点了没反应、什么也复制不了”。
 */
async function copyText(text) {
  if (window.isSecureContext && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      /* 权限被拒等 → 走降级路径 */
    }
  }
  try {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.top = "-9999px";
    area.style.left = "-9999px";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    area.setSelectionRange(0, text.length);
    const ok = document.execCommand("copy");
    area.remove();
    return ok;
  } catch {
    return false;
  }
}

document.querySelectorAll("[data-copy]").forEach((button) => {
  button.addEventListener("click", async () => {
    const text = codeTextOf(button);
    if (text === null) return;
    const ok = await copyText(text);
    if (!ok) {
      // 不静默失败：提示 + 选中代码，至少能让用户手动 Ctrl+C
      console.warn("[nobelog] 复制失败：浏览器拒绝了剪贴板写入");
      const code = button.closest(".highlight")?.querySelector("code");
      if (code) {
        const range = document.createRange();
        range.selectNodeContents(code);
        const selection = window.getSelection();
        selection?.removeAllRanges();
        selection?.addRange(range);
      }
      return;
    }
    button.classList.add("is-copied");
    window.setTimeout(() => button.classList.remove("is-copied"), 2000);
  });
});

/* --- 5. 目录 ----------------------------------------------------------- */
const toc = document.querySelector("#toc");
if (toc) {
  const links = Array.from(toc.querySelectorAll("a.toc-link"));

  const items = links
    .map((link) => {
      const id = decodeURIComponent(link.getAttribute("href")?.slice(1) ?? "");
      const heading = id ? document.getElementById(id) : null;
      return heading ? { link, heading } : null;
    })
    .filter((item) => item !== null);

  // 折叠过深的层级（与实时 DOM 一致：H4 及更深的子树默认收起）
  toc.querySelectorAll("ul.toc-list.is-collapsible").forEach((list) => {
    let depth = 0;
    let node = list.parentElement;
    while (node && node.id !== "toc") {
      if (node.tagName === "UL") depth += 1;
      node = node.parentElement;
    }
    if (depth >= 2) list.classList.add("is-collapsed");
  });

  const expandAncestors = (link) => {
    let node = link.parentElement;
    while (node && node.id !== "toc") {
      if (node.tagName === "UL") node.classList.remove("is-collapsed");
      node = node.parentElement;
    }
  };

  const highlight = () => {
    if (items.length === 0) return;
    let active = items[0];
    for (const item of items) {
      if (item.heading.getBoundingClientRect().top <= 120) active = item;
    }
    items.forEach((item) => {
      const on = item === active;
      item.link.classList.toggle("is-active-link", on);
      item.link.parentElement?.classList.toggle("is-active-li", on);
    });
    expandAncestors(active.link);
  };

  // 点已收起的目录项时先展开它的子级
  links.forEach((link) => {
    link.addEventListener("click", () => {
      const children = link.parentElement?.querySelector("ul.toc-list.is-collapsed");
      if (children) children.classList.remove("is-collapsed");
    });
  });

  highlight();
  window.addEventListener("scroll", highlight, { passive: true });
}

/* --- 6. 分享：复制链接 ------------------------------------------------- */
const copyLink = document.querySelector("[data-copy-link]");
copyLink?.addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(window.location.href);
  } catch {
    return;
  }
  const glyph = copyLink.querySelector("i");
  if (!glyph) return;
  const prevClass = glyph.className;
  glyph.className = "fa-fw fas fa-check small";
  window.setTimeout(() => {
    glyph.className = prevClass;
  }, 2000);
});

/* --- 7. 图片查看：点击放大到全屏浮层 ------------------------------- */
function initLightbox() {
  const images = document.querySelectorAll(".markdown-body img");
  if (!images.length) return;

  const overlay = document.createElement("div");
  overlay.className = "lightbox";
  overlay.hidden = true;
  overlay.setAttribute("role", "dialog");
  overlay.setAttribute("aria-modal", "true");
  overlay.setAttribute("aria-label", "图片预览");

  const preview = document.createElement("img");
  preview.alt = "";
  overlay.appendChild(preview);
  document.body.appendChild(overlay);

  let lastFocused = null;

  const close = () => {
    if (overlay.hidden) return;
    overlay.hidden = true;
    preview.removeAttribute("src");
    document.body.classList.remove("lightbox-open");
    lastFocused?.focus?.();
  };

  const open = (src, alt) => {
    if (!src) return;
    lastFocused = document.activeElement;
    preview.src = src;
    preview.alt = alt || "";
    overlay.hidden = false;
    document.body.classList.add("lightbox-open");
  };

  overlay.addEventListener("click", close);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") close();
  });

  images.forEach((image) => {
    image.addEventListener("click", (event) => {
      const link = image.closest("a.img-link, a.popup");
      const href = link?.getAttribute("href") || "";
      // 只有链接本身指向图片时才用 href；普通链接里的图片仍显示图片本身
      const imageHref = /\.(png|jpe?g|gif|webp|svg|avif)([?#]|$)/i.test(href) ? href : "";
      const src = imageHref || image.currentSrc || image.src;
      event.preventDefault();
      event.stopPropagation();
      open(src, image.alt);
    });
  });
}
initLightbox();
