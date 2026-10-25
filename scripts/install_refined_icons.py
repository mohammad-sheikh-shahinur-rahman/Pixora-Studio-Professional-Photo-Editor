"""Install the user's refined Pixora Studio logo icon to all asset directories."""

import os
from PIL import Image

def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    src_1024 = os.path.join(root, "branding", "refined", "pixora-studio-refined-1024.png")
    src_ico = os.path.join(root, "branding", "refined", "pixora-studio-refined.ico")
    
    app_icon_dir = os.path.join(root, "assets", "app-icon")
    hicolor_dir = os.path.join(app_icon_dir, "hicolor")

    print(f"Loading refined master icon: {src_1024}")
    master = Image.open(src_1024).convert("RGBA")
    
    # Save master 1024
    master.save(os.path.join(app_icon_dir, "pixora-1024.png"), "PNG")
    master.save(os.path.join(app_icon_dir, "photocraft-1024.png"), "PNG")
    print("Saved 1024x1024 masters.")

    # Resize to all standard sizes with Lanczos resampling
    sizes = [16, 24, 32, 48, 64, 128, 256, 512]
    for s in sizes:
        resized = master.resize((s, s), Image.Resampling.LANCZOS)
        # Clear filter ringing at transparent corners and clamp solid interior to 255
        data = list(resized.getdata())
        cleaned_data = [(r, g, b, 0 if a < 8 else (255 if a > 240 else a)) for (r, g, b, a) in data]
        resized.putdata(cleaned_data)
        size_dir = os.path.join(hicolor_dir, f"{s}x{s}", "apps")
        os.makedirs(size_dir, exist_ok=True)
        
        # Save both identifiers
        resized.save(os.path.join(size_dir, "com.amadersomaj.pixorastudio.png"), "PNG")
        resized.save(os.path.join(size_dir, "ai.storyteller.photocraft.png"), "PNG")
        print(f"Saved {s}x{s} icons in {size_dir}")

    # Copy the refined .ico directly
    with open(src_ico, "rb") as f:
        ico_bytes = f.read()

    with open(os.path.join(app_icon_dir, "pixora.ico"), "wb") as f:
        f.write(ico_bytes)
    with open(os.path.join(app_icon_dir, "photocraft.ico"), "wb") as f:
        f.write(ico_bytes)
    print(f"Copied refined Windows ICO ({len(ico_bytes)} bytes) to pixora.ico and photocraft.ico.")

if __name__ == "__main__":
    main()
