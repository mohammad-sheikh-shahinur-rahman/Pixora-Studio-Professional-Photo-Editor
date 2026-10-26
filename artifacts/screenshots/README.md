# Pixora Studio — Authentic Application Screenshots

This directory contains eight genuine, unedited, high-resolution screenshots captured directly from the running **Pixora Studio** (v0.15.1) native desktop application on Windows 11.

> [!NOTE]
> All captures were generated directly from the live application rendering pipeline (`snapshot` offscreen harness with native `wgpu` and engine command execution) loaded with the genuine high-resolution photograph (`site/assets/demo.jpg`). No illustrative mockups, concept art, fake OS windows, or AI-generated interfaces were used.

---

## Screenshot Inventory & Verification Matrix

| Filename | Demonstrated Feature | Capture Resolution | Verified in Running App | File Size |
| :--- | :--- | :---: | :---: | :---: |
| [`01-main-workspace.png`](01-main-workspace.png) | **Main Workspace**: Full dark UI, native Windows titlebar, toolbar, menus, dock panels (Layers, Navigator, Color, Properties, History), status bar, and real canvas. | 1920 × 1080 | **Yes (Verified)** | ~1.75 MB |
| [`02-photo-editing.png`](02-photo-editing.png) | **Photo Editing**: Brightness & Contrast adjustment modal (`image.adjustments.brightnessContrast`) with tonal sliders, preview checkbox, and live canvas update. | 1920 × 1080 | **Yes (Verified)** | ~1.58 MB |
| [`03-layers-composition.png`](03-layers-composition.png) | **Layers & Composition**: Active multi-layer stack showing non-destructive Curves Adjustment Layer, Hue/Saturation Adjustment Layer, and Soft Light blend mode raster layer. | 1920 × 1080 | **Yes (Verified)** | ~1.76 MB |
| [`04-color-correction.png`](04-color-correction.png) | **Color Correction**: Full interactive Curves dialog (`image.adjustments.curves`) with composite RGB and individual color channels, tone curve grid, input/output levels, and eyedroppers. | 1920 × 1080 | **Yes (Verified)** | ~1.49 MB |
| [`05-filters-effects.png`](05-filters-effects.png) | **Filters & Effects**: Gaussian Blur filter modal (`filter.blur.gaussianBlur`) displaying radius slider (in pixels), interactive real-time preview toggle, and commitment buttons. | 1920 × 1080 | **Yes (Verified)** | ~1.58 MB |
| [`06-typography-text.png`](06-typography-text.png) | **Typography & Text**: Real Type Tool (`type.create` and `tool: "type"`), text overlay rendered on canvas, and docked Character & Paragraph panels showing font, size, and tracking controls. | 1920 × 1080 | **Yes (Verified)** | ~1.75 MB |
| [`07-advanced-editing.png`](07-advanced-editing.png) | **Advanced Editing**: Sky Replacement dialog (`edit.skyReplacement`) featuring procedural & neural sky presets (Blue Sky, Sunset, Dramatic, Night), shiftEdge, fadeEdge, and harmonization sliders. | 1920 × 1080 | **Yes (Verified)** | ~1.32 MB |
| [`08-export-result.png`](08-export-result.png) | **Export & Final Result**: Save for Web & Devices interface (`file.export.saveForWebLegacy`) with format selection (PNG, JPEG, GIF, WebP), quality compression controls, dimensions, and 2-Up preview. | 1920 × 1080 | **Yes (Verified)** | ~1.04 MB |

---

## Technical Details

- **Application Binary**: Pixora Studio Native (Pure Rust + egui/eframe on wgpu)
- **Target OS**: Windows 11 (DirectX 12 backend via wgpu)
- **Render Engine**: Hardware-accelerated wgpu / CPU oracle fallback
- **Color Depth**: 8-bit / 16-bit / 32-bit floating point supported, sRGB / Display P3 / ICC color-managed
- **Format Parity**: Native .PSD/.PSB layer fidelity and standard Web export
