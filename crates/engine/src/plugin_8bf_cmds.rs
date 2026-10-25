//! `plugin.scan8bf`, `plugin.run8bf`, and `plugin.install8bf` commands.
//!
//! Provides support for classic Photoshop .8BF filter plugins via the Windows Host Bridge.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use photocraft_doc::LayerContent;
use photocraft_plugins::ps_8bf::{self, HostBridgeRequest, Ps8bfPlugin};
use serde_json::{Value, json};

use crate::commands::CommandSpec;
use crate::{EngineError, Result, Session};

static DISCOVERED_8BF: Mutex<Vec<Ps8bfPlugin>> = Mutex::new(Vec::new());

fn bad(cmd: &str, msg: impl Into<String>) -> EngineError {
    EngineError::BadParams { cmd: cmd.into(), msg: msg.into() }
}

/// Default plugin directory for Pixora Studio 8BF plugins.
pub fn default_8bf_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("Plug-ins").join("8BF")))
        .unwrap_or_else(|| PathBuf::from("Plug-ins").join("8BF"))
}

fn has_pixel_layer(s: &Session) -> std::result::Result<(), String> {
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

fn always(_: &Session) -> std::result::Result<(), String> {
    Ok(())
}

/// `plugin.scan8bf`: scan 8BF plugin folders and cache discovered filters.
fn scan_8bf(_s: &mut Session, p: &Value) -> Result<Value> {
    let mut dirs = vec![default_8bf_dir()];
    if let Some(custom) = p.get("dir").and_then(Value::as_str) {
        dirs.push(PathBuf::from(custom));
    }

    let mut found = Vec::new();
    for d in &dirs {
        found.extend(ps_8bf::scan_directory(d));
    }

    let list: Vec<Value> = found
        .iter()
        .map(|plug| {
            json!({
                "name": plug.name,
                "category": plug.category,
                "path": plug.path.to_string_lossy(),
                "is64Bit": plug.is_64bit,
                "entryPoint": plug.entry_point,
                "commandId": plug.command_id(),
            })
        })
        .collect();

    let count = found.len();
    if let Ok(mut cache) = DISCOVERED_8BF.lock() {
        *cache = found;
    }

    Ok(json!({ "plugins": list, "count": count }))
}

/// `plugin.run8bf`: executes an 8BF plugin on the active layer or selection.
fn run_8bf(s: &mut Session, p: &Value) -> Result<Value> {
    let path_str = p.get("path").and_then(Value::as_str).ok_or_else(|| bad("plugin.run8bf", "missing \"path\" parameter"))?;
    let entry_point = p.get("entryPoint").and_then(Value::as_str).unwrap_or("PluginMain").to_string();

    let d = s.active().ok_or(EngineError::NoDocument)?;
    let doc = d.doc.clone();
    let canvas = doc.bounds();
    let (w, h) = (canvas.width(), canvas.height());
    let id = d.active_layer.ok_or(EngineError::Other("no active layer".into()))?;

    let layer = doc.layer(id).ok_or(EngineError::NoLayer(id))?;
    let surf = layer.surface().ok_or(EngineError::Other("layer has no pixel surface".into()))?;

    let channels = surf.channels() as u32;
    let floats = surf.read_region(canvas);

    // Convert f32 to u8 bytes for the 8BF bridge
    let mut u8_bytes = Vec::with_capacity(floats.len());
    for f in floats {
        u8_bytes.push((f.clamp(0.0, 1.0) * 255.0).round() as u8);
    }

    let req = HostBridgeRequest {
        plugin_path: path_str.to_string(),
        entry_point,
        width: w,
        height: h,
        channels,
        rect: [canvas.x0, canvas.y0, canvas.x1, canvas.y1],
        pixels: u8_bytes,
    };

    let resp = ps_8bf::run_8bf_filter(req);
    if !resp.success {
        return Err(EngineError::Other(resp.message.unwrap_or_else(|| "8BF plugin execution failed".into())));
    }

    if let Some(out_bytes) = resp.output_pixels {
        s.edit("Run 8BF Filter", |doc, _| {
            let layer = doc.layer_mut(id).ok_or(EngineError::NoLayer(id))?;
            let surf = layer.surface_mut().ok_or(EngineError::Other("no surface".into()))?;

            let mut f32_pixels = Vec::with_capacity(out_bytes.len());
            for b in out_bytes {
                f32_pixels.push(f32::from(b) / 255.0);
            }
            surf.write_region(canvas, &f32_pixels);
            Ok(())
        })?;
    }

    Ok(json!({ "success": true, "plugin": path_str }))
}

/// `plugin.install8bf`: installs or copies an 8BF file into the local plugins directory.
fn install_8bf(s: &mut Session, p: &Value) -> Result<Value> {
    let src = p.get("path").and_then(Value::as_str).ok_or_else(|| bad("plugin.install8bf", "missing \"path\""))?;
    let src_path = Path::new(src);
    if !src_path.is_file() {
        return Err(bad("plugin.install8bf", "specified file does not exist"));
    }

    let target_dir = default_8bf_dir();
    let _ = std::fs::create_dir_all(&target_dir);

    let file_name = src_path.file_name().ok_or_else(|| bad("plugin.install8bf", "invalid file name"))?;
    let dest = target_dir.join(file_name);

    std::fs::copy(src_path, &dest).map_err(|e| EngineError::Other(format!("Failed to copy 8BF plugin: {e}")))?;

    // Re-scan
    let scan_params = json!({});
    scan_8bf(s, &scan_params)
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "plugin.scan8bf",
            label: "Scan 8BF Plug-ins",
            menu: &["Filter", "Plug-ins"],
            shortcut: None,
            params: r##"{"dir":string?} → {plugins:[{name,category,path,is64Bit,entryPoint,commandId},…],count}"##,
            enabled: always,
            run: scan_8bf,
            journal: false,
        },
        CommandSpec {
            id: "plugin.run8bf",
            label: "Run 8BF Plug-in…",
            menu: &["Filter", "Plug-ins"],
            shortcut: None,
            params: r##"{"path":string,"entryPoint":string="PluginMain"}"##,
            enabled: has_pixel_layer,
            run: run_8bf,
            journal: true,
        },
        CommandSpec {
            id: "plugin.install8bf",
            label: "Install 8BF Plug-in…",
            menu: &["Filter", "Plug-ins"],
            shortcut: None,
            params: r##"{"path":string}"##,
            enabled: always,
            run: install_8bf,
            journal: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_returns_commands() {
        let s = specs();
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].id, "plugin.scan8bf");
        assert_eq!(s[1].id, "plugin.run8bf");
        assert_eq!(s[2].id, "plugin.install8bf");
    }
}
