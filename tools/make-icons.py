#!/usr/bin/env python3
"""生成 nobelog 站点图标（可复跑，源图可选）。

用法（仓库根或任意目录）：
    python3 tools/make-icons.py                                    # 默认熊脸 tools/assets/bear-icon-source.jpg
    python3 tools/make-icons.py tools/assets/cat-icon-source.jpg   # 换回猫咪（一键切换）

输出前缀固定 `site-icon`，写进 content/images/（权限 644）：
    site-icon.svg                   内嵌「亮色 192 + 暗色 192」两张透明 PNG，
                                    用 @media (prefers-color-scheme: dark) 切换（favicon 首选）
    site-icon-32-transparent.png    透明底原色（favicon 回退，标签页不再有白框）
    site-icon-192-transparent.png   透明底原色（favicon 高分屏回退）
    site-icon-32-dark.png           暗色变体（透明底：身体深灰、描边五官近白、粉耳保留）
    site-icon-192-dark.png          暗色变体（同上，高分屏）
    site-icon-180.png               白底原色（apple-touch-icon 需要不透明）
    site-icon-512.png               白底原色（og:image）
    site-icon-transparent-512.png   透明底原色大图（备用）

流程：
    1. 用亮度阈值找「暗像素」（描边/五官）的 bbox；
    2. 裁到 bbox，按长边加约 5% 内边距，居中放进 1:1 方形画布（白底）；
    3. 按目标尺寸 LANCZOS 缩放。

透明版：把「与画布边缘相连的近白背景」用 flood fill 转成 alpha=0，被描边包围的内部
浅色（熊脸奶白、粉耳朵）保持不透明；背景与内容之间 3px 环带按「原图 = 前景 * a +
白 * (1-a)」反预乘，避免暗底上出现白边。注意：不能直接用 alpha = 255 - 亮度——
这张图的脸是奶白（min 通道约 228），那样会把整张脸洗成半透明。

暗色变体：在透明版基础上按像素重映射（保持 alpha）——
    近黑描边/眼睛/鼻子（亮度 < INK_LUMA）→ 近白 #f2f2f2；
    粉色耳朵（r-g 明显且够亮）→ 略提亮；
    其余（奶白身体）→ 深灰 #2b2b2b。
这样暗色标签页里轮廓与五官可见，而不是一块白方块。

预览专用（默认 .run/review/<源图名去掉 -source>/，可用 --preview-dir 指定，不入库）：
16 / 48 的亮色与暗色透明小图 + 全部产出副本。

换图只需替换 tools/assets 下的源图后重跑本脚本，或把别的源图路径作为参数传入。
"""

import argparse
import base64
import hashlib
import io
import os
import shutil
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = ROOT / "tools/assets/bear-icon-source.jpg"
OUT = ROOT / "content/images"
PREFIX = "site-icon"

# 亮度低于该值算「内容」（描边/五官/彩色块）；纯白背景不计入。
INK_THRESHOLD = 240
# 方形画布每边留白比例（相对 bbox 长边）。
PADDING_RATIO = 0.05
# 白底不透明尺寸（apple-touch / og:image 需要不透明底）。
OPAQUE = [180, 512]
# 透明底原色尺寸（favicon 回退）。
LIGHT_TRANSPARENT = [32, 192]
# 暗色变体尺寸（透明底）。
DARK_TRANSPARENT = [32, 192]
# 透明底原色大图（备用）。
BACKUP_TRANSPARENT = [512]
# 仅预览用的透明小图尺寸（亮色 / 暗色各一份）。
PREVIEW_LIGHT = [16, 48]
PREVIEW_DARK = [16, 48]
# SVG 内嵌的两张透明 PNG 尺寸。
SVG_SIZE = 192

# flood fill 容差（PIL 按通道差绝对值之和比较）：只吃纯白/近白背景，
# 奶白脸与白底的差约 37，留在容差外，且脸被描边包围，本来就填不进去。
FLOOD_THRESHOLD = 30
# 背景与内容之间的过渡环带宽度（像素）。
EDGE_RING = 3

# 暗色变体的映射目标。
DARK_BODY = (43, 43, 43)  # #2b2b2b：奶白身体 → 深灰
DARK_INK = (242, 242, 242)  # #f2f2f2：黑描边/眼睛/鼻子 → 近白
INK_LUMA = 90  # 亮度低于此值算「墨色」
PINK_MIN_RED = 140  # 粉耳判定：够亮
PINK_MIN_DIFF = 25  # 粉耳判定：r - g 明显大于 0（奶白脸 r-g 只有约 2）


def sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def display(path):
    """打印用路径：仓库内显示相对路径，仓库外（--out /tmp/...）显示绝对路径。"""
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def save_png(image, path):
    image.save(path, "PNG", optimize=True)
    os.chmod(path, 0o644)


def ink_bbox(image):
    """暗像素（亮度 < INK_THRESHOLD）的包围盒。"""
    gray = image.convert("L")
    mask = gray.point(lambda value: 255 if value < INK_THRESHOLD else 0)
    bbox = mask.getbbox()
    if bbox is None:
        raise SystemExit(f"图里找不到暗像素，阈值 {INK_THRESHOLD} 可能过高")
    return bbox


def padded_square(image, bbox):
    """裁到 bbox，按长边加约 5% 内边距，居中放进白底方形画布。"""
    left, top, right, bottom = bbox
    width, height = right - left, bottom - top
    side = int(round(max(width, height) * (1 + 2 * PADDING_RATIO)))
    canvas = Image.new("RGB", (side, side), (255, 255, 255))
    canvas.paste(image.crop(bbox), ((side - width) // 2, (side - height) // 2))
    return canvas


def background_to_alpha(image):
    """与边缘相连的近白背景转 alpha=0，内部浅色保持不透明，边缘环带反预乘。"""
    width, height = image.size
    sentinel = (255, 0, 255)
    work = image.copy()
    for seed in [(0, 0), (width - 1, 0), (0, height - 1), (width - 1, height - 1)]:
        if work.getpixel(seed) != sentinel:
            ImageDraw.floodfill(work, seed, sentinel, thresh=FLOOD_THRESHOLD)

    background = (
        ImageChops.difference(work, Image.new("RGB", (width, height), sentinel))
        .convert("L")
        .point(lambda value: 255 if value == 0 else 0)
    )
    if background.getbbox() is None:
        return image.convert("RGBA")

    opaque = Image.new("L", (width, height), 255)
    alpha = Image.composite(Image.new("L", (width, height), 0), opaque, background)

    ring = ImageChops.subtract(
        background.filter(ImageFilter.MaxFilter(2 * EDGE_RING + 1)), background
    )
    rgba = image.convert("RGBA")
    rgba.putalpha(alpha)

    # 环带像素是「描边/彩色 ↔ 白底」的抗锯齿混合，按白底反预乘保住边缘颜色。
    ring_box = ring.getbbox()
    if ring_box:
        pixels = rgba.load()
        ring_pixels = ring.load()
        for y in range(ring_box[1], ring_box[3]):
            for x in range(ring_box[0], ring_box[2]):
                if ring_pixels[x, y] == 0:
                    continue
                r, g, b, _ = pixels[x, y]
                low = min(r, g, b)
                a = 255 - low
                if a <= 0:
                    pixels[x, y] = (0, 0, 0, 0)
                    continue
                denom = 255 - low
                pixels[x, y] = (
                    (r - low) * 255 // denom,
                    (g - low) * 255 // denom,
                    (b - low) * 255 // denom,
                    a,
                )
    return rgba


def dark_variant(rgba):
    """透明底暗色变体：身体→深灰、墨色→近白、粉耳略提亮（alpha 不变）。"""
    out = rgba.copy()
    pixels = out.load()
    width, height = out.size
    for y in range(height):
        for x in range(width):
            r, g, b, a = pixels[x, y]
            if a == 0:
                continue
            if r - g > PINK_MIN_DIFF and r > PINK_MIN_RED:
                pixels[x, y] = (min(255, r + 16), min(255, g + 24), min(255, b + 20), a)
                continue
            luma = (299 * r + 587 * g + 114 * b) // 1000
            if luma < INK_LUMA:
                pixels[x, y] = (*DARK_INK, a)
            else:
                pixels[x, y] = (*DARK_BODY, a)
    return out


def png_base64(image):
    buffer = io.BytesIO()
    image.save(buffer, "PNG", optimize=True)
    return base64.b64encode(buffer.getvalue()).decode("ascii")


def svg_with_variants(light, dark, size):
    """内嵌亮/暗两张透明 PNG，按 prefers-color-scheme 切换显示。"""
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}" role="img" aria-label="\u7ad9\u70b9\u56fe\u6807">
  <style>
    .light {{ display: inline; }}
    .dark {{ display: none; }}
    @media (prefers-color-scheme: dark) {{
      .light {{ display: none; }}
      .dark {{ display: inline; }}
    }}
  </style>
  <image class="light" width="{size}" height="{size}" href="data:image/png;base64,{png_base64(light)}"/>
  <image class="dark" width="{size}" height="{size}" href="data:image/png;base64,{png_base64(dark)}"/>
</svg>
"""


def main():
    parser = argparse.ArgumentParser(description="生成 nobelog 站点图标")
    parser.add_argument(
        "source",
        nargs="?",
        default=str(DEFAULT_SOURCE),
        help=f"源图路径（默认 {DEFAULT_SOURCE.relative_to(ROOT)}）",
    )
    parser.add_argument("--out", default=str(OUT), help=f"输出目录（默认 {OUT.relative_to(ROOT)}）")
    parser.add_argument(
        "--preview-dir",
        default=None,
        help="预览目录（默认 .run/review/<源图名去掉 -source>）",
    )
    args = parser.parse_args()

    source_path = Path(args.source)
    if not source_path.is_absolute():
        source_path = (ROOT / source_path).resolve()
    if not source_path.is_file():
        raise SystemExit(f"缺少源图：{source_path}")
    out_dir = Path(args.out)
    if not out_dir.is_absolute():
        out_dir = (ROOT / out_dir).resolve()
    if args.preview_dir:
        preview_dir = Path(args.preview_dir)
        if not preview_dir.is_absolute():
            preview_dir = ROOT / preview_dir
    else:
        preview_dir = ROOT / ".run/review" / source_path.stem.replace("-source", "")
    os.makedirs(out_dir, exist_ok=True)
    os.makedirs(preview_dir, exist_ok=True)

    source = Image.open(source_path).convert("RGB")
    bbox = ink_bbox(source)
    square = padded_square(source, bbox)
    print(
        f"源图 {display(source_path)} {source.size[0]}x{source.size[1]}"
        f" sha256={sha256(source_path)}"
    )
    print(
        f"内容 bbox={bbox} 尺寸={bbox[2] - bbox[0]}x{bbox[3] - bbox[1]}"
        f" → 方形画布 {square.size[0]}x{square.size[1]}（每边留白约 {PADDING_RATIO * 100:.0f}%）"
    )

    produced = []
    for size in OPAQUE:
        path = out_dir / f"{PREFIX}-{size}.png"
        save_png(square.resize((size, size), Image.LANCZOS), path)
        produced.append(path)
    for size in LIGHT_TRANSPARENT:
        path = out_dir / f"{PREFIX}-{size}-transparent.png"
        save_png(background_to_alpha(square.resize((size, size), Image.LANCZOS)), path)
        produced.append(path)
    for size in DARK_TRANSPARENT:
        light = background_to_alpha(square.resize((size, size), Image.LANCZOS))
        path = out_dir / f"{PREFIX}-{size}-dark.png"
        save_png(dark_variant(light), path)
        produced.append(path)
    for size in BACKUP_TRANSPARENT:
        path = out_dir / f"{PREFIX}-transparent-{size}.png"
        save_png(background_to_alpha(square.resize((size, size), Image.LANCZOS)), path)
        produced.append(path)

    svg_light = background_to_alpha(square.resize((SVG_SIZE, SVG_SIZE), Image.LANCZOS))
    svg_path = out_dir / f"{PREFIX}.svg"
    svg_path.write_text(
        svg_with_variants(svg_light, dark_variant(svg_light), SVG_SIZE), encoding="utf-8"
    )
    os.chmod(svg_path, 0o644)
    produced.append(svg_path)

    # 预览目录引用同目录图标：先清旧副本，再放全量（含预览专用的亮/暗小图）。
    for stale in preview_dir.glob(f"{PREFIX}-*"):
        stale.unlink()
    preview_files = []
    for path in produced:
        target = preview_dir / path.name
        shutil.copy2(path, target)
        preview_files.append(target)
    for size in PREVIEW_LIGHT:
        light = background_to_alpha(square.resize((size, size), Image.LANCZOS))
        path = preview_dir / f"{PREFIX}-{size}-transparent.png"
        save_png(light, path)
        preview_files.append(path)
    for size in PREVIEW_DARK:
        light = background_to_alpha(square.resize((size, size), Image.LANCZOS))
        path = preview_dir / f"{PREFIX}-{size}-dark.png"
        save_png(dark_variant(light), path)
        preview_files.append(path)

    print(f"\n入库产出（{display(out_dir)}）：")
    for path in produced:
        if path.suffix == ".svg":
            print(
                f"  {display(path)}  {path.stat().st_size} B"
                f"  {oct(path.stat().st_mode & 0o777)}  sha256={sha256(path)[:12]}"
            )
            continue
        with Image.open(path) as im:
            print(
                f"  {display(path)}  {im.size[0]}x{im.size[1]} {im.mode}"
                f"  {path.stat().st_size} B  {oct(path.stat().st_mode & 0o777)}"
                f"  sha256={sha256(path)[:12]}"
            )
    print(f"\n预览产出（{display(preview_dir)}，不入库）：")
    for path in preview_files:
        if path.suffix == ".svg":
            print(f"  {display(path)}  {path.stat().st_size} B  sha256={sha256(path)[:12]}")
            continue
        with Image.open(path) as im:
            print(
                f"  {display(path)}  {im.size[0]}x{im.size[1]} {im.mode}"
                f"  {path.stat().st_size} B  sha256={sha256(path)[:12]}"
            )


if __name__ == "__main__":
    main()
