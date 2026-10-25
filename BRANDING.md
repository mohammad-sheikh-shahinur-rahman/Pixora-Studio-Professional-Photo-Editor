# Pixora Studio — Brand Identity & Design System

## Overview

**Pixora Studio** is a professional desktop photo editor developed and maintained by **IT Amadersomaj Inc.**
- **Product Name:** Pixora Studio
- **Publisher:** IT Amadersomaj Inc.
- **Short Description:** Professional Photo Editing Software
- **Tagline:** *Professional Photo Editing. Free for Everyone.*
- **Distribution:** Free, Open Source (MIT & Apache-2.0)
- **Target Platform:** Windows 10/11 x64 (Native Rust + wgpu)
- **Application ID:** `com.amadersomaj.pixorastudio`

---

## Visual Design Direction

Pixora Studio uses a refined, distraction-free creative workspace inspired by classic darkroom workflows and modern professional creative applications.

### 1. Interface Color Palette

| Role | Color | Hex Code | Usage |
|---|---|---|---|
| **Base Surface** | Studio Dark | `#18191B` | App background, titlebar, and panel framing |
| **Card / Panel Surface** | Neutral Surface | `#232529` | Toolbars, sidebars, dockable inspector panels |
| **Canvas Background** | Neutral Dark | `#2D2F34` | Document viewport backdrop behind active artboards |
| **Accent Primary** | Pixora Blue | `#2F7BF5` | Active tool indicators, focus outlines, primary buttons |
| **Accent Glow** | Electric Azure | `#4C8DFF` | Hover states, active selection highlights, handles |
| **Text Primary** | High Contrast White | `#F3F4F6` | Labels, values, menus, active tool tips |
| **Text Secondary** | Muted Slate | `#9CA3AF` | Inactive properties, shortcuts, helper text |
| **Borders & Dividers** | Subtle Hairline | `#374151` | Panel dividers, input borders, grid rulers |

### 2. Typography

- **Interface UI:** **Inter** (Regular, Medium, SemiBold) — clean legibility at all Windows scaling factors (100%, 125%, 150%, 200%).
- **Numeric & Technical:** **JetBrains Mono** — precise coordinate grids, histogram statistics, transform angles, pixel dimensions.

---

## Application Icon & Identity

The official Pixora Studio icon represents precision optics, digital layering, and modern creative editing.

### Master Asset Specifications
- **Master Resolution:** `1024x1024` pixels (`branding/refined/pixora-studio-refined-1024.png`)
- **Windows Executable & Desktop ICO:** `assets/app-icon/pixora.ico`
  - Multi-resolution Windows ICO package containing 7 discrete mip levels: `16x16`, `24x24`, `32x32`, `48x48`, `64x64`, `128x128`, `256x256`.
- **Vector Assets:** `assets/app-icon/pixora.svg`, `docs/brand/pixora-studio-logo.svg`
- **Linux / FreeDesktop Hicolor Icon Theme:** `assets/app-icon/hicolor/<size>/apps/com.amadersomaj.pixorastudio.png`

---

## Guidelines for Contributors & Distributors

1. **Independent Identity:** Pixora Studio is an independent open-source project by IT Amadersomaj Inc. Do not use proprietary trademarks or logos from Adobe Systems or third-party organizations in a way that suggests affiliation or endorsement.
2. **Original Assets:** All custom brand assets, logos, and identity graphics created for Pixora Studio are licensed under the MIT and Apache-2.0 licenses as part of the project repository.
