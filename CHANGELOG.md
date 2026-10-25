# Pixora Studio — Changelog

All notable changes to Pixora Studio are documented in this file.

## [0.15.1] - 2026-10-25

### Rebranding & Product Identity
- **Product Name:** Rebranded to **Pixora Studio** by **IT Amadersomaj Inc.**
- **Tagline:** *Professional Photo Editing. Free for Everyone.*
- **Visual Identity:**
  - Designed and implemented original Pixora Studio logo and iconography.
  - Generated multi-resolution Windows ICO (`assets/app-icon/pixora.ico`) supporting 16, 24, 32, 48, 64, 128, and 256 px.
  - Integrated high-resolution master graphics and SVG assets (`docs/brand/pixora-studio-logo.svg`, `assets/app-icon/pixora.svg`).
  - Completely replaced upstream ArtCraft trademarks, wordmarks, and logos in accordance with open-source trademark separation requirements.
- **Application UI:**
  - Modernized start canvas with Pixora Studio branding, subtitle, and tagline.
  - Updated window titles and viewport headers (`Pixora Studio` and `<Doc> — Pixora Studio`).
  - Updated About dialog with IT Amadersomaj Inc. attribution, copyright notice, and links.
  - Updated Help and Application menus (`About Pixora Studio`, website, GitHub links).
  - Updated internal app ID to `com.amadersomaj.pixorastudio`.

### Windows Native Integration & Packaging
- **Windows PE Metadata:**
  - Configured `ProductName` = "Pixora Studio".
  - Configured `CompanyName` = "IT Amadersomaj Inc.".
  - Configured `FileDescription` = "Pixora Studio — Professional Photo Editor".
  - Embedded multi-resolution `pixora.ico` into `photocraft.exe` and `photocraft-cli.exe`.
- **WiX Installer (MSI):**
  - Updated WiX v5 configuration (`photocraft.wxs` and `photocraft.en-us.wxl`).
  - Added Start Menu shortcuts and file associations (`.pcraft`, PSD/PSB, PNG, JPEG, TIFF, WEBP, etc.) under the Pixora Studio brand.
  - Set installation directory to `Program Files\Pixora Studio`.
- **Portable Distribution:**
  - Added portable mode storing configuration, recovery documents, and presets in `PixoraStudioData\`.
  - Configured portable ZIP archive packaging (`pixora-studio-0.15.1-windows-x64-portable.zip`).
- **Static CRT:**
  - Preserved static C runtime linking (`+crt-static`) so that neither the MSI installer nor the portable ZIP requires Microsoft Visual C++ Redistributable.

### Documentation & Compliance
- Added comprehensive `LICENSES.md` detailing dual MIT / Apache-2.0 licensing, original author copyright preservation, and third-party dependency licenses.
- Updated `NOTICE` and `ATTRIBUTION.md` reflecting IT Amadersomaj Inc. stewardship and original upstream credits.
- Added `BRANDING.md` design guide and color palette documentation.
- Rewrote `README.md` to feature Pixora Studio capabilities, feature highlights, and Windows build instructions.
- Added automated GitHub Actions release workflow producing signed packages and SHA-256 checksums (`SHA256SUMS.txt`).
