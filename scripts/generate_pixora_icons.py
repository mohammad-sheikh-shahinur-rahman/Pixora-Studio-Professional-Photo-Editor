"""Generate high-resolution Pixora Studio app icons and master assets."""

import os
import struct
import math
from PIL import Image, ImageDraw

def draw_pixora_icon(size):
    scale = 2
    canvas_size = size * scale
    img = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))

    pad = canvas_size * 0.05
    box = [pad, pad, canvas_size - pad, canvas_size - pad]
    radius = canvas_size * 0.22

    # Draw rounded rectangle background (Dark Slate with subtle gradient)
    bg = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    bg_draw = ImageDraw.Draw(bg)
    bg_draw.rounded_rectangle(box, radius=radius, fill=(20, 24, 32, 255))
    
    # Inner subtle rim / border
    rim_color = (60, 75, 102, 180)
    bg_draw.rounded_rectangle(box, radius=radius, outline=rim_color, width=max(2, int(scale * 1.5)))

    img = Image.alpha_composite(img, bg)

    # Center coordinates and radius for the photographic aperture & prism 'P'
    cx = canvas_size * 0.5
    cy = canvas_size * 0.5
    outer_r = canvas_size * 0.33
    inner_r = canvas_size * 0.12

    # Spectral aperture blade colors (Cyan -> Electric Blue -> Violet -> Magenta -> Amber)
    blade_colors = [
        ((0, 229, 255, 240), (0, 176, 255, 240)),    # Vibrant Cyan
        ((41, 121, 255, 240), (61, 90, 254, 240)),   # Electric Blue
        ((101, 31, 255, 240), (124, 77, 255, 240)),  # Deep Purple / Indigo
        ((224, 64, 251, 240), (255, 64, 129, 240)),  # Magenta to Pink
        ((255, 82, 82, 240), (255, 145, 0, 240)),    # Coral to Amber
        ((255, 214, 0, 240), (0, 230, 118, 240)),    # Bright Gold
    ]

    num_blades = len(blade_colors)
    angle_step = 2 * math.pi / num_blades

    # Draw camera aperture iris blades
    for i in range(num_blades):
        a1 = i * angle_step - 0.2
        a2 = a1 + angle_step + 0.35
        
        p1 = (cx + outer_r * math.cos(a1), cy + outer_r * math.sin(a1))
        p2 = (cx + outer_r * 1.05 * math.cos(a1 + 0.3), cy + outer_r * 1.05 * math.sin(a1 + 0.3))
        p3 = (cx + inner_r * 1.8 * math.cos(a2), cy + inner_r * 1.8 * math.sin(a2))
        p4 = (cx + inner_r * math.cos(a2 - 0.2), cy + inner_r * math.sin(a2 - 0.2))

        c1, _ = blade_colors[i]
        blade_img = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
        blade_draw = ImageDraw.Draw(blade_img)
        blade_draw.polygon([p1, p2, p3, p4], fill=c1, outline=(255, 255, 255, 120), width=max(1, int(scale * 1)))
        img = Image.alpha_composite(img, blade_img)

    draw = ImageDraw.Draw(img)

    # Central lens element (Deep dark core with cyan-tinted inner glow)
    lens_core = [cx - inner_r * 1.1, cy - inner_r * 1.1, cx + inner_r * 1.1, cy + inner_r * 1.1]
    draw.ellipse(lens_core, fill=(11, 14, 20, 255), outline=(0, 229, 255, 220), width=max(2, int(scale * 2)))

    # Lens focal highlight / reflection (photographic flare arc)
    flare_box = [cx - inner_r * 0.7, cy - inner_r * 0.7, cx + inner_r * 0.7, cy + inner_r * 0.7]
    draw.arc(flare_box, start=200, end=320, fill=(255, 255, 255, 220), width=max(2, int(scale * 2)))

    # Central sparkle / lens focal point
    focal_pt = (cx + inner_r * 0.25, cy - inner_r * 0.25)
    f_rad = inner_r * 0.2
    draw.ellipse([focal_pt[0] - f_rad, focal_pt[1] - f_rad, focal_pt[0] + f_rad, focal_pt[1] + f_rad], fill=(255, 255, 255, 240))

    # Corner photographic frame / focal brackets (studio crop marks)
    bracket_len = canvas_size * 0.08
    bracket_color = (0, 229, 255, 160)
    bw = max(2, int(scale * 2))
    
    bx1, by1 = pad * 2.2, pad * 2.2
    draw.line([(bx1, by1), (bx1 + bracket_len, by1)], fill=bracket_color, width=bw)
    draw.line([(bx1, by1), (bx1, by1 + bracket_len)], fill=bracket_color, width=bw)

    bx2, by2 = canvas_size - pad * 2.2, canvas_size - pad * 2.2
    draw.line([(bx2, by2), (bx2 - bracket_len, by2)], fill=bracket_color, width=bw)
    draw.line([(bx2, by2), (bx2, by2 - bracket_len)], fill=bracket_color, width=bw)

    final_img = img.resize((size, size), Image.Resampling.LANCZOS)
    return final_img

def pack_ico(png_bytes_list):
    """Pack a list of raw PNG byte strings into a Windows .ico following xtask/ico.rs."""
    count = len(png_bytes_list)
    header = struct.pack('<HHH', 0, 1, count)
    offset = 6 + 16 * count
    dir_entries = bytearray()
    
    for png in png_bytes_list:
        w = struct.unpack('>I', png[16:20])[0]
        h = struct.unpack('>I', png[20:24])[0]
        dim = 0 if w == 256 else w
        dir_entries.extend(struct.pack('<BBBBHHII', dim, dim, 0, 0, 1, 32, len(png), offset))
        offset += len(png)
        
    out = bytearray(header) + dir_entries
    for png in png_bytes_list:
        out.extend(png)
    return bytes(out)

def main():
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    app_icon_dir = os.path.join(base_dir, "assets", "app-icon")
    hicolor_dir = os.path.join(app_icon_dir, "hicolor")

    sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
    rendered = {}
    png_bytes_by_size = {}

    import io
    print("Rendering Pixora Studio icons...")
    for s in sizes:
        img = draw_pixora_icon(s)
        rendered[s] = img
        
        buf = io.BytesIO()
        img.save(buf, "PNG")
        png_bytes = buf.getvalue()
        png_bytes_by_size[s] = png_bytes
        
        size_str = f"{s}x{s}"
        size_dir = os.path.join(hicolor_dir, size_str, "apps")
        if os.path.exists(size_dir):
            new_path = os.path.join(size_dir, "com.amadersomaj.pixorastudio.png")
            with open(new_path, "wb") as f:
                f.write(png_bytes)
            legacy_path = os.path.join(size_dir, "ai.storyteller.photocraft.png")
            with open(legacy_path, "wb") as f:
                f.write(png_bytes)
            print(f"  Saved {new_path}")

    # Master 1024
    p1024 = os.path.join(app_icon_dir, "pixora-1024.png")
    with open(p1024, "wb") as f:
        f.write(png_bytes_by_size[1024])
    with open(os.path.join(app_icon_dir, "photocraft-1024.png"), "wb") as f:
        f.write(png_bytes_by_size[1024])
    print(f"Saved master 1024: {p1024}")

    # Pack multi-size ICO: 16, 24, 32, 48, 64, 128, 256
    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    ico_pngs = [png_bytes_by_size[s] for s in ico_sizes]
    ico_data = pack_ico(ico_pngs)
    
    pixora_ico = os.path.join(app_icon_dir, "pixora.ico")
    photocraft_ico = os.path.join(app_icon_dir, "photocraft.ico")
    
    with open(pixora_ico, "wb") as f:
        f.write(ico_data)
    with open(photocraft_ico, "wb") as f:
        f.write(ico_data)
    print(f"Saved Windows ICO: {pixora_ico} ({len(ico_sizes)} resolutions, {len(ico_data)} bytes)")

if __name__ == "__main__":
    main()
