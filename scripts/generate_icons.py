"""Generate BuffBrain icons from buffbrain-logo.png source.

Takes the user-provided logo (with curved corners), adds alpha transparency,
and generates all required icon formats: PNGs (all sizes), ICNS (macOS), ICO (Windows), tray icons.
"""
import os
import struct
import zlib
from PIL import Image, ImageDraw, ImageFont, ImageFilter

OUT = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
SRC = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons", "buffbrain-logo.png")
os.makedirs(OUT, exist_ok=True)

WHITE = (255, 255, 255)


def add_rounded_rect_mask(img, corner_ratio=0.18):
    """Add alpha channel with rounded rectangle transparency."""
    if img.mode == "RGB":
        img = img.convert("RGBA")
    w, h = img.size
    corner = int(min(w, h) * corner_ratio)
    mask = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(mask)
    draw.rounded_rectangle((0, 0, w - 1, h - 1), radius=corner, fill=255)
    img.putalpha(mask)  # this adds/replaces alpha
    return img


def make_tray_icon(size, output_size=None):
    """Generate a monochrome tray icon (white 'b' on transparent)."""
    if output_size is None:
        output_size = size
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    font_size = int(size * 0.72)
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Palatino.ttc", font_size)
    except Exception:
        font = ImageFont.load_default()
    bbox = font.getbbox("b")
    char_w = bbox[2] - bbox[0]
    char_h = bbox[3] - bbox[1]
    cx = (size - char_w) // 2 - bbox[0]
    cy = (size - char_h) // 2 - bbox[1]
    draw.text((cx, cy), "b", fill=WHITE, font=font)
    if output_size != size:
        img = img.resize((output_size, output_size), Image.LANCZOS)
    return img


def save_icns(png_path, icns_path):
    """Convert a 1024x1024 PNG to ICNS using macOS iconutil."""
    iconset_dir = png_path.replace(".png", ".iconset")
    os.makedirs(iconset_dir, exist_ok=True)
    img = Image.open(png_path)
    sizes = {
        "icon_16x16": 16, "icon_16x16@2x": 32,
        "icon_32x32": 32, "icon_32x32@2x": 64,
        "icon_128x128": 128, "icon_128x128@2x": 256,
        "icon_256x256": 256, "icon_256x256@2x": 512,
        "icon_512x512": 512, "icon_512x512@2x": 1024,
    }
    for name, s in sizes.items():
        resized = img.resize((s, s), Image.LANCZOS)
        # Ensure RGBA for ICNS
        if resized.mode != "RGBA":
            resized = resized.convert("RGBA")
        resized.save(os.path.join(iconset_dir, f"{name}.png"))
    os.system(f"iconutil -c icns {iconset_dir} -o {icns_path}")
    for f in os.listdir(iconset_dir):
        os.remove(os.path.join(iconset_dir, f))
    os.rmdir(iconset_dir)


def save_ico(png_path, ico_path):
    """Convert PNG to ICO (Windows icon)."""
    img = Image.open(png_path)
    img_32 = img.resize((32, 32), Image.LANCZOS)
    img_16 = img.resize((16, 16), Image.LANCZOS)
    img_32.save(ico_path, format="ICO", sizes=[(32, 32), (16, 16)])


def main():
    print(f"Loading source logo: {SRC}")
    logo = Image.open(SRC)
    print(f"  Original: {logo.size} mode={logo.mode}")

    # Add rounded-rect alpha mask for transparency
    logo = add_rounded_rect_mask(logo)
    logo.save(os.path.join(OUT, "icon.png"))
    print(f"  Saved: icon.png ({logo.size[0]}x{logo.size[1]})")

    # Generate all required PNG sizes
    sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }
    for name, s in sizes.items():
        resized = logo.resize((s, s), Image.LANCZOS)
        if resized.mode != "RGBA":
            resized = resized.convert("RGBA")
        resized.save(os.path.join(OUT, name))
        print(f"  {name} ({s}x{s})")

    print("Generating ICNS...")
    icns_path = os.path.join(OUT, "icon.icns")
    save_icns(os.path.join(OUT, "icon.png"), icns_path)
    print(f"  icon.icns")

    print("Generating ICO...")
    ico_path = os.path.join(OUT, "icon.ico")
    save_ico(os.path.join(OUT, "icon.png"), ico_path)
    print(f"  icon.ico")

    # Tray icons - monochrome 'b'
    print("Generating tray icons...")
    tray_color = logo.resize((32, 32), Image.LANCZOS)
    tray_color.save(os.path.join(OUT, "tray-color.png"))
    print(f"  tray-color.png (32x32)")

    tray = make_tray_icon(64, 32)
    tray.save(os.path.join(OUT, "tray.png"))
    print(f"  tray.png (32x32)")

    print("\nAll icons generated successfully!")


if __name__ == "__main__":
    main()
