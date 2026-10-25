<p align="center">
  <img alt="Pixora Studio" src="docs/brand/pixora-studio-logo.png" width="360">
</p>

<p align="center">
  <img src="assets/app-icon/hicolor/256x256/apps/com.amadersomaj.pixorastudio.png" width="128" height="128" alt="Pixora Studio Icon">
</p>

<h1 align="center">Pixora Studio</h1>

<p align="center">
  <b>Professional Photo Editing. Free for Everyone.</b><br>
  Published by <b>IT Amadersomaj Inc.</b><br>
  Layers, masks, adjustment layers, layer styles, typography, vectors, brushes, and true PSD support,<br>
  in a native desktop application written entirely in Rust. Free, open source, offline, and yours.
</p>

<p align="center">
  <img alt="100% Rust" src="https://img.shields.io/badge/100%25-Rust-b7410e?style=flat-square&logo=rust">
  <img alt="Windows 10/11 x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20x64-0078d4?style=flat-square&logo=windows">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square">
  <img alt="Distribution: Free" src="https://img.shields.io/badge/distribution-Free-success?style=flat-square">
</p>

<br>

<p align="center">
  <img src="docs/images/photocraft-demo.jpg" alt="Pixora Studio editing Hokusai's The Great Wave with Curves and Vibrance adjustment layers and drop shadows" width="100%">
  <br>
  <sub>Pixora Studio running native hardware-accelerated composition with live type, adjustment layers, and drop shadow styles.</sub>
</p>

<br>

<table>
  <tr>
    <td width="25%" valign="top">
      <h3>🎛️ Familiar by Design</h3>
      Menus, keyboard shortcuts, tools, and dockable panels are right where professional editors expect them. From Ctrl+J to Ctrl+T, your muscle memory works out of the box.
    </td>
    <td width="25%" valign="top">
      <h3>⚡ Native & Lightning Fast</h3>
      Pure Rust with a high-performance GPU compositor on <b>wgpu</b> (DirectX 12 and Vulkan), tile-based multithreading, and copy-on-write memory. Zero Electron, zero webview, zero lag.
    </td>
    <td width="25%" valign="top">
      <h3>🗂️ True PSD Fidelity</h3>
      Open, edit, and save layered Adobe Photoshop documents (PSD/PSB). High-fidelity preservation of layer structures, blend modes, masks, and smart objects.
    </td>
    <td width="25%" valign="top">
      <h3>🔒 100% Offline & Free</h3>
      No subscriptions, no mandatory accounts, no cloud dependencies, and zero telemetry. Pixora Studio runs completely on your machine and belongs to you.
    </td>
  </tr>
</table>

<br>

## Key Features

### Professional Image Editing & Layer Stack
- **Full Layer Management:** Raster layers, vector shape layers, type layers, group folders, and adjustment layers.
- **Masking:** Pixel masks, vector clipping paths, clipping masks, and layer style blending options.
- **Non-Destructive Adjustments:** Curves, Levels, Hue/Saturation, Vibrance, Color Balance, Black & White, Threshold, and Invert.
- **Layer Effects (FX):** Drop Shadows, Inner Shadows, Outer Glow, Inner Glow, Bevel & Emboss, Color Overlay, and Stroke.
- **Blending Modes:** 27 industry-standard blend modes matching ISO 32000 specifications (Multiply, Screen, Overlay, Soft Light, Linear Dodge, etc.).

### Precision Tools & Canvas
- **Selection Suite:** Rectangular & Elliptical Marquee, Freehand Lasso, Polygonal Lasso, Magnetic Lasso, Magic Wand, and Quick Selection.
- **Transformation:** Free Transform (Scale, Rotate, Skew, Distort, Perspective) with high-precision bicubic/bilinear interpolation.
- **Retouching & Brushes:** Healing Brush, Clone Stamp, Spot Removal, Eraser, Gradient, Paint Bucket, Blur, Sharpen, and Smudge.
- **Color Management:** High-precision ICC color workflows via `photocraft-cms`, supporting 8-bit, 16-bit integer, and 32-bit floating point pipelines.

### Supported File Formats
- **Native Project:** `.pcraft` (high-speed transactional document format)
- **Photoshop:** `.psd`, `.psb` (read and write)
- **Standard Raster:** PNG, JPEG, TIFF, WebP, GIF, BMP, TGA, ICO, QOI
- **High Dynamic Range:** OpenEXR (`.exr`), Radiance HDR (`.hdr`), AVIF (`.avif`)

---

## Downloads (Windows 10/11 x64)

Pixora Studio is distributed for 64-bit Windows systems. Binaries link the C runtime statically (`+crt-static`), requiring no external Visual C++ runtime installation.

| Package | Description | Download |
|---|---|---|
| **Windows Setup Installer (`.exe`)** | Self-contained single-click executable setup installer | [GitHub Releases](https://github.com/amadersomaj/pixora-studio/releases) |
| **Windows MSI Installer (`.msi`)** | Per-machine Windows installer with Start Menu shortcuts and file associations | [GitHub Releases](https://github.com/amadersomaj/pixora-studio/releases) |
| **Portable Edition (`.zip`)** | Zero-install standalone archive. Keeps settings in `PixoraStudioData\` beside the executable | [GitHub Releases](https://github.com/amadersomaj/pixora-studio/releases) |

---

## Building from Source

### Prerequisites
- **Operating System:** Windows 10/11 (x64)
- **Rust Toolchain:** Stable Rust 1.85+ (`rustup default stable`)
- **C++ Build Tools:** Visual Studio 2022 Build Tools (MSVC with Windows SDK)
- **WiX Toolset (Optional, for MSI and EXE packaging):**
  ```powershell
  dotnet tool install --global wix --version 5.0.2
  wix extension add -g WixToolset.UI.wixext/5.0.2 WixToolset.Util.wixext/5.0.2 WixToolset.BootstrapperApplications.wixext/5.0.2
  ```

### Quick Build
```powershell
# Clone the repository
git clone https://github.com/amadersomaj/pixora-studio.git
cd pixora-studio

# Build release binaries
cargo build --release -p photocraft -p photocraft-cli
```
The compiled binaries will be generated at:
- `target\release\photocraft.exe` (Pixora Studio GUI)
- `target\release\photocraft-cli.exe` (Pixora Studio CLI)

### Package Installers and Portable ZIP
```powershell
pwsh packaging/windows/package.ps1 -Arch x64
```
Artifacts are produced in `dist\release\`:
- `pixora-studio-0.15.1-windows-x64-installer.exe`
- `pixora-studio-0.15.1-windows-x64.msi`
- `pixora-studio-0.15.1-windows-x64-portable.zip`

---

## Open-Source License & Credits

Pixora Studio is free software published by **IT Amadersomaj Inc.**

- **License:** Dual-licensed under the **MIT License** ([`LICENSE-MIT`](LICENSE-MIT)) and **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE)).
- **Fork Heritage:** Pixora Studio is an independent fork derived from PhotoCraft (created by the ArtCraft Team and open-source contributors). Portions copyright &copy; 2026 ArtCraft Team and PhotoCraft contributors.
- **Trademark Notice:** All original ArtCraft brand assets and logos have been completely removed and replaced with the original Pixora Studio visual identity. Pixora Studio is not affiliated with or endorsed by Adobe Systems Inc. or the original ArtCraft Team.
- For third-party font, icon, and library licenses, see [`LICENSES.md`](LICENSES.md), [`NOTICE`](NOTICE), and [`ATTRIBUTION.md`](ATTRIBUTION.md).
