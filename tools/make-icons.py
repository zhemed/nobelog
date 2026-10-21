#!/usr/bin/env python3
"""从 tools/assets/cat-icon-source.jpg 生成 nobelog 猫咪站点图标（可复跑）。

用法（仓库根或任意目录）：
    python3 tools/make-icons.py

源图是 1080x1080 白底黑线稿，猫只占中间一小条；直接缩放成 32px 会看不见，
所以流程是「阈值找暗像素 bbox → 裁到笔画 → 加约 6% 内边距 → 居中放进 1:1 方形画布
（猫占满宽度）→ 再缩放」。

产出（content/images，权限 644）：
    cat-icon-32.png    透明底黑线：alpha = 255 - 亮度，RGB 全黑（favicon 兜底）
    cat-icon-192.png   白底黑线（favicon 高分屏兜底）
    cat-icon-180.png   白底黑线（apple-touch-icon）
    cat-icon-512.png   白底黑线（og:image）
    cat-icon.svg       内嵌 base64 透明 PNG（192 源），暗色下 filter: invert(1) 把黑线变白线

预览专用（.run/review/cat-icon/，不入库）：cat-icon-16.png、cat-icon-48.png（透明版）。

改图标只需替换 tools/assets/cat-icon-source.jpg 后重跑本脚本；
新增尺寸时同步改常量，并确认 rust/src/templates.rs 与测试引用一致。
"""

import base64
import hashlib
import os
import shutil
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "tools/assets/cat-icon-source.jpg"
OUT = ROOT / "content/images"
PREVIEW = ROOT / ".run/review/cat-icon"

# 亮度低于该值算「笔画」，用于求 bbox（含抗锯齿灰边）。
INK_THRESHOLD = 200
# 方形画布每边留白比例（相对笔画 bbox 的长边）。
PADDING_RATIO = 0.06
# 白底不透明尺寸。
OPAQUE = [180, 192, 512]
# 透明 favicon 尺寸。
FAVICON = 32
# SVG 内嵌的透明 PNG 尺寸（192 够标签页用，且 SVG 体积小）。
SVG_RASTER = 192
# 仅预览用的透明尺寸。
PREVIEW_TRANSPARENT = [16, 48]


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def save_png(image, path):
    image.save(path, "PNG", optimize=True)
    os.chmod(path, 0o644)


def ink_bbox(image):
    """暗像素（亮度 < INK_THRESHOLD）的包围盒。"""
    gray = image.convert("L")
    mask = gray.point(lambda value: 255 if value < INK_THRESHOLD else 0)
    bbox = mask.getbbox()
    if bbox is None:
        raise SystemExit(f"{SRC} 里找不到暗像素，阈值 {INK_THRESHOLD} 可能过高")
    return bbox


def padded_square(image, bbox):
    """裁到 bbox，按长边加 6% 内边距，居中放进白底方形画布。"""
    left, top, right, bottom = bbox
    width, height = right - left, bottom - top
    side = int(round(max(width, height) * (1 + 2 * PADDING_RATIO)))
    canvas = Image.new("RGB", (side, side), (255, 255, 255))
    canvas.paste(image.crop(bbox), ((side - width) // 2, (side - height) // 2))
    return canvas


def transparent_from_rgb(image):
    """黑线透明版：alpha = 255 - 亮度，RGB 全黑（保留抗锯齿灰边为半透明）。"""
    alpha = image.convert("L").point(lambda value: 255 - value)
    black = Image.new("L", image.size, 0)
    return Image.merge("RGBA", (black, black, black, alpha))


def svg_with_embedded_png(raster, size):
    """把透明 PNG 内嵌成 base64，并在暗色媒体查询里反色。"""
    import io

    buffer = io.BytesIO()
    raster.save(buffer, "PNG", optimize=True)
    payload = base64.b64encode(buffer.getvalue()).decode("ascii")
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}" role="img" aria-label="\u732b\u54aa\u56fe\u6807">
  <style>
    /* 暗色标签页里把黑线反成白线。SVG 里的位图元素是 image（不是 HTML 的 img），
       两个选择器都写上：img 是任务约定的写法，image 才是真正生效的那个。 */
    @media (prefers-color-scheme: dark) {{
      img, image {{ filter: invert(1); }}
    }}
  </style>
  <image width="{size}" height="{size}" href="data:image/png;base64,{payload}"/>
</svg>
"""


def main():
    if not SRC.is_file():
        raise SystemExit(f"缺少源图：{SRC}")
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(PREVIEW, exist_ok=True)

    source = Image.open(SRC).convert("RGB")
    bbox = ink_bbox(source)
    square = padded_square(source, bbox)
    print(f"源图 {SRC.relative_to(ROOT)} {source.size[0]}x{source.size[1]} sha256={sha256(SRC)}")
    print(
        f"笔画 bbox={bbox} 尺寸={bbox[2] - bbox[0]}x{bbox[3] - bbox[1]}"
        f" → 方形画布 {square.size[0]}x{square.size[1]}（每边留白约 {PADDING_RATIO * 100:.0f}%）"
    )

    produced = []

    # 透明 favicon：先缩放 RGB 再转 alpha，避免透明像素污染边缘。
    favicon = transparent_from_rgb(square.resize((FAVICON, FAVICON), Image.LANCZOS))
    path = OUT / f"cat-icon-{FAVICON}.png"
    save_png(favicon, path)
    produced.append(path)

    for size in OPAQUE:
        path = OUT / f"cat-icon-{size}.png"
        save_png(square.resize((size, size), Image.LANCZOS), path)
        produced.append(path)

    raster = transparent_from_rgb(square.resize((SVG_RASTER, SVG_RASTER), Image.LANCZOS))
    svg_path = OUT / "cat-icon.svg"
    svg_path.write_text(svg_with_embedded_png(raster, SVG_RASTER), encoding="utf-8")
    os.chmod(svg_path, 0o644)
    produced.append(svg_path)

    # 预览目录引用同目录图标：先清旧副本，再放全量（含预览专用尺寸）。
    for stale in PREVIEW.glob("cat-icon-*"):
        stale.unlink()
    preview_files = []
    for path in produced:
        target = PREVIEW / path.name
        shutil.copy2(path, target)
        preview_files.append(target)
    for size in PREVIEW_TRANSPARENT:
        icon = transparent_from_rgb(square.resize((size, size), Image.LANCZOS))
        path = PREVIEW / f"cat-icon-{size}.png"
        save_png(icon, path)
        preview_files.append(path)

    print("\n入库产出（content/images）：")
    for path in produced:
        if path.suffix == ".svg":
            print(f"  {path.relative_to(ROOT)}  {path.stat().st_size} B  {oct(path.stat().st_mode & 0o777)}  sha256={sha256(path)[:12]}")
        else:
            with Image.open(path) as im:
                print(
                    f"  {path.relative_to(ROOT)}  {im.size[0]}x{im.size[1]} {im.mode}"
                    f"  {path.stat().st_size} B  {oct(path.stat().st_mode & 0o777)}"
                    f"  sha256={sha256(path)[:12]}"
                )
    print("\n预览产出（.run/review/cat-icon，不入库）：")
    for path in preview_files:
        if path.suffix == ".svg":
            print(f"  {path.relative_to(ROOT)}  {path.stat().st_size} B  sha256={sha256(path)[:12]}")
            continue
        with Image.open(path) as im:
            print(
                f"  {path.relative_to(ROOT)}  {im.size[0]}x{im.size[1]} {im.mode}"
                f"  {path.stat().st_size} B  sha256={sha256(path)[:12]}"
            )


if __name__ == "__main__":
    main()
