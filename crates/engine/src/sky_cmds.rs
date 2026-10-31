//! Edit › Sky Replacement command: detect sky, render procedural/custom sky preset,
//! and blend into document either as active layer or non-destructive layers.

use photocraft_algo::segment::sky::{self, SkyParams};
use photocraft_algo::segment::Sampler;
use photocraft_algo::sky_replace::{SkyPreset, SkyReplaceParams, harmonize_pixel, refine_sky_mask, render_sky, sky_ambient_color};
use photocraft_doc::{Document, Layer, LayerContent, LayerMask};
use photocraft_geom::Rect;
use photocraft_raster::{Surface, from_rgba_into, to_rgba};
use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session};

pub const CMD: &str = "edit.skyReplacement";

fn bad(cmd: &str, msg: impl Into<String>) -> EngineError {
    EngineError::BadParams { cmd: cmd.into(), msg: msg.into() }
}

fn has_unlocked_layer(s: &Session) -> std::result::Result<(), String> {
    let d = s.active().ok_or("no document open")?;
    let id = d.active_layer.ok_or("no active layer")?;
    let l = d.doc.layer(id).ok_or("no such layer")?;
    if !matches!(l.content, LayerContent::Raster(_)) {
        return Err("the active layer is not a pixel layer".into());
    }
    let locks = d.doc.effective_locks(id);
    if locks.pixels || locks.all {
        return Err("the active layer is locked".into());
    }
    Ok(())
}

struct CompositeSampler<'a>(&'a Document);

impl Sampler for CompositeSampler<'_> {
    fn rgba(&self, r: Rect) -> Vec<[f32; 4]> {
        photocraft_compose::render(self.0, r).px
    }
}

fn parse_params(p: &Value) -> Result<(SkyReplaceParams, String)> {
    let preset_str = p.get("preset").and_then(Value::as_str).unwrap_or("blueSky");
    let preset = SkyPreset::parse(preset_str);

    let shift_edge = p.get("shiftEdge").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let fade_edge = p.get("fadeEdge").and_then(Value::as_f64).unwrap_or(10.0) as f32;
    let sky_brightness = p.get("skyBrightness").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let sky_temperature = p.get("skyTemperature").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let sky_scale = p.get("skyScale").and_then(Value::as_f64).unwrap_or(100.0) as f32;
    let flip = p.get("flip").and_then(Value::as_bool).unwrap_or(false);
    let foreground_lighting = p.get("foregroundLighting").and_then(Value::as_f64).unwrap_or(20.0) as f32;
    let output = p.get("output").and_then(Value::as_str).unwrap_or("newLayers").to_string();

    let params = SkyReplaceParams {
        preset,
        shift_edge: shift_edge.clamp(-100.0, 100.0),
        fade_edge: fade_edge.clamp(0.0, 100.0),
        sky_brightness: sky_brightness.clamp(-100.0, 100.0),
        sky_temperature: sky_temperature.clamp(-100.0, 100.0),
        sky_scale: sky_scale.clamp(50.0, 500.0),
        flip,
        foreground_lighting: foreground_lighting.clamp(0.0, 100.0),
    };

    Ok((params, output))
}

fn run(s: &mut Session, p: &Value) -> Result<Value> {
    let (sky_params, output) = parse_params(p)?;
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = d.doc.clone();
    let canvas = doc.bounds();
    let (w, h) = (canvas.width() as usize, canvas.height() as usize);
    if w < 4 || h < 4 {
        return Err(bad(CMD, "document too small for sky replacement"));
    }

    let active_id = d.active_layer.ok_or(EngineError::Other("no active layer".into()))?;

    // 1. Detect sky region
    let detected = sky::select_sky(&CompositeSampler(&doc), canvas, SkyParams::default());
    let Some(region) = detected else {
        return Ok(json!({
            "success": false,
            "changed": false,
            "message": "No sky was detected in the active image",
        }));
    };

    let b = region.bbox;
    let full_raw_mask = region.to_mask_in_rect(canvas);

    // 2. Refine mask (shift edge + fade edge)
    let refined_mask = refine_sky_mask(&full_raw_mask, w, h, sky_params.shift_edge, sky_params.fade_edge);

    // 3. Render replacement sky surface
    let sky_colors = render_sky(sky_params.preset, w, h, &sky_params);
    let ambient = sky_ambient_color(&sky_colors);

    // 4. Edit document
    s.edit("Sky Replacement", |doc, active| {
        let fmt = doc.pixel_format();
        let channels = fmt.channels();

        if output == "activeLayer" {
            // Blend directly into the active layer's surface
            let layer = doc.layer_mut(active_id).ok_or(EngineError::NoLayer(active_id))?;
            let surf = layer.surface_mut().ok_or(EngineError::Other("active layer has no pixel surface".into()))?;

            let orig_floats = surf.read_region(canvas);

            let mut blended = Vec::with_capacity(w * h * channels);
            let mut enc = vec![0.0f32; channels];
            for (i, &sky_rgb) in sky_colors.iter().enumerate().take(w * h) {
                let sky_alpha = f32::from(refined_mask.get(i).copied().unwrap_or(0)) / 255.0;
                let base = i * channels;
                let orig_px = orig_floats.get(base..base + channels).unwrap_or(&[]);
                let orig_rgba = to_rgba(&fmt, orig_px);

                let fg = harmonize_pixel([orig_rgba[0], orig_rgba[1], orig_rgba[2]], ambient, sky_params.foreground_lighting);

                let r = fg[0] * (1.0 - sky_alpha) + sky_rgb[0] * sky_alpha;
                let g = fg[1] * (1.0 - sky_alpha) + sky_rgb[1] * sky_alpha;
                let b = fg[2] * (1.0 - sky_alpha) + sky_rgb[2] * sky_alpha;
                let a = orig_rgba[3].max(sky_alpha);

                from_rgba_into(&fmt, [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a], &mut enc);
                blended.extend_from_slice(&enc);
            }
            surf.write_region(canvas, &blended);
        } else if output == "duplicateLayer" {
            let active_layer = doc.layer(active_id).ok_or(EngineError::NoLayer(active_id))?;
            let mut dup = active_layer.duplicate();
            if doc.is_background(active_id) {
                dup.name = "Background Copy".into();
                dup.locks = Default::default();
            } else {
                dup.name = format!("{} Copy", active_layer.name);
            }
            if let Some(surf) = dup.surface_mut() {
                let orig_floats = surf.read_region(canvas);
                let mut blended = Vec::with_capacity(w * h * channels);
                let mut enc = vec![0.0f32; channels];
                for (i, &sky_rgb) in sky_colors.iter().enumerate().take(w * h) {
                    let sky_alpha = f32::from(refined_mask.get(i).copied().unwrap_or(0)) / 255.0;
                    let base = i * channels;
                    let orig_px = orig_floats.get(base..base + channels).unwrap_or(&[]);
                    let orig_rgba = to_rgba(&fmt, orig_px);

                    let fg = harmonize_pixel([orig_rgba[0], orig_rgba[1], orig_rgba[2]], ambient, sky_params.foreground_lighting);

                    let r = fg[0] * (1.0 - sky_alpha) + sky_rgb[0] * sky_alpha;
                    let g = fg[1] * (1.0 - sky_alpha) + sky_rgb[1] * sky_alpha;
                    let b = fg[2] * (1.0 - sky_alpha) + sky_rgb[2] * sky_alpha;
                    let a = orig_rgba[3].max(sky_alpha);

                    from_rgba_into(&fmt, [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a], &mut enc);
                    blended.extend_from_slice(&enc);
                }
                surf.write_region(canvas, &blended);
            }
            let dup_id = dup.id;
            let active_idx = doc.layers.iter().position(|l| l.id == active_id).unwrap_or(0);
            doc.layers.insert(active_idx + 1, dup);
            *active = Some(dup_id);
        } else {
            // "newLayers": Non-destructive layer creation
            crate::extra_cmds::background_to_layer_for_mask(doc, active_id);

            // Invert the sky mask so the foreground layer reveals everything except the sky
            let mut inverted_mask = vec![0u8; w * h];
            for (i, m) in inverted_mask.iter_mut().enumerate().take(w * h) {
                *m = 255u8.saturating_sub(refined_mask.get(i).copied().unwrap_or(0));
            }

            // Create sky raster layer
            let mut sky_surf = Surface::new(fmt);
            let mut sky_data = Vec::with_capacity(w * h * channels);
            let mut enc = vec![0.0f32; channels];
            for &[r, g, b] in &sky_colors {
                from_rgba_into(&fmt, [r, g, b, 1.0], &mut enc);
                sky_data.extend_from_slice(&enc);
            }
            sky_surf.write_region(canvas, &sky_data);

            let sky_layer = Layer::new(format!("Sky: {}", sky_params.preset.name()), LayerContent::Raster(sky_surf));

            // Set mask on active layer to reveal the new sky beneath
            let mask_region = photocraft_algo::segment::Region {
                bbox: canvas,
                mask: inverted_mask,
            };
            let mask = crate::commands::mask_at_depth(
                LayerMask {
                    surface: photocraft_algo::matting::region_surface(&mask_region),
                    ..LayerMask::reveal_all()
                },
                doc.depth,
            )?;
            doc.layer_mut(active_id).ok_or(EngineError::NoLayer(active_id))?.mask = Some(mask);

            // Insert the replacement sky layer beneath the active layer
            let active_idx = doc.layers.iter().position(|l| l.id == active_id).unwrap_or(0);
            doc.layers.insert(active_idx, sky_layer);
        }

        doc.selection = None;
        Ok(())
    })?;

    Ok(json!({
        "success": true,
        "changed": true,
        "preset": sky_params.preset.name(),
        "bounds": [b.x0, b.y0, b.width() as i32, b.height() as i32],
    }))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec {
        id: CMD,
        label: "Sky Replacement…",
        menu: &["Edit"],
        shortcut: None,
        params: r##"{"preset":"blueSky|sunset|dramatic|night"="blueSky","shiftEdge":-100..100=0,"fadeEdge":0..100=10,"skyBrightness":-100..100=0,"skyTemperature":-100..100=0,"skyScale":100..300=100,"flip":bool=false,"foregroundLighting":0..100=20,"output":"newLayers|duplicateLayer|activeLayer"="newLayers"}"##,
        enabled: has_unlocked_layer,
        run,
        journal: true,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_returns_command() {
        let s = specs();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].id, CMD);
        assert_eq!(s[0].menu, &["Edit"]);
    }

    #[test]
    fn sky_replacement_new_layers_on_background_succeeds() {
        let mut s = Session::new();
        let w = 64;
        let h = 64;
        let mut doc = Document::new("Sky Test", photocraft_geom::Size::new(w, h), photocraft_color::ColorMode::Rgb, photocraft_color::SampleType::U8);
        let mut surf = Surface::new(doc.pixel_format());
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        let horizon = h / 2;
        for y in 0..h {
            for _x in 0..w {
                if y < horizon {
                    px.extend_from_slice(&[0.2, 0.5, 0.95, 1.0]);
                } else {
                    px.extend_from_slice(&[0.2, 0.2, 0.1, 1.0]);
                }
            }
        }
        surf.write_region(doc.bounds(), &px);
        let mut bg = Layer::new("Background", LayerContent::Raster(surf));
        bg.locks.transparency = true;
        bg.locks.position = true;
        doc.layers.push(bg);
        s.add_document(doc, None);

        let res = s.execute(CMD, json!({"output": "newLayers", "preset": "blueSky"}));
        assert!(res.is_ok(), "Sky replacement failed: {:?}", res);
        let d = s.active().unwrap();
        assert_eq!(d.doc.layers.len(), 2);
        assert!(d.doc.layers[0].name.starts_with("Sky:"));
        let fg_layer = &d.doc.layers[1];
        assert_eq!(fg_layer.name, "Layer 0");
        assert!(fg_layer.mask.is_some(), "Foreground layer must have a mask");
    }

    #[test]
    fn sky_replacement_duplicate_layer_succeeds() {
        let mut s = Session::new();
        let w = 64;
        let h = 64;
        let mut doc = Document::new("Sky Test", photocraft_geom::Size::new(w, h), photocraft_color::ColorMode::Rgb, photocraft_color::SampleType::U8);
        let mut surf = Surface::new(doc.pixel_format());
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for _x in 0..w {
                if y < h / 2 {
                    px.extend_from_slice(&[0.2, 0.5, 0.95, 1.0]);
                } else {
                    px.extend_from_slice(&[0.2, 0.2, 0.1, 1.0]);
                }
            }
        }
        surf.write_region(doc.bounds(), &px);
        let mut bg = Layer::new("Background", LayerContent::Raster(surf));
        bg.locks.transparency = true;
        bg.locks.position = true;
        doc.layers.push(bg);
        s.add_document(doc, None);

        let res = s.execute(CMD, json!({"output": "duplicateLayer", "preset": "sunset"}));
        assert!(res.is_ok(), "Duplicate layer sky replacement failed: {:?}", res);
        let d = s.active().unwrap();
        assert_eq!(d.doc.layers.len(), 2);
        assert_eq!(d.doc.layers[1].name, "Background Copy");
    }

    #[test]
    fn sky_replacement_active_layer_succeeds() {
        let mut s = Session::new();
        let w = 64;
        let h = 64;
        let mut doc = Document::new("Sky Test", photocraft_geom::Size::new(w, h), photocraft_color::ColorMode::Rgb, photocraft_color::SampleType::U8);
        let mut surf = Surface::new(doc.pixel_format());
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for _x in 0..w {
                if y < h / 2 {
                    px.extend_from_slice(&[0.2, 0.5, 0.95, 1.0]);
                } else {
                    px.extend_from_slice(&[0.2, 0.2, 0.1, 1.0]);
                }
            }
        }
        surf.write_region(doc.bounds(), &px);
        let mut bg = Layer::new("Background", LayerContent::Raster(surf));
        bg.locks.transparency = true;
        bg.locks.position = true;
        doc.layers.push(bg);
        s.add_document(doc, None);

        let res = s.execute(CMD, json!({"output": "activeLayer", "preset": "dramatic"}));
        assert!(res.is_ok(), "Active layer sky replacement failed: {:?}", res);
        let d = s.active().unwrap();
        assert_eq!(d.doc.layers.len(), 1);
    }
}
