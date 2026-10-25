//! Built-in Photoshop & production scripts for File › Scripts:
//! - Export Layers to Files… (`file.scripts.exportLayersToFiles`)
//! - Layer Comps to Files… (`file.scripts.layerCompsToFiles`)
//! - Layer Comps to PDF… (`file.scripts.layerCompsToPdf`)
//! - Export Artboards to Files… (`file.scripts.exportArtboardsToFiles`)
//! - Export Artboards to PDF… (`file.scripts.exportArtboardsToPdf`)
//! - Fit Image… (`file.scripts.fitImage`)
//! - Batch Rename Layers… (`file.scripts.batchRenameLayers`)
//! - Delete All Empty Layers (`file.scripts.deleteAllEmptyLayers`)
//! - Delete All Hidden Layers (`file.scripts.deleteAllHiddenLayers`)
//! - Color Code Layers by Kind (`file.scripts.colorCodeLayers`)
//! - Export Social Media Assets… (`file.scripts.exportSocialPresets`)
//! - Split Image to Grid… (`file.scripts.splitToGrid`)
//! - Export Layer Specs (JSON)… (`file.scripts.exportLayoutJson`)
//! - Make Seamless Texture… (`file.scripts.makeSeamlessTexture`)
//! - Trim Document… (`file.scripts.trimDocument`)

use photocraft_color::{ColorMode, PixelFormat, SampleType};
use photocraft_doc::{Document, LabelColor, Layer, LayerContent, LayerId, Size};
use photocraft_raster::Surface;
use serde_json::{Value, json};

use crate::artboard_cmds::raster_pdf;
use crate::commands::CommandSpec;
use crate::comps_cmds::apply_comp;
use crate::file_cmds::{SaveOpts, encode, native_doc, str_param, write_file};
use crate::{EngineError, Result, Session};

fn bad(cmd: &str, msg: impl Into<String>) -> EngineError {
    EngineError::BadParams { cmd: cmd.into(), msg: msg.into() }
}

fn has_doc(s: &Session) -> std::result::Result<(), String> {
    s.active().map(|_| ()).ok_or_else(|| "no document open".into())
}

fn has_comps(s: &Session) -> std::result::Result<(), String> {
    let d = s.active().ok_or("no document open")?;
    if d.doc.layer_comps.is_empty() {
        Err("the document has no layer comps".into())
    } else {
        Ok(())
    }
}

fn has_artboards(s: &Session) -> std::result::Result<(), String> {
    let d = s.active().ok_or("no document open")?;
    if d.doc.has_artboards() {
        Ok(())
    } else {
        Err("the document has no artboards".into())
    }
}

// -------------------------------------------------------------------------------------------------
// Delegated scripts
// -------------------------------------------------------------------------------------------------

fn export_layers_to_files(s: &mut Session, p: &Value) -> Result<Value> {
    s.execute("file.export.layersToFiles", p.clone())
}

fn layer_comps_to_files(s: &mut Session, p: &Value) -> Result<Value> {
    s.execute("file.export.layerCompsToFiles", p.clone())
}

fn export_artboards_to_files(s: &mut Session, p: &Value) -> Result<Value> {
    s.execute("file.export.artboardsToFiles", p.clone())
}

fn export_artboards_to_pdf(s: &mut Session, p: &Value) -> Result<Value> {
    s.execute("file.export.artboardsToPdf", p.clone())
}

fn fit_image_script(s: &mut Session, p: &Value) -> Result<Value> {
    s.execute("file.automate.fitImage", p.clone())
}

fn trim_document_script(s: &mut Session, p: &Value) -> Result<Value> {
    s.execute("image.trim", p.clone())
}

// -------------------------------------------------------------------------------------------------
// Layer Comps to PDF
// -------------------------------------------------------------------------------------------------

fn layer_comps_to_pdf(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "file.scripts.layerCompsToPdf";
    let path = str_param(p, "path", cmd)?.to_string();
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = d.doc.clone();
    if doc.layer_comps.is_empty() {
        return Err(bad(cmd, "the document has no layer comps"));
    }

    let save = SaveOpts::from_params(s, p, cmd)?;
    let mut pages = Vec::new();
    for comp in &doc.layer_comps {
        let mut one = (*doc).clone();
        apply_comp(&mut one, comp, false);
        one.last_applied_comp = Some(comp.id);

        let buf = photocraft_compose::flatten(&one).over_background([1.0, 1.0, 1.0]);
        let fmt = PixelFormat::new(ColorMode::Rgb, SampleType::U8, true);
        let mut flat = Document::new(&comp.name, one.size, ColorMode::Rgb, SampleType::U8);
        let mut surf = Surface::new(fmt);
        let vals: Vec<f32> = buf.px.iter().flat_map(|px| photocraft_raster::from_rgba(&fmt, *px)).collect();
        surf.write_region(one.bounds(), &vals);
        flat.layers.push(Layer::new("Background", LayerContent::Raster(surf)));

        let (jpeg, _) = encode(&flat, "page.jpg", save)?;
        pages.push((one.size.width, one.size.height, doc.resolution_dpi, jpeg));
    }

    if pages.is_empty() {
        return Err(bad(cmd, "no layer comps could be exported"));
    }
    write_file(&path, &raster_pdf(&pages))?;
    Ok(json!({"path": path, "pages": pages.len()}))
}

// -------------------------------------------------------------------------------------------------
// Batch Rename Layers
// -------------------------------------------------------------------------------------------------

fn batch_rename_layers(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "file.scripts.batchRenameLayers";
    let find = p.get("find").and_then(Value::as_str);
    let replace = p.get("replace").and_then(Value::as_str).unwrap_or("");
    let prefix = p.get("prefix").and_then(Value::as_str).unwrap_or("");
    let suffix = p.get("suffix").and_then(Value::as_str).unwrap_or("");
    let numbering = p.get("numbering").and_then(Value::as_bool).unwrap_or(false);
    let start = p.get("start").and_then(Value::as_u64).unwrap_or(1);
    let digits = (p.get("digits").and_then(Value::as_u64).unwrap_or(2) as usize).clamp(1, 8);
    let selected_only = p.get("selectedOnly").and_then(Value::as_bool).unwrap_or(false);

    let d = s.active().ok_or(EngineError::NoDocument)?;
    let target_ids: Vec<LayerId> = if selected_only && !d.selected_layers.is_empty() {
        d.selected_layers.clone()
    } else {
        d.doc.layers.iter().map(|l| l.id).collect()
    };

    if target_ids.is_empty() {
        return Err(bad(cmd, "no layers to rename"));
    }

    let mut renamed_count = 0;
    s.edit("Batch Rename Layers", |doc, _| {
        let mut idx = start;
        for id in &target_ids {
            if let Some(l) = doc.layer_mut(*id) {
                let mut name = l.name.clone();
                if let Some(f) = find {
                    if !f.is_empty() {
                        name = name.replace(f, replace);
                    }
                }
                if numbering {
                    let num_str = format!("{:0width$}", idx, width = digits);
                    if name.contains("{n}") {
                        name = name.replace("{n}", &num_str);
                    } else if name.contains("{index}") {
                        name = name.replace("{index}", &num_str);
                    } else {
                        name = format!("{name}_{num_str}");
                    }
                    idx = idx.saturating_add(1);
                }
                l.name = format!("{prefix}{name}{suffix}");
                renamed_count += 1;
            }
        }
        Ok(())
    })?;

    Ok(json!({"renamed": renamed_count}))
}

// -------------------------------------------------------------------------------------------------
// Delete All Hidden Layers
// -------------------------------------------------------------------------------------------------

fn delete_all_hidden_layers(s: &mut Session, _p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let hidden_ids: Vec<LayerId> = d.doc.layers.iter().filter(|l| !l.visible).map(|l| l.id).collect();
    if hidden_ids.is_empty() {
        return Ok(json!({"deleted": 0}));
    }

    let deleted = hidden_ids.len();
    s.edit("Delete All Hidden Layers", |doc, _| {
        doc.layers.retain(|l| l.visible);
        Ok(())
    })?;

    if let Some(d) = s.active_mut() {
        d.selected_layers.retain(|id| d.doc.layer(*id).is_some());
        if d.active_layer.is_some_and(|id| d.doc.layer(id).is_none()) {
            d.active_layer = d.doc.layers.last().map(|l| l.id);
        }
    }

    Ok(json!({"deleted": deleted}))
}

// -------------------------------------------------------------------------------------------------
// Color Code Layers by Kind
// -------------------------------------------------------------------------------------------------

/// Auto-labels layer colors by their kind to keep document layers organized:
/// Type -> Yellow, Vector/Shape -> Orange, Adjustments -> Cyan, Smart Objects -> Violet, Raster -> Blue, Group -> Red.
fn color_code_layers(s: &mut Session, _p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    if d.doc.layers.is_empty() {
        return Ok(json!({"colorCoded": 0}));
    }

    let mut count = 0;
    s.edit("Color Code Layers", |doc, _| {
        for l in &mut doc.layers {
            let color = match &l.content {
                LayerContent::Text(_) => LabelColor::Yellow,
                LayerContent::Shape(_) => LabelColor::Orange,
                LayerContent::Adjustment(_) => LabelColor::Seafoam,
                LayerContent::Smart(_) => LabelColor::Violet,
                LayerContent::Raster(_) => LabelColor::Blue,
                LayerContent::Group(_) => LabelColor::Red,
                _ => LabelColor::None,
            };
            l.label = color;
            count += 1;
        }
        Ok(())
    })?;

    Ok(json!({"colorCoded": count}))
}

// -------------------------------------------------------------------------------------------------
// Export Social Media Presets
// -------------------------------------------------------------------------------------------------

/// Export current design into standard social media image sizes (Instagram, Facebook, YouTube, Twitter/X).
fn export_social_presets(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "file.scripts.exportSocialPresets";
    let dir = str_param(p, "dir", cmd).or_else(|_| str_param(p, "output", cmd))?.to_string();
    let format = p.get("format").and_then(Value::as_str).unwrap_or("jpg").trim_start_matches('.').to_ascii_lowercase();
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc_orig = d.doc.clone();

    let social_targets = [
        ("Instagram_Square_1080x1080", 1080, 1080),
        ("Instagram_Story_1080x1920", 1080, 1920),
        ("Facebook_Post_1200x630", 1200, 630),
        ("YouTube_Thumb_1280x720", 1280, 720),
        ("Twitter_Post_1200x675", 1200, 675),
    ];

    let save = SaveOpts::from_params(s, p, cmd)?;
    let mut files = Vec::new();

    for (name, target_w, target_h) in &social_targets {
        let buf = photocraft_compose::flatten(&doc_orig).over_background([1.0, 1.0, 1.0]);
        let orig_w = doc_orig.size.width;
        let orig_h = doc_orig.size.height;

        let scale = (f64::from(*target_w) / f64::from(orig_w.max(1))).min(f64::from(*target_h) / f64::from(orig_h.max(1)));
        let new_w = ((f64::from(orig_w) * scale).round() as u32).max(1);
        let new_h = ((f64::from(orig_h) * scale).round() as u32).max(1);

        let fmt = PixelFormat::new(ColorMode::Rgb, SampleType::U8, true);
        let mut scaled_surf = Surface::new(fmt);
        for y in 0..new_h {
            let src_y = ((f64::from(y) / scale).floor() as u32).min(orig_h.saturating_sub(1));
            for x in 0..new_w {
                let src_x = ((f64::from(x) / scale).floor() as u32).min(orig_w.saturating_sub(1));
                let idx = (src_y * orig_w + src_x) as usize;
                let px = buf.px.get(idx).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
                let vals = photocraft_raster::from_rgba(&fmt, px);
                scaled_surf.write_pixel(x as i32, y as i32, &vals);
            }
        }

        let mut out_doc = Document::new(*name, Size::new(new_w, new_h), ColorMode::Rgb, SampleType::U8);
        out_doc.layers.push(Layer::new("Background", LayerContent::Raster(scaled_surf)));

        let out_path = format!("{dir}/{name}.{format}");
        crate::file_cmds::save_doc(&out_doc, &out_path, save)?;
        files.push(out_path);
    }

    Ok(json!({"files": files, "count": files.len()}))
}

// -------------------------------------------------------------------------------------------------
// Split Image to Grid (Tileset / Instagram Carousel)
// -------------------------------------------------------------------------------------------------

/// Slices the document into rows × cols tiles and exports each numbered tile.
fn split_to_grid(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "file.scripts.splitToGrid";
    let dir = str_param(p, "dir", cmd).or_else(|_| str_param(p, "output", cmd))?.to_string();
    let rows = (p.get("rows").and_then(Value::as_u64).unwrap_or(3) as u32).clamp(1, 20);
    let cols = (p.get("cols").and_then(Value::as_u64).unwrap_or(3) as u32).clamp(1, 20);
    let format = p.get("format").and_then(Value::as_str).unwrap_or("png").trim_start_matches('.').to_ascii_lowercase();

    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = d.doc.clone();
    let w = doc.size.width;
    let h = doc.size.height;
    let tile_w = (w / cols).max(1);
    let tile_h = (h / rows).max(1);

    let buf = photocraft_compose::flatten(&doc).over_background([1.0, 1.0, 1.0]);
    let fmt = PixelFormat::new(ColorMode::Rgb, SampleType::U8, true);
    let save = SaveOpts::from_params(s, p, cmd)?;
    let mut files = Vec::new();

    for r in 0..rows {
        for c in 0..cols {
            let start_x = c * tile_w;
            let start_y = r * tile_h;
            let mut surf = Surface::new(fmt);
            for ty in 0..tile_h {
                let y = (start_y + ty).min(h.saturating_sub(1));
                for tx in 0..tile_w {
                    let x = (start_x + tx).min(w.saturating_sub(1));
                    let idx = (y * w + x) as usize;
                    let px = buf.px.get(idx).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    let vals = photocraft_raster::from_rgba(&fmt, px);
                    surf.write_pixel(tx as i32, ty as i32, &vals);
                }
            }
            let tile_name = format!("tile_r{}_c{}", r + 1, c + 1);
            let mut tile_doc = Document::new(&tile_name, Size::new(tile_w, tile_h), ColorMode::Rgb, SampleType::U8);
            tile_doc.layers.push(Layer::new("Background", LayerContent::Raster(surf)));

            let out_path = format!("{dir}/{tile_name}.{format}");
            crate::file_cmds::save_doc(&tile_doc, &out_path, save)?;
            files.push(out_path);
        }
    }

    Ok(json!({"files": files, "tiles": files.len(), "rows": rows, "cols": cols}))
}

// -------------------------------------------------------------------------------------------------
// Export Layer Specs (JSON)
// -------------------------------------------------------------------------------------------------

/// Export layer coordinates, dimensions, opacity, and blend modes for frontend/UI developers.
fn export_layout_json(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "file.scripts.exportLayoutJson";
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = &d.doc;

    let layers_data: Vec<Value> = doc.layers.iter().map(|l| {
        let bounds = crate::layer_multi_cmds::layer_bounds(l).unwrap_or_else(|| doc.bounds());
        json!({
            "id": l.id.0,
            "name": l.name,
            "kind": l.content.kind_name(),
            "visible": l.visible,
            "opacity": l.opacity,
            "blendMode": format!("{:?}", l.blend),
            "x": bounds.x0,
            "y": bounds.y0,
            "width": bounds.width(),
            "height": bounds.height(),
        })
    }).collect();

    let output = json!({
        "document": doc.name,
        "width": doc.size.width,
        "height": doc.size.height,
        "resolutionDpi": doc.resolution_dpi,
        "layersCount": doc.layers.len(),
        "layers": layers_data,
    });

    if let Some(path) = p.get("path").and_then(Value::as_str) {
        let text = serde_json::to_string_pretty(&output).map_err(|e| bad(cmd, format!("JSON error: {e}")))?;
        write_file(path, text.as_bytes())?;
    }

    Ok(output)
}

// -------------------------------------------------------------------------------------------------
// Make Seamless Texture
// -------------------------------------------------------------------------------------------------

fn make_seamless_texture(s: &mut Session, p: &Value) -> Result<Value> {
    let d = s.active().ok_or(EngineError::NoDocument)?;
    let w = d.doc.size.width;
    let h = d.doc.size.height;
    if w == 0 || h == 0 {
        return Err(bad("file.scripts.makeSeamlessTexture", "empty document"));
    }

    let half_w = (w / 2) as i32;
    let half_h = (h / 2) as i32;

    s.execute(
        "filter.other.offset",
        json!({
            "horizontal": half_w,
            "vertical": half_h,
            "undefinedAreas": "wrap"
        }),
    )?;

    let blend = (p.get("blendWidth").and_then(Value::as_u64).unwrap_or(24) as u32).clamp(2, w.min(h) / 4);
    let active_id = s.active().and_then(|d| d.active_layer).ok_or_else(|| bad("file.scripts.makeSeamlessTexture", "no active layer"))?;

    s.edit("Make Seamless Texture", |doc, _| {
        let Some(layer) = doc.layer_mut(active_id) else { return Ok(()) };
        let LayerContent::Raster(surf) = &mut layer.content else { return Ok(()) };

        let bounds = surf.content_bounds();
        let center_x = bounds.x0 + (bounds.width() as i32 / 2);
        let center_y = bounds.y0 + (bounds.height() as i32 / 2);
        let b = blend as i32;

        let channels = surf.channels();
        let mut p1 = vec![0.0f32; channels];
        let mut p2 = vec![0.0f32; channels];
        let mut cur = vec![0.0f32; channels];

        // Blend horizontal strip across center_y
        for y in (center_y - b)..=(center_y + b) {
            let t = ((y - (center_y - b)) as f32 / (2 * b) as f32).clamp(0.0, 1.0);
            let weight = 0.5 * (1.0 - (2.0 * t - 1.0).abs());
            for x in bounds.x0..bounds.x1 {
                let y1 = (y - b).max(bounds.y0);
                let y2 = (y + b).min(bounds.y1 - 1);
                surf.read_pixel(x, y1, &mut p1);
                surf.read_pixel(x, y2, &mut p2);
                surf.read_pixel(x, y, &mut cur);
                for c in 0..channels {
                    let avg = 0.5 * (p1[c] + p2[c]);
                    cur[c] = cur[c] * (1.0 - weight) + avg * weight;
                }
                surf.write_pixel(x, y, &cur);
            }
        }

        // Blend vertical strip across center_x
        for x in (center_x - b)..=(center_x + b) {
            let t = ((x - (center_x - b)) as f32 / (2 * b) as f32).clamp(0.0, 1.0);
            let weight = 0.5 * (1.0 - (2.0 * t - 1.0).abs());
            for y in bounds.y0..bounds.y1 {
                let x1 = (x - b).max(bounds.x0);
                let x2 = (x + b).min(bounds.x1 - 1);
                surf.read_pixel(x1, y, &mut p1);
                surf.read_pixel(x2, y, &mut p2);
                surf.read_pixel(x, y, &mut cur);
                for c in 0..channels {
                    let avg = 0.5 * (p1[c] + p2[c]);
                    cur[c] = cur[c] * (1.0 - weight) + avg * weight;
                }
                surf.write_pixel(x, y, &cur);
            }
        }
        Ok(())
    })?;

    Ok(json!({"seamless": true, "blendWidth": blend}))
}

// -------------------------------------------------------------------------------------------------
// Specs Registration
// -------------------------------------------------------------------------------------------------

pub fn specs() -> Vec<CommandSpec> {
    macro_rules! spec {
        ($id:expr, $label:expr, $menu:expr, $params:expr, $enabled:expr, $run:expr) => {
            CommandSpec { id: $id, label: $label, menu: $menu, shortcut: None, params: $params, enabled: $enabled, journal: true, run: $run }
        };
    }
    vec![
        spec!(
            "file.scripts.exportLayersToFiles",
            "Export Layers to Files…",
            &["File", "Scripts"],
            r##"{"dir":folder,"format":"png|jpg|psd|tiff|…"="png","prefix":str=document name,"visibleOnly":bool=true,"quality":0..12?} → {files}"##,
            native_doc,
            export_layers_to_files
        ),
        spec!(
            "file.scripts.layerCompsToFiles",
            "Layer Comps to Files…",
            &["File", "Scripts"],
            r##"{"dir":folder,"format":"png|jpg|psd|tiff|…"="png","prefix":str=document name,"selectedOnly":bool=false,"comps":[id]?,"quality":0..12?} → {files}"##,
            has_comps,
            layer_comps_to_files
        ),
        spec!(
            "file.scripts.layerCompsToPdf",
            "Layer Comps to PDF…",
            &["File", "Scripts"],
            r##"{"path":str,"quality":0..12=10} → {path, pages}"##,
            has_comps,
            layer_comps_to_pdf
        ),
        spec!(
            "file.scripts.exportArtboardsToFiles",
            "Export Artboards to Files…",
            &["File", "Scripts"],
            r##"{"dir":folder,"format":"png|jpg|psd|tiff|…"="png","prefix":str=document name,"artboards":[id]?} → {files}"##,
            has_artboards,
            export_artboards_to_files
        ),
        spec!(
            "file.scripts.exportArtboardsToPdf",
            "Export Artboards to PDF…",
            &["File", "Scripts"],
            r##"{"path":str,"quality":0..12=10,"artboards":[id]?} → {path, pages}"##,
            has_artboards,
            export_artboards_to_pdf
        ),
        spec!(
            "file.scripts.fitImage",
            "Fit Image…",
            &["File", "Scripts"],
            r##"{"width":px,"height":px,"dontEnlarge":bool=false} → scales document proportionally"##,
            has_doc,
            fit_image_script
        ),
        spec!(
            "file.scripts.batchRenameLayers",
            "Batch Rename Layers…",
            &["File", "Scripts"],
            r##"{"find":str?,"replace":str="","prefix":str="","suffix":str="","numbering":bool=false,"start":integer=1,"digits":1..8=2,"selectedOnly":bool=false} → {renamed}"##,
            has_doc,
            batch_rename_layers
        ),
        spec!(
            "file.scripts.deleteAllHiddenLayers",
            "Delete All Hidden Layers",
            &["File", "Scripts"],
            "{} → deletes non-visible layers",
            has_doc,
            delete_all_hidden_layers
        ),
        spec!(
            "file.scripts.colorCodeLayers",
            "Color Code Layers by Kind",
            &["File", "Scripts"],
            "{} → assigns colors based on layer kind (Type=Yellow, Shape=Orange, Adjust=Cyan, Smart=Violet, Raster=Blue, Group=Red)",
            has_doc,
            color_code_layers
        ),
        spec!(
            "file.scripts.exportSocialPresets",
            "Export Social Media Assets…",
            &["File", "Scripts"],
            r##"{"dir":folder,"format":"jpg|png"="jpg"} → exports Instagram (Square & Story), Facebook, YouTube & Twitter"##,
            has_doc,
            export_social_presets
        ),
        spec!(
            "file.scripts.splitToGrid",
            "Split Image to Grid…",
            &["File", "Scripts"],
            r##"{"dir":folder,"rows":1..20=3,"cols":1..20=3,"format":"png|jpg"="png"} → splits into numbered grid tiles"##,
            has_doc,
            split_to_grid
        ),
        spec!(
            "file.scripts.exportLayoutJson",
            "Export Layer Specs (JSON)…",
            &["File", "Scripts"],
            r##"{"path":str?} → returns structured JSON with coordinates, dimensions and blend modes for developers"##,
            has_doc,
            export_layout_json
        ),
        spec!(
            "file.scripts.trimDocument",
            "Trim Document…",
            &["File", "Scripts"],
            r##"{"basedOn":"transparent|topLeft|bottomRight"="transparent","top":bool=true,"bottom":bool=true,"left":bool=true,"right":bool=true} → trims canvas"##,
            has_doc,
            trim_document_script
        ),
        spec!(
            "file.scripts.makeSeamlessTexture",
            "Make Seamless Texture…",
            &["File", "Scripts"],
            r##"{"blendWidth":2..256=24} → offsets 50% and blends cross seams for repeating texture tile"##,
            has_doc,
            make_seamless_texture
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use photocraft_color::{ColorMode, PixelFormat, SampleType};
    use photocraft_doc::{Document, LabelColor, Layer, LayerContent, Size};
    use photocraft_raster::Surface;

    fn test_doc() -> Document {
        let mut doc = Document::new("Test", Size::new(100, 100), ColorMode::Rgb, SampleType::U8);
        let fmt = PixelFormat::new(ColorMode::Rgb, SampleType::U8, true);
        let surf = Surface::new(fmt);
        doc.layers.push(Layer::new("Layer 1", LayerContent::Raster(surf.clone())));
        doc.layers.push(Layer::new("Layer 2", LayerContent::Raster(surf)));
        doc
    }

    #[test]
    fn batch_rename_replaces_and_numbers() {
        let mut s = Session::new();
        s.open_document(test_doc(), None);
        let r = s.execute("file.scripts.batchRenameLayers", json!({"find": "Layer", "replace": "Sprite", "numbering": true, "start": 1, "digits": 2})).unwrap();
        assert_eq!(r["renamed"], 2);
        let d = s.active().unwrap();
        assert_eq!(d.doc.layers[0].name, "Sprite 1_01");
        assert_eq!(d.doc.layers[1].name, "Sprite 2_02");
    }

    #[test]
    fn delete_all_hidden_layers_removes_invisible() {
        let mut s = Session::new();
        let mut doc = test_doc();
        doc.layers[0].visible = false;
        s.open_document(doc, None);
        let r = s.execute("file.scripts.deleteAllHiddenLayers", json!({})).unwrap();
        assert_eq!(r["deleted"], 1);
        assert_eq!(s.active().unwrap().doc.layers.len(), 1);
        assert_eq!(s.active().unwrap().doc.layers[0].name, "Layer 2");
    }

    #[test]
    fn fit_image_script_resizes_proportional() {
        let mut s = Session::new();
        s.open_document(test_doc(), None);
        s.execute("file.scripts.fitImage", json!({"width": 50, "height": 50})).unwrap();
        assert_eq!(s.active().unwrap().doc.size.width, 50);
        assert_eq!(s.active().unwrap().doc.size.height, 50);
    }

    #[test]
    fn color_code_layers_assigns_colors() {
        let mut s = Session::new();
        s.open_document(test_doc(), None);
        let r = s.execute("file.scripts.colorCodeLayers", json!({})).unwrap();
        assert_eq!(r["colorCoded"], 2);
        assert_eq!(s.active().unwrap().doc.layers[0].label, LabelColor::Blue);
    }

    #[test]
    fn export_layout_json_returns_specs() {
        let mut s = Session::new();
        s.open_document(test_doc(), None);
        let r = s.execute("file.scripts.exportLayoutJson", json!({})).unwrap();
        assert_eq!(r["layersCount"], 2);
        assert_eq!(r["width"], 100);
    }
}
