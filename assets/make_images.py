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
BOLT = "#FACC15"  # the link

# Coordinates on a 1024 x 1024 canvas, identical to logo.svg: a link (yellow) arriving at a version (hexagon).
HEX_CENTER, HEX_RADIUS, HEX_STROKE = (600, 512), 270, 60
LINK_START, LINK_END = (150, 512), (600, 512)
LINK_STROKE, LINK_GAP, LINK_DOT = 64, 150, 84


def hexagon(center: tuple[float, float], radius: float) -> list[tuple[float, float]]:
    cx, cy = center
    half = 0.8660254 * radius
    return [
        (cx, cy - radius),
        (cx + half, cy - radius / 2),
        (cx + half, cy + radius / 2),
        (cx, cy + radius),
        (cx - half, cy + radius / 2),
        (cx - half, cy - radius / 2),
    ]


def logo(size: int, rounded: bool = True) -> Image.Image:
    """Draws the logo at 4x and scales down for smooth edges."""
    scale = 4 * size / 1024
    canvas = Image.new("RGBA", (4 * size, 4 * size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)

    def dot(point: tuple[float, float], radius: float, fill: str) -> None:
        x, y = point[0] * scale, point[1] * scale
        r = radius * scale
        draw.ellipse([x - r, y - r, x + r, y + r], fill=fill)

    def stroke(points: list[tuple[float, float]], width: float, fill: str) -> None:
        """A line with round ends and round corners, like SVG stroke-linecap/linejoin round."""
        draw.line([(x * scale, y * scale) for x, y in points], fill=fill, width=round(width * scale))
        for point in points:
            dot(point, width / 2, fill)

    if rounded:
        draw.rounded_rectangle([0, 0, 4 * size - 1, 4 * size - 1], radius=224 * scale, fill=BACKGROUND)
    else:
        draw.rectangle([0, 0, 4 * size, 4 * size], fill=BACKGROUND)
    outline = hexagon(HEX_CENTER, HEX_RADIUS)
    stroke(outline + [outline[0]], HEX_STROKE, HEXAGON)
    # A dark band under the link cuts the hexagon where the link passes through it.
    stroke([LINK_START, LINK_END], LINK_GAP, BACKGROUND)
    stroke([LINK_START, LINK_END], LINK_STROKE, BOLT)
    dot(LINK_END, LINK_DOT, BOLT)
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
    # Built into the program and shown in desktop notifications.
    logo(256).save(ASSETS / "logo-256.png", optimize=True)
    # Shown on the first page of the GitBook documentation.
    (ROOT / "docs" / ".gitbook" / "assets").mkdir(parents=True, exist_ok=True)
    logo(512).save(ROOT / "docs" / ".gitbook" / "assets" / "logo.png")
    # Square, no transparent corners: GitHub crops organisation avatars itself.
    logo(500, rounded=False).convert("RGB").save(ASSETS / "avatar-500.png")
    logo(256).save(ASSETS / "sne.ico", sizes=[(s, s) for s in (16, 24, 32, 48, 64, 128, 256)])
    # Inno Setup wizard artwork at 200% scale; it shrinks them on normal displays.
    on_background(328, 628, 240, 120).save(INSTALLER / "wizard-large.bmp")
    on_background(110, 116, 104, 6).save(INSTALLER / "wizard-small.bmp")


if __name__ == "__main__":
    main()
