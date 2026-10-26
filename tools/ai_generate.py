import os, sys, urllib.request, urllib.parse, io
from PIL import Image

def generate_ai_image(prompt, width, height, out_path):
    if not prompt or prompt.strip() == "":
        prompt = "photorealistic studio portrait background soft bokeh lighting"
    
    clean_prompt = prompt.strip()
    encoded = urllib.parse.quote(clean_prompt)
    url = f"https://image.pollinations.ai/prompt/{encoded}?width={width}&height={height}&nologo=true&seed=42"
    req = urllib.request.Request(url, headers={"User-Agent": "PixoraStudio-AI/1.0"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        data = resp.read()
    img = Image.open(io.BytesIO(data)).convert("RGBA")
    if img.size != (width, height):
        img = img.resize((width, height), Image.Resampling.LANCZOS)
    img.save(out_path, format="PNG")
    print(f"OK: {out_path}")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--prompt", type=str, default="")
    parser.add_argument("--width", type=int, default=1024)
    parser.add_argument("--height", type=int, default=1024)
    parser.add_argument("--out", type=str, required=True)
    args = parser.parse_args()
    generate_ai_image(args.prompt, args.width, args.height, args.out)
