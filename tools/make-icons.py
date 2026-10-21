#!/usr/bin/env python3
"""make-icons.py — buat ikon peluncur Android dari ikon PWA repo web.

Sumber: ../korean-grammar/icons/icon-512.png (latar #e14b6a + huruf 한 putih).

Keluarannya (di android/app/src/main/res/):
  mipmap-<dpi>/ic_launcher.png          ikon biasa (persegi membulat)
  mipmap-<dpi>/ic_launcher_round.png    ikon bundar
  mipmap-<dpi>/ic_launcher_foreground.png  lapisan depan ikon adaptif (huruf만)
  drawable/ic_launcher_background.xml   latar adaptif (warna aksen)

Jalankan: python tools/make-icons.py
"""
from __future__ import annotations

import pathlib
import sys

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
SRC = ROOT.parent / "korean-grammar" / "icons" / "icon-512.png"
RES = ROOT / "android" / "app" / "src" / "main" / "res"

# dpi -> ukuran ikon peluncur (48dp)
DENSITIES = {
    "mdpi": 48,
    "hdpi": 72,
    "xhdpi": 96,
    "xxhdpi": 144,
    "xxxhdpi": 192,
}

BG = (225, 75, 106, 255)  # #e14b6a, aksen css/style.css


def rounded_mask(size: int, radius_ratio: float = 0.22) -> Image.Image:
    """Mask persegi membulat (sudut ~22%), mirip ikon PWA."""
    mask = Image.new("L", (size * 4, size * 4), 0)
    r = int(size * 4 * radius_ratio)
    from PIL import ImageDraw

    ImageDraw.Draw(mask).rounded_rectangle([0, 0, size * 4 - 1, size * 4 - 1], radius=r, fill=255)
    return mask.resize((size, size), Image.LANCZOS)


def circle_mask(size: int) -> Image.Image:
    mask = Image.new("L", (size * 4, size * 4), 0)
    from PIL import ImageDraw

    ImageDraw.Draw(mask).ellipse([0, 0, size * 4 - 1, size * 4 - 1], fill=255)
    return mask.resize((size, size), Image.LANCZOS)


def glyph_layer(size: int, src: Image.Image) -> Image.Image:
    """Lapisan depan ikon adaptif: huruf 한 putih transparan, aman di dalam
    area aman 66% (ikon adaptif bisa dipotong sampai lingkaran 72dp dari 108dp)."""
    scale = int(size * 0.60)
    glyph = src.resize((scale, scale), Image.LANCZOS)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    off = (size - scale) // 2
    canvas.alpha_composite(glyph, (off, off))
    return canvas


def main() -> int:
    if not SRC.exists():
        print(f"ikon sumber tidak ditemukan: {SRC}", file=sys.stderr)
        return 1

    src = Image.open(SRC).convert("RGBA")

    for dpi, px in DENSITIES.items():
        out = RES / f"mipmap-{dpi}"
        out.mkdir(parents=True, exist_ok=True)

        icon = src.resize((px, px), Image.LANCZOS)
        rounded = Image.new("RGBA", (px, px), (0, 0, 0, 0))
        rounded.paste(icon, (0, 0), rounded_mask(px))
        rounded.save(out / "ic_launcher.png")

        round_icon = Image.new("RGBA", (px, px), (0, 0, 0, 0))
        round_icon.paste(icon, (0, 0), circle_mask(px))
        round_icon.save(out / "ic_launcher_round.png")

        # Lapisan depan adaptif: 108dp dengan aman 66% -> ukuran kanvas 2x ikon
        adpx = int(px * 1.5)
        glyph_layer(adpx, src).save(out / "ic_launcher_foreground.png")

        print(f"mipmap-{dpi}: {px}px")

    drawable = RES / "drawable"
    drawable.mkdir(parents=True, exist_ok=True)
    (drawable / "ic_launcher_background.xml").write_text(
        '<?xml version="1.0" encoding="utf-8"?>\n'
        '<shape xmlns:android="http://schemas.android.com/apk/res/android"\n'
        '    android:shape="rectangle">\n'
        '    <solid android:color="#E14B6A" />\n'
        "</shape>\n",
        encoding="utf-8",
    )
    print("drawable/ic_launcher_background.xml")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
