#!/usr/bin/env python3
"""从 tools/assets/site-icon-source.png 生成 nobelog 站点图标（可复跑）。

用法（仓库根或任意目录）：
    python3 tools/make-icons.py

产出（content/images，权限 644）：
    site-icon-32-transparent.png   favicon 32（透明底）
    site-icon-192-transparent.png  favicon 192（透明底）
    site-icon-512-transparent.png  分享/备用（透明底）
    site-icon-180.png              apple-touch-icon（不透明米白底）
    site-icon-512.png              og:image（不透明米白底）

规则：
- 不透明版：整图 LANCZOS 缩放，不裁切（环形完整、留白比例与原图一致），RGB PNG。
- 透明版：先缩放到目标尺寸，再做「近白底转 alpha」：
      alpha = 255 - min(r,g,b)
      F = (C - min) * 255 / (255 - min)   # 按「原图 = 前景 * a + 白 * (1-a)」反预乘
  先缩放后转 alpha，避免全透明像素（颜色为黑）在 LANCZOS 里污染边缘。
  源图底色不是纯白（角点约 251-254），所以背景会保留 alpha 1-4（约 1% 不透明，暗底不可见）。
- 16/48 只生成到 .run/review/icon/（预览用，不入库）。

改图标只需替换 tools/assets/site-icon-source.png 后重跑本脚本；
新增尺寸时同步改下面的常量，并确认模板/测试引用一致。
"""

import hashlib
import os
import shutil
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "tools/assets/site-icon-source.png"
OUT = ROOT / "content/images"
PREVIEW = ROOT / ".run/review/icon"

OPAQUE = [180, 512]
TRANSPARENT = [32, 192, 512]
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


def white_to_alpha(image):
    """近白底转透明：alpha = 255 - min(r,g,b)，再反预乘保住前景色。"""
    out = []
    for (r, g, b) in image.getdata():
        mn = min(r, g, b)
        alpha = 255 - mn
        if alpha == 0:
            out.append((0, 0, 0, 0))
            continue
        denom = 255 - mn
        out.append(
            (
                (r - mn) * 255 // denom,
                (g - mn) * 255 // denom,
                (b - mn) * 255 // denom,
                alpha,
            )
        )
    rgba = Image.new("RGBA", image.size)
    rgba.putdata(out)
    return rgba


def main():
    if not SRC.is_file():
        raise SystemExit(f"缺少源图：{SRC}")
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(PREVIEW, exist_ok=True)

    src = Image.open(SRC).convert("RGB")
    print(f"源图 {SRC.relative_to(ROOT)} {src.size[0]}x{src.size[1]} sha256={sha256(SRC)}")
    print("角点:", [src.getpixel(p) for p in [(0, 0), (src.width - 1, 0), (0, src.height - 1)]])

    produced = []  # 入库文件
    for size in OPAQUE:
        path = OUT / f"site-icon-{size}.png"
        save_png(src.resize((size, size), Image.LANCZOS), path)
        produced.append(path)
    for size in TRANSPARENT:
        base = src.resize((size, size), Image.LANCZOS)
        path = OUT / f"site-icon-{size}-transparent.png"
        save_png(white_to_alpha(base), path)
        produced.append(path)

    # 预览目录引用同目录图标：先清掉旧副本，再放全量（含预览专用尺寸）。
    for stale in PREVIEW.glob("site-icon-*.png"):
        stale.unlink()
    preview_files = []
    for path in produced:
        target = PREVIEW / path.name
        shutil.copy2(path, target)
        preview_files.append(target)
    for size in PREVIEW_TRANSPARENT:
        base = src.resize((size, size), Image.LANCZOS)
        path = PREVIEW / f"site-icon-{size}-transparent.png"
        save_png(white_to_alpha(base), path)
        preview_files.append(path)

    print("\n入库产出（content/images）：")
    for path in produced:
        with Image.open(path) as im:
            print(
                f"  {path.relative_to(ROOT)}  {im.size[0]}x{im.size[1]} {im.mode}"
                f"  {path.stat().st_size} B  {oct(path.stat().st_mode & 0o777)}"
                f"  sha256={sha256(path)[:12]}"
            )
    print("\n预览产出（.run/review/icon，不入库）：")
    for path in preview_files:
        with Image.open(path) as im:
            print(
                f"  {path.relative_to(ROOT)}  {im.size[0]}x{im.size[1]} {im.mode}"
                f"  {path.stat().st_size} B  sha256={sha256(path)[:12]}"
            )


if __name__ == "__main__":
    main()
