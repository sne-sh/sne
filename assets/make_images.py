"""Regenerates the PNG, ICO and installer images from the same shapes as logo.svg.

Run from the repository root:  python assets/make_images.py   (needs Pillow)
"""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
INSTALLER = ROOT / "installer"

BACKGROUND = "#0B1220"
HEXAGON = "#22D3EE"
BOLT = "#FACC15"

# Coordinates on a 1024 x 1024 canvas, identical to logo.svg.
HEX_OUTER = [(512, 172), (806.4, 342), (806.4, 682), (512, 852), (217.6, 682), (217.6, 342)]
HEX_INNER = [(512, 236), (751, 374), (751, 650), (512, 788), (273, 650), (273, 374)]
BOLT_SHAPE = [(570, 290), (395, 545), (505, 545), (455, 735), (635, 480), (522, 480)]


def logo(size: int, rounded: bool = True) -> Image.Image:
    """Draws the logo at 4x and scales down for smooth edges."""
    scale = 4 * size / 1024
    canvas = Image.new("RGBA", (4 * size, 4 * size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    points = lambda shape: [(x * scale, y * scale) for x, y in shape]
    if rounded:
        draw.rounded_rectangle([0, 0, 4 * size - 1, 4 * size - 1], radius=224 * scale, fill=BACKGROUND)
    else:
        draw.rectangle([0, 0, 4 * size, 4 * size], fill=BACKGROUND)
    draw.polygon(points(HEX_OUTER), fill=HEXAGON)
    draw.polygon(points(HEX_INNER), fill=BACKGROUND)
    draw.polygon(points(BOLT_SHAPE), fill=BOLT)
    return canvas.resize((size, size), Image.LANCZOS)


def on_background(width: int, height: int, logo_size: int, top: int) -> Image.Image:
    """The logo on a plain dark panel, for the installer wizard (BMP has no transparency)."""
    panel = Image.new("RGB", (width, height), BACKGROUND)
    mark = logo(logo_size, rounded=False).convert("RGB")
    panel.paste(mark, ((width - logo_size) // 2, top))
    return panel


def main() -> None:
    INSTALLER.mkdir(exist_ok=True)
    logo(1024).save(ASSETS / "logo-1024.png")
    logo(512).save(ASSETS / "logo-512.png")
    # Square, no transparent corners: GitHub crops organisation avatars itself.
    logo(500, rounded=False).convert("RGB").save(ASSETS / "avatar-500.png")
    logo(256).save(ASSETS / "sne.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
    # Inno Setup wizard artwork at 200% scale; it shrinks them on normal displays.
    on_background(328, 628, 240, 120).save(INSTALLER / "wizard-large.bmp")
    on_background(110, 116, 104, 6).save(INSTALLER / "wizard-small.bmp")


if __name__ == "__main__":
    main()
