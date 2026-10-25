//! Sky Replacement: detect sky, render procedural or custom replacement skies,
//! refine edges, and harmonize ambient foreground illumination.

use photocraft_geom::Rect;

use crate::noise::hash01;
use crate::segment::sky::{SkyParams, select_sky};
use crate::segment::Sampler;

/// Supported replacement sky presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyPreset {
    /// Crisp blue sky with gentle sunlit cumulus clouds.
    BlueSky,
    /// Radiant sunset / golden hour with warm amber, coral and glowing cirrus clouds.
    Sunset,
    /// Moody overcast with dramatic storm clouds and silver highlights.
    Dramatic,
    /// Deep midnight indigo with stars and an emerald aurora veil.
    Night,
}

impl SkyPreset {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "sunset" | "goldenhour" => Self::Sunset,
            "dramatic" | "storm" | "clouds" => Self::Dramatic,
            "night" | "aurora" => Self::Night,
            _ => Self::BlueSky,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::BlueSky => "Blue Sky",
            Self::Sunset => "Sunset",
            Self::Dramatic => "Dramatic Clouds",
            Self::Night => "Night Sky & Aurora",
        }
    }
}

/// Options for Sky Replacement.
#[derive(Clone, Debug, PartialEq)]
pub struct SkyReplaceParams {
    pub preset: SkyPreset,
    /// Shift sky boundary inward (< 0) or outward (> 0). Range -100..100.
    pub shift_edge: f32,
    /// Soften/feather the transition edge (0..100).
    pub fade_edge: f32,
    /// Sky brightness adjustment (-100..100).
    pub sky_brightness: f32,
    /// Sky color temperature: cooler/blue (< 0) to warmer/amber (> 0). Range -100..100.
    pub sky_temperature: f32,
    /// Scale factor for cloud details (100..300 %).
    pub sky_scale: f32,
    /// Mirror sky horizontally.
    pub flip: bool,
    /// Harmonize foreground ambient lighting with sky (0..100).
    pub foreground_lighting: f32,
}

impl Default for SkyReplaceParams {
    fn default() -> Self {
        Self {
            preset: SkyPreset::BlueSky,
            shift_edge: 0.0,
            fade_edge: 10.0,
            sky_brightness: 0.0,
            sky_temperature: 0.0,
            sky_scale: 100.0,
            flip: false,
            foreground_lighting: 20.0,
        }
    }
}

/// Fractional Brownian motion noise for cloud synthesis.
fn fbm(x: f32, y: f32, seed: u32) -> f32 {
    let mut sum = 0.0f32;
    let mut amp = 0.5f32;
    let mut freq = 1.0f32;
    for o in 0..5u32 {
        let (fx, fy) = (x * freq, y * freq);
        let (ix, iy) = (fx.floor() as i32, fy.floor() as i32);
        let (tx, ty) = (fx - ix as f32, fy - iy as f32);
        let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
        let s = seed.wrapping_add(o * 1013);
        let h = |a: i32, b: i32| hash01(a, b, 0, s);
        let top = h(ix, iy) + (h(ix + 1, iy) - h(ix, iy)) * sx;
        let bot = h(ix, iy + 1) + (h(ix + 1, iy + 1) - h(ix, iy + 1)) * sx;
        sum += (top + (bot - top) * sy) * amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum
}

/// Renders a procedural replacement sky surface at `(w, h)` resolution.
pub fn render_sky(preset: SkyPreset, w: usize, h: usize, p: &SkyReplaceParams) -> Vec<[f32; 3]> {
    let mut out = Vec::with_capacity(w * h);
    let scale = (p.sky_scale / 100.0).clamp(0.5, 5.0);
    let bright = ((p.sky_brightness / 100.0) * 0.6).exp();
    let temp = p.sky_temperature / 100.0;

    let (r_mult, b_mult) = if temp >= 0.0 {
        (1.0 + temp * 0.35, 1.0 - temp * 0.25)
    } else {
        (1.0 + temp * 0.25, 1.0 - temp * 0.35)
    };

    let seed = match preset {
        SkyPreset::BlueSky => 42,
        SkyPreset::Sunset => 1337,
        SkyPreset::Dramatic => 9999,
        SkyPreset::Night => 7777,
    };

    for y in 0..h {
        let v = y as f32 / (h as f32).max(1.0); // 0 at top, 1 at horizon
        for x in 0..w {
            let mut u = x as f32 / (w as f32).max(1.0);
            if p.flip {
                u = 1.0 - u;
            }

            let rgb = match preset {
                SkyPreset::BlueSky => {
                    // Sky gradient: deep rich azure at top to soft pale cyan at horizon
                    let sky_top = [0.12f32, 0.42, 0.88];
                    let sky_bot = [0.72f32, 0.86, 0.98];
                    let t = (v * 1.2).clamp(0.0, 1.0);
                    let base_r = sky_top[0] + (sky_bot[0] - sky_top[0]) * t;
                    let base_g = sky_top[1] + (sky_bot[1] - sky_top[1]) * t;
                    let base_b = sky_top[2] + (sky_bot[2] - sky_top[2]) * t;

                    // Cumulus clouds in lower-middle atmosphere
                    let cloud_noise = fbm(u * 4.0 * scale, v * 3.0 * scale + 0.5, seed);
                    let cloud_density = ((cloud_noise - 0.45) * 2.8).clamp(0.0, 1.0) * (1.0 - (v - 0.5).abs() * 1.5).max(0.0);

                    let c_r = base_r * (1.0 - cloud_density * 0.9) + 0.98 * cloud_density * 0.9;
                    let c_g = base_g * (1.0 - cloud_density * 0.9) + 0.98 * cloud_density * 0.9;
                    let c_b = base_b * (1.0 - cloud_density * 0.9) + 1.00 * cloud_density * 0.9;
                    [c_r, c_g, c_b]
                }
                SkyPreset::Sunset => {
                    // Sunset gradient: violet-indigo -> coral-orange -> golden amber at horizon
                    let top = [0.22f32, 0.08, 0.32];
                    let mid = [0.90f32, 0.32, 0.18];
                    let bot = [1.00f32, 0.78, 0.22];

                    let (base_r, base_g, base_b) = if v < 0.5 {
                        let t = v * 2.0;
                        (top[0] + (mid[0] - top[0]) * t, top[1] + (mid[1] - top[1]) * t, top[2] + (mid[2] - top[2]) * t)
                    } else {
                        let t = (v - 0.5) * 2.0;
                        (mid[0] + (bot[0] - mid[0]) * t, mid[1] + (bot[1] - mid[1]) * t, mid[2] + (bot[2] - mid[2]) * t)
                    };

                    // Illuminated sunset cirrus streaks
                    let cloud_noise = fbm(u * 6.0 * scale, v * 1.5 * scale, seed);
                    let cloud_streak = ((cloud_noise - 0.40) * 3.0).clamp(0.0, 1.0) * (0.8 - (v - 0.4).abs()).max(0.0);

                    let c_r = (base_r * (1.0 - cloud_streak * 0.7) + 1.0 * cloud_streak * 0.7).min(1.0);
                    let c_g = (base_g * (1.0 - cloud_streak * 0.7) + 0.75 * cloud_streak * 0.7).min(1.0);
                    let c_b = (base_b * (1.0 - cloud_streak * 0.7) + 0.35 * cloud_streak * 0.7).min(1.0);
                    [c_r, c_g, c_b]
                }
                SkyPreset::Dramatic => {
                    // Overcast stormy slate clouds
                    let top = [0.15f32, 0.17, 0.22];
                    let bot = [0.45f32, 0.48, 0.52];
                    let t = v.clamp(0.0, 1.0);
                    let base_r = top[0] + (bot[0] - top[0]) * t;
                    let base_g = top[1] + (bot[1] - top[1]) * t;
                    let base_b = top[2] + (bot[2] - top[2]) * t;

                    let turb = fbm(u * 5.0 * scale, v * 4.0 * scale, seed);
                    let contrast = ((turb - 0.5) * 1.5 + 0.5).clamp(0.0, 1.0);

                    [base_r * contrast * 1.2, base_g * contrast * 1.2, base_b * contrast * 1.2]
                }
                SkyPreset::Night => {
                    // Midnight navy -> deep dark emerald aurora glow
                    let top = [0.03f32, 0.05, 0.12];
                    let bot = [0.06f32, 0.12, 0.20];
                    let t = v.clamp(0.0, 1.0);
                    let mut r = top[0] + (bot[0] - top[0]) * t;
                    let mut g = top[1] + (bot[1] - top[1]) * t;
                    let mut b = top[2] + (bot[2] - top[2]) * t;

                    // Aurora curtain waving across horizon
                    let aurora_wave = fbm(u * 3.0 * scale, v * 2.0 * scale + 0.2, seed);
                    let aurora_intensity = ((aurora_wave - 0.42) * 2.5).clamp(0.0, 1.0) * (1.0 - (v - 0.65).abs() * 3.0).max(0.0);
                    g += aurora_intensity * 0.75;
                    b += aurora_intensity * 0.45;

                    // Tiny twinkling star specks in upper sky
                    if v < 0.6 {
                        let star_hash = hash01(x as i32, y as i32, 0, 777);
                        if star_hash > 0.9985 {
                            let star_lum = (star_hash - 0.9985) / 0.0015;
                            r = (r + star_lum * 0.9).min(1.0);
                            g = (g + star_lum * 0.9).min(1.0);
                            b = (b + star_lum * 1.0).min(1.0);
                        }
                    }

                    [r, g, b]
                }
            };

            // Apply temperature and brightness
            let final_r = (rgb[0] * r_mult * bright).clamp(0.0, 1.0);
            let final_g = (rgb[1] * bright).clamp(0.0, 1.0);
            let final_b = (rgb[2] * b_mult * bright).clamp(0.0, 1.0);
            out.push([final_r, final_g, final_b]);
        }
    }
    out
}

/// Refines sky mask with Shift Edge (-100..100) and Fade Edge (0..100).
pub fn refine_sky_mask(mask: &[u8], w: usize, h: usize, shift_edge: f32, fade_edge: f32) -> Vec<u8> {
    if mask.is_empty() || w == 0 || h == 0 {
        return Vec::new();
    }
    let mut out = mask.to_vec();

    // 1. Shift edge: if shift > 0, expand sky (dilate); if shift < 0, contract sky (erode)
    let shift_steps = ((shift_edge / 100.0) * 4.0).round() as i32;
    if shift_steps != 0 {
        let mut shifted = out.clone();
        let dilate = shift_steps > 0;
        let steps = shift_steps.unsigned_abs() as usize;
        for _ in 0..steps {
            let src = shifted.clone();
            for y in 0..h {
                for x in 0..w {
                    let idx = y * w + x;
                    let mut val = src[idx];
                    let neighbors = [
                        (x > 0).then(|| idx - 1),
                        (x + 1 < w).then(|| idx + 1),
                        (y > 0).then(|| idx - w),
                        (y + 1 < h).then(|| idx + w),
                    ];
                    for n in neighbors.into_iter().flatten() {
                        if dilate {
                            val = val.max(src[n]);
                        } else {
                            val = val.min(src[n]);
                        }
                    }
                    shifted[idx] = val;
                }
            }
        }
        out = shifted;
    }

    // 2. Fade edge: smooth the boundary transition
    let radius = ((fade_edge / 100.0) * 8.0).round() as usize;
    if radius > 0 {
        let mut blurred = vec![0u8; w * h];
        let r = radius.min(16);
        for y in 0..h {
            for x in 0..w {
                let mut sum = 0u32;
                let mut count = 0u32;
                let y0 = y.saturating_sub(r);
                let y1 = (y + r).min(h - 1);
                let x0 = x.saturating_sub(r);
                let x1 = (x + r).min(w - 1);
                for ny in y0..=y1 {
                    for nx in x0..=x1 {
                        sum += u32::from(out[ny * w + nx]);
                        count += 1;
                    }
                }
                blurred[y * w + x] = (sum / count.max(1)) as u8;
            }
        }
        out = blurred;
    }

    out
}

/// Compute average ambient light color from a sky surface.
pub fn sky_ambient_color(sky_rgb: &[[f32; 3]]) -> [f32; 3] {
    if sky_rgb.is_empty() {
        return [0.5, 0.5, 0.5];
    }
    let mut sum_r = 0.0f32;
    let mut sum_g = 0.0f32;
    let mut sum_b = 0.0f32;
    for &[r, g, b] in sky_rgb {
        sum_r += r;
        sum_g += g;
        sum_b += b;
    }
    let n = sky_rgb.len() as f32;
    [sum_r / n, sum_g / n, sum_b / n]
}

/// Harmonizes a foreground RGB pixel with the ambient sky lighting.
#[inline]
pub fn harmonize_pixel(fg: [f32; 3], ambient: [f32; 3], strength: f32) -> [f32; 3] {
    let s = (strength / 100.0).clamp(0.0, 1.0) * 0.35;
    [
        (fg[0] * (1.0 - s) + (fg[0] * ambient[0] * 1.5) * s).clamp(0.0, 1.0),
        (fg[1] * (1.0 - s) + (fg[1] * ambient[1] * 1.5) * s).clamp(0.0, 1.0),
        (fg[2] * (1.0 - s) + (fg[2] * ambient[2] * 1.5) * s).clamp(0.0, 1.0),
    ]
}

/// Detects sky and performs replacement on pixel buffer.
/// Returns (processed_rgb, sky_detected).
pub fn replace_sky_in_buffer(
    sampler: &dyn Sampler,
    canvas: Rect,
    fg_rgb: &[[f32; 3]],
    p: &SkyReplaceParams,
) -> Option<Vec<[f32; 3]>> {
    let w = canvas.width() as usize;
    let h = canvas.height() as usize;
    if w == 0 || h == 0 || fg_rgb.len() != w * h {
        return None;
    }

    // 1. Detect sky region
    let region = select_sky(sampler, canvas, SkyParams::default())?;
    let raw_mask = region.mask;

    // 2. Refine mask
    let mask = refine_sky_mask(&raw_mask, w, h, p.shift_edge, p.fade_edge);

    // 3. Render replacement sky
    let sky = render_sky(p.preset, w, h, p);
    let ambient = sky_ambient_color(&sky);

    // 4. Blend foreground and replacement sky with harmonization
    let mut result = Vec::with_capacity(w * h);
    for i in 0..w * h {
        let sky_alpha = f32::from(mask.get(i).copied().unwrap_or(0)) / 255.0;
        let orig_fg = fg_rgb[i];
        let harmonized_fg = harmonize_pixel(orig_fg, ambient, p.foreground_lighting);
        let sky_color = sky[i];

        let r = harmonized_fg[0] * (1.0 - sky_alpha) + sky_color[0] * sky_alpha;
        let g = harmonized_fg[1] * (1.0 - sky_alpha) + sky_color[1] * sky_alpha;
        let b = harmonized_fg[2] * (1.0 - sky_alpha) + sky_color[2] * sky_alpha;
        result.push([r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_presets_render_smoothly() {
        let p = SkyReplaceParams::default();
        let sky = render_sky(SkyPreset::BlueSky, 64, 48, &p);
        assert_eq!(sky.len(), 64 * 48);
        // Top should be bluer than bottom
        let top = sky[0];
        let bot = sky[64 * 47];
        assert!(top[2] > top[0], "blue dominates top: {:?}", top);
        assert!(bot[0] > top[0], "bottom is brighter/warmer: bot={:?} top={:?}", bot, top);

        let sunset = render_sky(SkyPreset::Sunset, 64, 48, &p);
        assert_eq!(sunset.len(), 64 * 48);
        let sun_bot = sunset[64 * 47];
        assert!(sun_bot[0] > sun_bot[2], "sunset horizon is warm/red: {:?}", sun_bot);
    }

    #[test]
    fn refine_mask_handles_shift_and_fade() {
        let mut mask = vec![0u8; 16 * 16];
        for y in 0..8 {
            for x in 0..16 {
                mask[y * 16 + x] = 255;
            }
        }
        let refined = refine_sky_mask(&mask, 16, 16, 0.0, 20.0);
        assert_eq!(refined.len(), 16 * 16);
        // Boundary around row 7-8 should be feathered between 0 and 255
        let mid = refined[8 * 16 + 8];
        assert!(mid < 255, "feathered boundary: {}", mid);
    }

    #[test]
    fn ambient_harmonization_tints_foreground() {
        let fg = [0.5, 0.5, 0.5];
        let warm_sky = [0.9, 0.6, 0.2];
        let harmonized = harmonize_pixel(fg, warm_sky, 50.0);
        assert!(harmonized[0] > fg[0], "red boosted by warm sky: {:?}", harmonized);
        assert!(harmonized[2] < harmonized[0], "warm sky shifts balance: {:?}", harmonized);
    }
}
