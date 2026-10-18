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

/* --- 7. 图片查看：点击放大到全屏浮层（支持缩放/拖拽/双指） ---------- */
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

  const MIN_ZOOM = 1;
  const MAX_ZOOM = 4;
  const DOUBLE_TAP_ZOOM = 2.5;

  let lastFocused = null;
  let baseWidth = 0; // 1x 时图片的实际显示宽度
  let zoom = 1;
  let suppressClick = false;
  let drag = null; // 鼠标拖拽
  let pinch = null; // 双指

  const clampZoom = (value) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, value));
  const distance = (a, b) => Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
  const midpoint = (a, b) => ({
    x: (a.clientX + b.clientX) / 2,
    y: (a.clientY + b.clientY) / 2,
  });

  function resetZoom() {
    zoom = 1;
    overlay.classList.remove("lightbox-zoomed");
    preview.style.width = "";
    preview.style.height = "";
    preview.style.flex = "";
    overlay.scrollLeft = 0;
    overlay.scrollTop = 0;
    drag = null;
    pinch = null;
  }

  // 缩放到 next，并尽量让屏幕点 (x, y) 下的图像内容保持不动
  function zoomAt(next, x, y) {
    next = clampZoom(next);
    if (next === zoom) return;
    if (!baseWidth) measure();
    if (!baseWidth) return;
    const before = preview.getBoundingClientRect();
    const u = before.width ? (x - before.left) / before.width : 0.5;
    const v = before.height ? (y - before.top) / before.height : 0.5;

    zoom = next;
    if (zoom === 1) {
      resetZoom();
      return;
    }

    overlay.classList.add("lightbox-zoomed");
    preview.style.flex = "0 0 auto";
    preview.style.width = `${Math.round(baseWidth * zoom)}px`;
    const after = preview.getBoundingClientRect();
    overlay.scrollLeft += after.left + u * after.width - x;
    overlay.scrollTop += after.top + v * after.height - y;
  }

  function measure() {
    if (overlay.hidden || zoom !== 1) return;
    const width =
      preview.clientWidth ||
      preview.getBoundingClientRect().width ||
      preview.naturalWidth;
    if (width > 0) baseWidth = width;
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
    // 等布局完成后记录 1x 宽度
    if (preview.complete) {
      requestAnimationFrame(measure);
    } else {
      preview.addEventListener("load", () => requestAnimationFrame(measure), { once: true });
    }
  };

  // 点击背景关闭；拖拽后不误触
  overlay.addEventListener("click", () => {
    if (suppressClick) {
      suppressClick = false;
      return;
    }
    close();
  });

  // 滚轮缩放（锚点在鼠标位置）
  overlay.addEventListener(
    "wheel",
    (event) => {
      if (overlay.hidden) return;
      event.preventDefault();
      zoomAt(zoom * Math.exp(-event.deltaY * 0.0015), event.clientX, event.clientY);
    },
    { passive: false },
  );

  // 双击 1x ↔ 2.5x
  preview.addEventListener("dblclick", (event) => {
    event.preventDefault();
    zoomAt(zoom > 1 ? 1 : DOUBLE_TAP_ZOOM, event.clientX, event.clientY);
  });

  // 鼠标拖拽平移（图片放大后）
  overlay.addEventListener("pointerdown", (event) => {
    if (overlay.hidden || event.pointerType !== "mouse" || zoom <= 1) return;
    event.preventDefault();
    drag = {
      x: event.clientX,
      y: event.clientY,
      left: overlay.scrollLeft,
      top: overlay.scrollTop,
      moved: false,
    };
    overlay.setPointerCapture?.(event.pointerId);
  });

  overlay.addEventListener("pointermove", (event) => {
    if (!drag || event.pointerType !== "mouse") return;
    const dx = event.clientX - drag.x;
    const dy = event.clientY - drag.y;
    if (Math.abs(dx) + Math.abs(dy) > 4) drag.moved = true;
    overlay.scrollLeft = drag.left - dx;
    overlay.scrollTop = drag.top - dy;
  });

  const endDrag = (event) => {
    if (!drag || event.pointerType !== "mouse") return;
    if (drag.moved) suppressClick = true;
    drag = null;
  };
  overlay.addEventListener("pointerup", endDrag);
  overlay.addEventListener("pointercancel", endDrag);

  // 双指捏合（触摸）
  overlay.addEventListener(
    "touchstart",
    (event) => {
      if (overlay.hidden || event.touches.length !== 2) return;
      event.preventDefault();
      if (!baseWidth) measure();
      if (!baseWidth) return;
      const [a, b] = event.touches;
      const rect = preview.getBoundingClientRect();
      pinch = {
        startDistance: distance(a, b) || 1,
        startZoom: zoom,
        u: rect.width ? (midpoint(a, b).x - rect.left) / rect.width : 0.5,
        v: rect.height ? (midpoint(a, b).y - rect.top) / rect.height : 0.5,
      };
    },
    { passive: false },
  );

  overlay.addEventListener(
    "touchmove",
    (event) => {
      if (!pinch || event.touches.length < 2) return;
      event.preventDefault();
      const [a, b] = event.touches;
      const next = clampZoom((pinch.startZoom * distance(a, b)) / pinch.startDistance);
      if (next === 1) {
        resetZoom();
        return;
      }
      zoom = next;
      overlay.classList.add("lightbox-zoomed");
      preview.style.flex = "0 0 auto";
      preview.style.width = `${Math.round(baseWidth * zoom)}px`;
      const after = preview.getBoundingClientRect();
      const center = midpoint(a, b);
      overlay.scrollLeft += after.left + pinch.u * after.width - center.x;
      overlay.scrollTop += after.top + pinch.v * after.height - center.y;
    },
    { passive: false },
  );

  overlay.addEventListener("touchend", (event) => {
    if (event.touches.length < 2) pinch = null;
  });

  // 键盘：Esc 关闭，+/- 缩放，0 复位
  document.addEventListener("keydown", (event) => {
    if (overlay.hidden) return;
    if (event.key === "Escape") {
      close();
      return;
    }
    if (event.key === "+" || event.key === "=" || event.key === "-" || event.key === "0") {
      event.preventDefault();
      const rect = preview.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      if (event.key === "0") {
        resetZoom();
      } else {
        zoomAt(zoom * (event.key === "-" ? 0.8 : 1.25), centerX, centerY);
      }
    }
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
