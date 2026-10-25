# Pixora Studio — Licensing & Compliance Guide

## Summary

Pixora Studio is open-source software published by **IT Amadersomaj Inc.**

The project is dual-licensed under the terms of:
- **MIT License** ([`LICENSE-MIT`](LICENSE-MIT))
- **Apache License, Version 2.0** ([`LICENSE-APACHE`](LICENSE-APACHE))

You may choose to use Pixora Studio under the terms of either license.

---

## Fork Information & Copyright Notices

Pixora Studio is an independent open-source fork of PhotoCraft (originally created by the ArtCraft Team and PhotoCraft contributors).

- **Current Rebrand & Modifications:**
  Copyright (c) 2026 IT Amadersomaj Inc.
- **Original Codebase & Contributions:**
  Portions copyright (c) 2026 ArtCraft Team and the PhotoCraft contributors.

### Independence Notice
Pixora Studio is developed and maintained independently by **IT Amadersomaj Inc.** It is not affiliated with, sponsored by, or endorsed by the original ArtCraft Team, Learning Machines LLC, or Adobe Systems Inc.

All proprietary brand assets and trademarks of the original upstream project (including the ArtCraft mark and logos previously residing in `docs/brand/`) have been removed in accordance with their brand guidelines.

---

## Modifications by IT Amadersomaj Inc.

IT Amadersomaj Inc. has made the following comprehensive modifications:
1. **Product Rebranding:**
   - Transformed identity to **Pixora Studio** with publisher attribution to **IT Amadersomaj Inc.**
   - Tagline established: *Professional Photo Editing. Free for Everyone.*
   - Modernized UI start canvas, About dialog, titlebar, window viewport headers, and help menus.
2. **Visual Identity & Iconography:**
   - Introduced original Pixora Studio logo and iconography.
   - Built multi-resolution Windows `.ico` packaging (16, 24, 32, 48, 64, 128, 256 px).
   - Generated native high-resolution vector and raster assets.
3. **Windows Native PE & Packaging:**
   - Configured Windows PE metadata (`ProductName = "Pixora Studio"`, `CompanyName = "IT Amadersomaj Inc."`, `FileDescription = "Pixora Studio — Professional Photo Editor"`).
   - Updated WiX Toolset installer scripts (`photocraft.wxs`, `photocraft.en-us.wxl`, `package.ps1`) for per-machine MSI installer and standalone portable ZIP distribution.
   - Preserved static CRT linking (`+crt-static`) for zero-dependency execution across Windows 10/11 x64.
4. **CI/CD & Release Workflow:**
   - Implemented self-contained automated GitHub Actions release workflow for Windows x64.
   - Added automated SHA-256 checksum generation for verified releases.

---

## Third-Party Assets & Dependencies

Pixora Studio bundles permissive third-party assets and dependencies in compliance with their respective licenses:

| Component | Source / Author | License | License Reference |
|---|---|---|---|
| **Inter Font** | Rasmus Andersson / The Inter Project | SIL OFL 1.1 | [`assets/fonts/OFL-Inter.txt`](assets/fonts/OFL-Inter.txt) |
| **JetBrains Mono** | JetBrains s.r.o. | SIL OFL 1.1 | [`assets/fonts/OFL-JetBrainsMono.txt`](assets/fonts/OFL-JetBrainsMono.txt) |
| **Lucide Icons** | Lucide Contributors | ISC / MIT | [`assets/icons/LICENSE-lucide.txt`](assets/icons/LICENSE-lucide.txt) |
| **SCOWL Wordlist** | Kevin Atkinson | Permissive (SCOWL) | [`assets/dict/LICENSE-SCOWL.txt`](assets/dict/LICENSE-SCOWL.txt) |
| **libfacedetection** | Shiqi Yu | BSD-3-Clause | [`crates/algo/data/LICENSE-libfacedetection.txt`](crates/algo/data/LICENSE-libfacedetection.txt) |
| **X11 Colors** | X.Org Foundation | MIT | In source (`crates/codecs/src/codecs/xpm/x11_colors.rs`) |

For a complete per-file inventory of third-party attributions, consult [`ATTRIBUTION.md`](ATTRIBUTION.md) and [`NOTICE`](NOTICE).
