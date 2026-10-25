//! Photoshop 2024–2026 Contextual Task Bar: a modern, floating, draggable on-canvas HUD
//! that adapts dynamically to the current workflow (Selection, Subject/Background removal,
//! Generative Fill, Type).

use egui::{vec2, Color32, FontId, Id, Pos2, Rect, Sense, Stroke};
use photocraft_doc::LayerContent;
use serde_json::json;

use crate::PhotocraftApp;
use crate::theme::Tokens;
use crate::widgets;

#[derive(Clone, Debug, Default)]
struct BarState {
    prompt_open: bool,
    prompt_text: String,
    pinned: bool,
}

/// Renders the floating Contextual Task Bar over the active document canvas.
pub fn show(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    if !app.ui.panels.contextual_taskbar {
        return;
    }
    if app.session.active().is_none() {
        return;
    }

    let t = Tokens::get(ui.ctx());
    let ctx = ui.ctx().clone();
    let state_id = Id::new("contextual-taskbar-state");
    let mut state = ctx.data_mut(|d| d.get_temp::<BarState>(state_id).unwrap_or_default());

    let has_selection = app.session.active().is_some_and(|d| {
        d.doc.selection.as_ref().is_some_and(|m| !m.content_bounds().is_empty())
    });

    let active_id = app.session.active().and_then(|d| d.active_layer);
    let is_text = active_id.and_then(|id| app.session.active()?.doc.layer(id)).is_some_and(|l| {
        matches!(l.content, LayerContent::Text(_))
    }) || app.ui.tool.is_type();

    let avail = ui.available_rect_before_wrap();
    let bar_height = 38.0;
    let bar_width = if state.prompt_open {
        440.0
    } else if has_selection {
        480.0
    } else if is_text {
        320.0
    } else {
        390.0
    };

    let default_pos = Pos2::new(
        (avail.center().x - bar_width * 0.5).max(avail.left() + 20.0),
        (avail.bottom() - bar_height - 35.0).max(avail.top() + 50.0),
    );

    let current_pos = app.ui.panels.contextual_taskbar_pos.map(|[x, y]| Pos2::new(x, y)).unwrap_or(default_pos);

    let area = egui::Area::new(Id::new("contextual-taskbar-area"))
        .fixed_pos(current_pos)
        .order(egui::Order::Foreground);

    let mut close_prompt = false;
    let mut submit_prompt = false;
    let mut toggle_prompt = false;
    let mut hide_bar = false;
    let mut reset_pos = false;

    area.show(&ctx, |ui| {
        let frame_rect = Rect::from_min_size(ui.cursor().min, vec2(bar_width, bar_height));
        let painter = ui.painter();

        // Dark translucent capsule
        let bg = Color32::from_rgba_unmultiplied(28, 30, 34, 245);
        painter.rect_filled(frame_rect, 7.0, bg);
        painter.rect_stroke(frame_rect, 7.0, Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30)), egui::StrokeKind::Inside);

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(frame_rect.shrink(4.0)));
        child.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // Drag handle (6-dot icon)
            let drag_handle = ui.add(egui::Label::new(
                egui::RichText::new("⠿").color(t.text_faint).font(FontId::proportional(14.0))
            ).sense(Sense::drag()));

            if drag_handle.dragged() && !state.pinned {
                let new_pos = current_pos + drag_handle.drag_delta();
                app.ui.panels.contextual_taskbar_pos = Some([new_pos.x, new_pos.y]);
            }

            ui.separator();

            if state.prompt_open {
                // Inline Generative Fill Prompt input
                let text_edit = egui::TextEdit::singleline(&mut state.prompt_text)
                    .hint_text("Describe what to generate, or leave empty…")
                    .desired_width(260.0);
                let resp = ui.add(text_edit);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    submit_prompt = true;
                }

                if widgets::primary_button(ui, "Generate", 76.0).clicked() {
                    submit_prompt = true;
                }
                if widgets::secondary_button(ui, "Cancel", 56.0).clicked() {
                    close_prompt = true;
                }
            } else if has_selection {
                // Selection Context
                if widgets::primary_button(ui, "🪄 Generative Fill", 116.0).clicked() {
                    toggle_prompt = true;
                }
                if widgets::secondary_button(ui, "Select & Mask", 94.0).clicked() {
                    let _ = app.run("select.selectAndMask", json!({}));
                }
                if widgets::secondary_button(ui, "Refine Hair", 78.0).clicked() {
                    let _ = app.run("select.refineHair", json!({}));
                }
                if widgets::secondary_button(ui, "Invert", 52.0).clicked() {
                    let _ = app.run("select.inverse", json!({}));
                }
                if widgets::secondary_button(ui, "Deselect", 64.0).clicked() {
                    let _ = app.run("select.deselect", json!({}));
                }
            } else if is_text {
                // Type Context
                ui.label(egui::RichText::new("Text Layer").size(12.0).color(t.text));
                if widgets::primary_button(ui, "Commit ✔", 75.0).clicked() {
                    let _ = app.run("type.commit", json!({}));
                }
                if widgets::secondary_button(ui, "Cancel ✖", 70.0).clicked() {
                    let _ = app.run("type.cancel", json!({}));
                }
            } else {
                // Default Photo Context
                if widgets::secondary_button(ui, "🔍 Select Subject", 112.0).clicked() {
                    let _ = app.run("select.subject", json!({}));
                }
                if widgets::secondary_button(ui, "✂ Remove Background", 140.0).clicked() {
                    let _ = app.run("layer.removeBackground", json!({}));
                }
                if widgets::secondary_button(ui, "🪄 Generative Fill…", 116.0).clicked() {
                    toggle_prompt = true;
                }
            }

            ui.separator();

            // Overflow menu
            ui.menu_button("⋯", |ui| {
                if ui.checkbox(&mut state.pinned, "Pin Bar Position").clicked() {
                    ui.close();
                }
                if ui.button("Reset Position").clicked() {
                    reset_pos = true;
                    ui.close();
                }
                if ui.button("Hide Task Bar").clicked() {
                    hide_bar = true;
                    ui.close();
                }
            });
        });
    });

    if reset_pos {
        app.ui.panels.contextual_taskbar_pos = None;
    }
    if hide_bar {
        app.ui.panels.contextual_taskbar = false;
    }
    if toggle_prompt {
        state.prompt_open = true;
    }
    if close_prompt {
        state.prompt_open = false;
    }
    if submit_prompt {
        state.prompt_open = false;
        let prompt_clone = state.prompt_text.clone();
        state.prompt_text.clear();
        let _ = app.run("edit.generativeFill", json!({
            "prompt": prompt_clone,
            "output": "new"
        }));
    }

    ctx.data_mut(|d| d.insert_temp(state_id, state));
}
