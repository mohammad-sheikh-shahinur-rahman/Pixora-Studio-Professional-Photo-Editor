import os
import sys
import shutil
import subprocess
from PIL import Image

ROOT = os.path.abspath(".")
STAGE = os.path.join(ROOT, "dist", "msix-stage")
ASSETS = os.path.join(STAGE, "Assets")
BIN_SRC = os.path.join(ROOT, "target", "x86_64-pc-windows-msvc", "release")
if not os.path.exists(os.path.join(BIN_SRC, "pixora-studio.exe")):
    BIN_SRC = os.path.join(ROOT, "target", "release")
DIST = os.path.join(ROOT, "dist", "release")
OUT_MSIX = os.path.join(DIST, "pixora-studio-0.15.1-windows-x64.msix")
KIT_BIN = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64"
MAKEAPPX = os.path.join(KIT_BIN, "makeappx.exe")
MAKEPRI = os.path.join(KIT_BIN, "makepri.exe")

print("1. Cleaning and preparing staging directory...")
if os.path.exists(STAGE):
    shutil.rmtree(STAGE)
os.makedirs(ASSETS, exist_ok=True)
os.makedirs(DIST, exist_ok=True)

print("2. Copying binaries...")
shutil.copy2(os.path.join(BIN_SRC, "pixora-studio.exe"), os.path.join(STAGE, "pixora-studio.exe"))
shutil.copy2(os.path.join(BIN_SRC, "pixora-studio-cli.exe"), os.path.join(STAGE, "pixora-studio-cli.exe"))

print("3. Generating complete Visual Assets set (Base + Scaled versions)...")
icon = Image.open(os.path.join(ROOT, "assets", "app-icon", "pixora-1024.png"))

def save_scaled(name, base_w, base_h):
    # Base
    img_100 = icon.resize((base_w, base_h), Image.Resampling.LANCZOS)
    img_100.save(os.path.join(ASSETS, f"{name}.png"), "PNG")
    img_100.save(os.path.join(ASSETS, f"{name}.scale-100.png"), "PNG")
    # Scale 200
    img_200 = icon.resize((base_w * 2, base_h * 2), Image.Resampling.LANCZOS)
    img_200.save(os.path.join(ASSETS, f"{name}.scale-200.png"), "PNG")

save_scaled("Square44x44Logo", 44, 44)
save_scaled("Square71x71Logo", 71, 71)
save_scaled("Square150x150Logo", 150, 150)
save_scaled("Square310x310Logo", 310, 310)
save_scaled("StoreLogo", 50, 50)

# Target sizes for Square44x44Logo (taskbar, alt-tab)
for sz in [16, 24, 32, 48, 256]:
    icon.resize((sz, sz), Image.Resampling.LANCZOS).save(
        os.path.join(ASSETS, f"Square44x44Logo.targetsize-{sz}.png"), "PNG"
    )

# Wide 310x150
def make_wide(w, h, icon_sz, filename):
    wide = Image.new("RGBA", (w, h), (18, 18, 24, 255))
    ic = icon.resize((icon_sz, icon_sz), Image.Resampling.LANCZOS)
    x = (w - icon_sz) // 2
    y = (h - icon_sz) // 2
    wide.paste(ic, (x, y), ic if ic.mode == "RGBA" else None)
    wide.save(os.path.join(ASSETS, filename), "PNG")

make_wide(310, 150, 120, "Wide310x150Logo.png")
make_wide(310, 150, 120, "Wide310x150Logo.scale-100.png")
make_wide(620, 300, 240, "Wide310x150Logo.scale-200.png")

print("4. Writing strictly schema-compliant AppxManifest.xml...")
manifest = """<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap rescap">

  <Identity
    Name="ITAmadersomajInc.PixoraStudio"
    Publisher="CN=88BD5184-3CC3-4CE2-BC26-1D0DF54D2A8E"
    Version="0.15.1.0"
    ProcessorArchitecture="x64" />

  <Properties>
    <DisplayName>Pixora Studio</DisplayName>
    <PublisherDisplayName>IT Amadersomaj Inc</PublisherDisplayName>
    <Logo>Assets\\StoreLogo.png</Logo>
  </Properties>

  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>

  <Resources>
    <Resource Language="EN-US" />
  </Resources>

  <Applications>
    <Application Id="PixoraStudio"
      Executable="pixora-studio.exe"
      EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="Pixora Studio"
        Description="Professional Photo Editor"
        BackgroundColor="#121218"
        Square150x150Logo="Assets\\Square150x150Logo.png"
        Square44x44Logo="Assets\\Square44x44Logo.png">
        <uap:DefaultTile
          Square71x71Logo="Assets\\Square71x71Logo.png"
          Wide310x150Logo="Assets\\Wide310x150Logo.png"
          Square310x310Logo="Assets\\Square310x310Logo.png" />
      </uap:VisualElements>
    </Application>
  </Applications>

  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
"""

with open(os.path.join(STAGE, "AppxManifest.xml"), "w", encoding="utf-8") as f:
    f.write(manifest)

print("5. Generating resources.pri with makepri.exe...")
pri_config = os.path.join(ROOT, "dist", "priconfig.xml")
cmd_config = [
    MAKEPRI, "createconfig",
    "/cf", pri_config,
    "/dq", "en-US",
    "/pv", "10.0.0",
    "/o"
]
subprocess.run(cmd_config, check=True, capture_output=True)

pri_out = os.path.join(STAGE, "resources.pri")
cmd_pri = [
    MAKEPRI, "new",
    "/pr", STAGE,
    "/cf", pri_config,
    "/of", pri_out,
    "/o"
]
pri_res = subprocess.run(cmd_pri, capture_output=True, text=True)
print(pri_res.stdout)
if pri_res.returncode != 0:
    print("makepri failed:", pri_res.stderr)
    sys.exit(1)

print("6. Packing MSIX package with makeappx.exe...")
cmd_pack = [
    MAKEAPPX, "pack",
    "/d", STAGE,
    "/p", OUT_MSIX,
    "/o"
]
pack_res = subprocess.run(cmd_pack, capture_output=True, text=True)
print(pack_res.stdout)
if pack_res.returncode != 0:
    print("makeappx failed:", pack_res.stderr)
    sys.exit(1)

print(f"\n[SUCCESS] Built MSIX: {OUT_MSIX} ({os.path.getsize(OUT_MSIX):,} bytes)")
