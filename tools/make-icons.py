#!/usr/bin/env python3
"""生成 nobelog 站点图标（可复跑，源图可选）。

用法（仓库根或任意目录）：
    python3 tools/make-icons.py                                    # 默认熊脸 tools/assets/bear-icon-source.jpg
    python3 tools/make-icons.py tools/assets/cat-icon-source.jpg   # 换回猫咪（一键切换）

输出前缀固定 `site-icon`，写进 content/images/（权限 644）：
    site-icon-32.png               白底彩色（favicon 兜底）
    site-icon-192.png              白底彩色（favicon 高分屏兜底）
    site-icon-180.png              白底彩色（apple-touch-icon）
    site-icon-512.png              白底彩色（og:image）
    site-icon-transparent-512.png  背景转透明的备用版（保留插画原色）

流程：
    1. 用亮度阈值找「暗像素」（描边/五官）的 bbox；
    2. 裁到 bbox，按长边加约 5% 内边距，居中放进 1:1 方形画布（白底）；
    3. 按目标尺寸 LANCZOS 缩放。

透明版（仅 512）：把「与画布边缘相连的近白背景」用 flood fill 转成 alpha=0，
被描边包围的内部浅色（熊脸奶白、粉耳朵）保持不透明；背景与内容之间 3px 环带
按「原图 = 前景 * a + 白 * (1-a)」反预乘，避免暗底上出现白边。
注意：不能直接用 alpha = 255 - 亮度——这张图的脸是奶白（min 通道约 228），
那样会把整张脸洗成半透明。

预览专用（默认 .run/review/<源图名去掉 -source>/，不入库）：16 / 48 白底小图 + 图标副本。

本版不生成 SVG（暗色标签页自适应在猫咪那版试过，这版按需求只出 PNG）；
换图只需替换 tools/assets 下的源图后重跑本脚本，或把别的源图路径作为参数传入。
"""

import argparse
import hashlib
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
# 白底不透明尺寸。
OPAQUE = [32, 180, 192, 512]
# 透明版尺寸。
TRANSPARENT = [512]
# 仅预览用的白底小图尺寸。
PREVIEW_ONLY = [16, 48]

# flood fill 容差（PIL 按通道差绝对值之和比较）：只吃纯白/近白背景，
# 奶白脸与白底的差约 37，留在容差外，且脸被描边包围，本来就填不进去。
FLOOD_THRESHOLD = 30
# 背景与内容之间的过渡环带宽度（像素）。
EDGE_RING = 3


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
    preview_dir = (
        Path(args.preview_dir)
        if args.preview_dir
        else ROOT / ".run/review" / source_path.stem.replace("-source", "")
    )
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
    for size in TRANSPARENT:
        path = out_dir / f"{PREFIX}-transparent-{size}.png"
        save_png(background_to_alpha(square.resize((size, size), Image.LANCZOS)), path)
        produced.append(path)

    # 预览目录引用同目录图标：先清旧副本，再放全量（含预览专用尺寸）。
    for stale in preview_dir.glob(f"{PREFIX}-*"):
        stale.unlink()
    preview_files = []
    for path in produced:
        target = preview_dir / path.name
        shutil.copy2(path, target)
        preview_files.append(target)
    for size in PREVIEW_ONLY:
        path = preview_dir / f"{PREFIX}-{size}.png"
        save_png(square.resize((size, size), Image.LANCZOS), path)
        preview_files.append(path)

    print(f"\n入库产出（{display(out_dir)}）：")
    for path in produced:
        with Image.open(path) as im:
            print(
                f"  {display(path)}  {im.size[0]}x{im.size[1]} {im.mode}"
                f"  {path.stat().st_size} B  {oct(path.stat().st_mode & 0o777)}"
                f"  sha256={sha256(path)[:12]}"
            )
    print(f"\n预览产出（{display(preview_dir)}，不入库）：")
    for path in preview_files:
        with Image.open(path) as im:
            print(
                f"  {display(path)}  {im.size[0]}x{im.size[1]} {im.mode}"
                f"  {path.stat().st_size} B  sha256={sha256(path)[:12]}"
            )


if __name__ == "__main__":
    main()
