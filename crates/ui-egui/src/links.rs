//! Community and project links: the Help menu, the About dialog, the start screen and the title
//! bar all open these. URLs follow the crafting-app pattern (`getartcraft.com/apps/<app>`,
//! `github.com/storytold/<app>`).

use serde_json::{Value, json};

use crate::PhotocraftApp;

pub const DISCORD: &str = "https://discord.gg/pixorastudio";
pub const APP_PAGE: &str = "https://pixora.amadersomaj.com";
pub const GITHUB: &str = "https://github.com/mohammad-sheikh-shahinur-rahman/Pixora-Studio-Professional-Photo-Editor";
pub const ISSUES: &str = "https://github.com/mohammad-sheikh-shahinur-rahman/Pixora-Studio-Professional-Photo-Editor/issues";

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`.
pub const COMMANDS: &[(&str, &str)] = &[("help.reportIssue", ISSUES)];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// One query string value, percent-encoded (RFC 3986 unreserved characters stay literal).
pub fn url_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// A new-issue URL on the project tracker, pre-filled with `title` and `body`. The reporting
/// mechanism the Microsoft Store requires of generative-AI features (policy 11.16).
pub fn report_url(title: &str, body: &str) -> String {
    format!("{ISSUES}/new?title={}&body={}", url_query(title), url_query(body))
}

/// Open a pre-filled issue reporting a generated (AI) result. `layer` is the Generative Fill
/// layer's id when the report comes from its context menu; the prompt it was made from is read
/// from the layer name (`edit.generativeFill` renames layers to "Generative Fill: {prompt}").
pub fn report_generated(app: &mut PhotocraftApp, ctx: &egui::Context, layer: Option<u64>) -> Value {
    let name = layer
        .and_then(|id| app.session.active().and_then(|s| s.doc.layer(photocraft_doc::LayerId(id))))
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "Generative Fill".into());
    let prompt = name.strip_prefix("Generative Fill: ").unwrap_or(&name);
    let title = format!("Report generated content: {prompt}");
    let body = format!(
        "Version: {version}\nGenerated-content prompt: {prompt}\n\nPlease describe what is inappropriate or wrong about this generated result, and attach a screenshot if you can.\n\n(Pixora Studio records the prompt the fill was made from but stores no image outside your document.)",
        version = photocraft_engine::build_info::long_version(),
    );
    open(app, ctx, &report_url(&title, &body))
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut PhotocraftApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// The prominent "Join us on Discord" button.
pub fn discord_button(app: &mut PhotocraftApp, ui: &mut egui::Ui, min_width: f32) -> egui::Response {
    let r = crate::widgets::primary_button(ui, tl!("Join us on Discord"), min_width).on_hover_text(DISCORD);
    if r.clicked() {
        open(app, ui.ctx(), DISCORD);
    }
    r
}

/// "PhotoCraft website · GitHub · ArtCraft" as links, centred. Clicks route through [`open`] (the
/// platform browser service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let links = [(tl!("Pixora Studio website"), APP_PAGE), (tl!("GitHub"), GITHUB)];
    let font = egui::FontId::proportional(12.5);
    let sep = "  ·  ";
    let width: f32 = links.iter().map(|(l, _)| ui.painter().layout_no_wrap((*l).into(), font.clone(), t.text).size().x).sum::<f32>()
        + 2.0 * ui.painter().layout_no_wrap(sep.into(), font.clone(), t.text).size().x;
    let mut clicked: Option<&str> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        for (i, (label, url)) in links.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new(sep).font(font.clone()).color(t.text_faint));
            }
            if ui.link(egui::RichText::new(*label).font(font.clone()).color(t.accent)).on_hover_text(*url).clicked() {
                clicked = Some(url);
            }
        }
    });
    if let Some(url) = clicked {
        open(app, ui.ctx(), url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links (Help menu, Discord button, start-page links) must open in the browser.
        // They route through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let ctx = egui::Context::default();
        // Every Help-menu link id reaches the service with its URL.
        for (id, url) in COMMANDS {
            crate::menus::invoke(&mut app, &ctx, id, json!({})).unwrap();
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(*url), "{id}");
        }
        // The direct open() helper (Discord button / start-page links) also uses it.
        open(&mut app, &ctx, DISCORD);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(DISCORD));
    }

    #[test]
    fn help_menu_lists_links_then_separator_then_system_info_and_about() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert_eq!(
            help,
            [
                "help.checkForUpdates",
                "help.reportIssue",
                "---",
                "help.systemInfo",
                "help.about"
            ]
        );
    }

    #[test]
    fn link_commands_open_their_urls() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let ctx = egui::Context::default();
        let (id, url) = ("help.reportIssue", ISSUES);
        let r = crate::menus::invoke(&mut app, &ctx, id, serde_json::json!({})).unwrap();
        assert_eq!(r["url"], url);
        assert_eq!(app.ui.status, format!("Opened {url}"));
    }

    #[test]
    fn query_values_are_percent_encoded() {
        assert_eq!(url_query("plain"), "plain");
        assert_eq!(url_query("a b&c=d/e?f"), "a%20b%26c%3Dd%2Fe%3Ff");
        assert_eq!(url_query("cli-ent.ien_t~1"), "cli-ent.ien_t~1");
        // Bengali and other non-ASCII texts survive as UTF-8, not as mojibake.
        assert!(url_query("ছবি").starts_with(&format!("%{:02X}%{:02X}", 0xE0, 0xA6)), "{}", url_query("ছবি"));
    }

    #[test]
    fn report_generated_opens_a_prefilled_issue() {
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let ctx = egui::Context::default();
        report_generated(&mut app, &ctx, None);
        let url = opened.lock().unwrap().last().cloned().unwrap_or_default();
        assert!(url.starts_with(ISSUES));
        assert!(url.contains("/new?title="), "{url}");
        assert!(url.contains(url_query("Generative Fill").as_str()), "{url}");
        assert!(!url.contains(' '), "{url}");
    }
}
