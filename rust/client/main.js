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
copyLink?.addEventListener("click", async (event) => {
  event.preventDefault();
  const glyph = copyLink.querySelector("i");
  const prevClass = glyph?.className;
  const text = window.location.href;

  const feedback = (ok) => {
    if (glyph) {
      glyph.className = ok ? "fa-fw fas fa-check small" : "fa-fw fas fa-xmark small";
    }
    copyLink.setAttribute("title", ok ? "已复制" : "复制失败");
    window.setTimeout(() => {
      if (glyph && prevClass) glyph.className = prevClass;
      copyLink.setAttribute("title", "复制链接");
    }, 2000);
  };

  // 安全上下文用 Clipboard API
  if (window.isSecureContext && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      feedback(true);
      return;
    } catch {
      // 权限被拒时继续走回退方案
    }
  }

  // 非 HTTPS（如局域网 http://IP）没有 Clipboard API，用隐藏 textarea 回退
  try {
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    area.setSelectionRange(0, text.length);
    const ok = document.execCommand("copy");
    area.remove();
    feedback(ok);
  } catch {
    feedback(false);
  }
});

/* --- 7. 图片查看：点击放大到全屏浮层（缩放/拖拽/双指） ----------------- */
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
  preview.draggable = false;
  overlay.appendChild(preview);
  document.body.appendChild(overlay);

  const MIN_SCALE = 1;
  const MAX_SCALE = 4;
  const DOUBLE_TAP_SCALE = 2.5;

  let lastFocused = null;
  let scale = 1;
  let translateX = 0;
  let translateY = 0;
  let suppressClick = false;
  let drag = null;
  let pinch = null;
  const pointers = new Map();

  const clampScale = (value) => Math.min(MAX_SCALE, Math.max(MIN_SCALE, value));

  function applyTransform() {
    if (scale === 1 && translateX === 0 && translateY === 0) {
      overlay.classList.remove("lightbox-zoomed");
      preview.style.transform = "";
    } else {
      overlay.classList.add("lightbox-zoomed");
      preview.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scale})`;
    }
  }

  function centerOf() {
    const rect = preview.getBoundingClientRect();
    return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2, w: rect.width, h: rect.height };
  }

  // 限制平移范围，别把图拖丢（用未缩放的布局尺寸计算，避免读到旧 transform）
  function clampTranslate() {
    if (scale <= 1) {
      translateX = 0;
      translateY = 0;
      return;
    }
    const box = overlay.getBoundingClientRect();
    const scaledWidth = preview.offsetWidth * scale;
    const scaledHeight = preview.offsetHeight * scale;
    const maxX = Math.max(0, (scaledWidth - box.width) / 2) + 24;
    const maxY = Math.max(0, (scaledHeight - box.height) / 2) + 24;
    translateX = Math.min(maxX, Math.max(-maxX, translateX));
    translateY = Math.min(maxY, Math.max(-maxY, translateY));
  }

  function resetZoom() {
    scale = 1;
    translateX = 0;
    translateY = 0;
    drag = null;
    pinch = null;
    pointers.clear();
    applyTransform();
  }

  // 缩放并保持屏幕点 (x, y) 下的内容不动
  function zoomAt(next, x, y) {
    next = clampScale(next);
    if (next === scale) return;
    const c = centerOf();
    const k = next / scale;
    translateX += (x - c.x) * (1 - k);
    translateY += (y - c.y) * (1 - k);
    scale = next;
    if (scale === 1) {
      resetZoom();
      return;
    }
    clampTranslate();
    applyTransform();
  }

  const close = () => {
    if (overlay.hidden) return;
    overlay.hidden = true;
    preview.removeAttribute("src");
    resetZoom();
    document.body.classList.remove("lightbox-open");
    lastFocused?.focus?.();
  };

  const open = (src, alt) => {
    if (!src) return;
    lastFocused = document.activeElement;
    resetZoom();
    preview.src = src;
    preview.alt = alt || "";
    overlay.hidden = false;
    document.body.classList.add("lightbox-open");
  };

  overlay.addEventListener("click", (event) => {
    if (suppressClick) {
      suppressClick = false;
      return;
    }
    // 放大后点图片本身不关闭，只有点背景才关，避免拖拽/查看时误触
    if (event.target === preview && scale > 1) return;
    close();
  });

  // 滚轮缩放，锚点在鼠标位置
  overlay.addEventListener(
    "wheel",
    (event) => {
      if (overlay.hidden) return;
      event.preventDefault();
      zoomAt(scale * Math.exp(-event.deltaY * 0.0015), event.clientX, event.clientY);
    },
    { passive: false },
  );

  // 双击 1x ↔ 2.5x
  preview.addEventListener("dblclick", (event) => {
    event.preventDefault();
    zoomAt(scale > 1 ? 1 : DOUBLE_TAP_SCALE, event.clientX, event.clientY);
  });

  // 指针：单指/鼠标拖拽平移，双指捏合缩放
  overlay.addEventListener("pointerdown", (event) => {
    if (overlay.hidden) return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    try {
      overlay.setPointerCapture?.(event.pointerId);
    } catch {
      // 合成事件/异常指针没有可捕获的 pointerId，忽略即可
    }

    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()];
      const c = centerOf();
      pinch = {
        startScale: scale,
        startTranslateX: translateX,
        startTranslateY: translateY,
        startDistance: Math.hypot(a.x - b.x, a.y - b.y) || 1,
        startMidX: (a.x + b.x) / 2,
        startMidY: (a.y + b.y) / 2,
        center: c,
      };
      drag = null;
      return;
    }

    if (pointers.size === 1 && scale > 1) {
      drag = {
        x: event.clientX,
        y: event.clientY,
        startTranslateX: translateX,
        startTranslateY: translateY,
        moved: false,
      };
    }
  });

  overlay.addEventListener("pointermove", (event) => {
    if (!pointers.has(event.pointerId)) return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });

    if (pinch && pointers.size >= 2) {
      const [a, b] = [...pointers.values()];
      const distance = Math.hypot(a.x - b.x, a.y - b.y) || 1;
      const nextScale = clampScale((pinch.startScale * distance) / pinch.startDistance);
      const k = nextScale / pinch.startScale;
      const midX = (a.x + b.x) / 2;
      const midY = (a.y + b.y) / 2;
      scale = nextScale;
      if (scale === 1) {
        resetZoom();
        return;
      }
      translateX =
        pinch.startTranslateX + (pinch.startMidX - pinch.center.x) * (1 - k) + (midX - pinch.startMidX);
      translateY =
        pinch.startTranslateY + (pinch.startMidY - pinch.center.y) * (1 - k) + (midY - pinch.startMidY);
      clampTranslate();
      applyTransform();
      return;
    }

    if (drag && pointers.size === 1) {
      const dx = event.clientX - drag.x;
      const dy = event.clientY - drag.y;
      if (Math.abs(dx) + Math.abs(dy) > 4) drag.moved = true;
      translateX = drag.startTranslateX + dx;
      translateY = drag.startTranslateY + dy;
      clampTranslate();
      applyTransform();
    }
  });

  const endPointer = (event) => {
    if (!pointers.has(event.pointerId)) return;
    pointers.delete(event.pointerId);
    if (pointers.size < 2) pinch = null;
    if (pointers.size === 0) {
      if (drag?.moved) suppressClick = true;
      drag = null;
    }
  };
  overlay.addEventListener("pointerup", endPointer);
  overlay.addEventListener("pointercancel", endPointer);

  // 键盘：Esc 关闭，+/- 缩放，0 复位
  document.addEventListener("keydown", (event) => {
    if (overlay.hidden) return;
    if (event.key === "Escape") {
      close();
      return;
    }
    if (event.key === "+" || event.key === "=" || event.key === "-" || event.key === "0") {
      event.preventDefault();
      const box = overlay.getBoundingClientRect();
      if (event.key === "0") {
        resetZoom();
      } else {
        zoomAt(scale * (event.key === "-" ? 0.8 : 1.25), box.left + box.width / 2, box.top + box.height / 2);
      }
    }
  });

  images.forEach((image) => {
    image.addEventListener("click", (event) => {
      const link = image.closest("a.img-link, a.popup");
      const href = link?.getAttribute("href") || "";
      const imageHref = /\.(png|jpe?g|gif|webp|svg|avif)([?#]|$)/i.test(href) ? href : "";
      const src = imageHref || image.currentSrc || image.src;
      event.preventDefault();
      event.stopPropagation();
      open(src, image.alt);
    });
  });
}
initLightbox();
