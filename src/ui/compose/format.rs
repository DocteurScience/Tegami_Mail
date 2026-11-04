//! The composing formats (rich text and plain text), switching between
//! them, and the body's zoom.

use super::messages::ComposeInput;
use super::Compose;
use crate::config::ComposeFormat;
use crate::i18n::i18n;
use crate::ui::context_menu::MenuEntry;
use adw::prelude::*;
use relm4::prelude::*;

/// What a format is called where the user meets it.
pub(super) fn format_label(format: ComposeFormat) -> String {
    match format {
        ComposeFormat::Rich => i18n("Rich text"),
        ComposeFormat::Plain => i18n("Plain text"),
    }
}

/// The one-line explanation under each format in the menu.
fn format_hint(format: ComposeFormat) -> String {
    match format {
        ComposeFormat::Rich => i18n("Format as you type; it is sent as HTML"),
        ComposeFormat::Plain => i18n("No formatting at all"),
    }
}

/// The zoom of the body, in percent: the session's, so the next composer
/// opens at the size the last one was left at. 0 until the first change,
/// which reads as Settings → Composing → Default zoom, itself the reader's
/// unless set: a message written at one size and read at another looked
/// shrunk in the writing (#535).
static ZOOM: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The body's zoom now; see [`ZOOM`].
pub(super) fn session_zoom() -> u32 {
    match ZOOM.load(std::sync::atomic::Ordering::Relaxed) {
        0 => crate::config::load_compose_zoom(),
        z => z,
    }
}

/// Back to the default from Settings for the next composer.
pub(crate) fn forget_zoom() {
    ZOOM.store(0, std::sync::atomic::Ordering::Relaxed);
}

/// The body's zoom as a chip at the start of the format bar, shown only
/// away from 100%: in rich text the size picker names the size that is
/// sent, and the chip is what says the text on screen is not at it. A
/// click goes back to 100%, as Ctrl+0 does.
pub(super) fn zoom_chip(sender: &ComponentSender<Compose>) -> gtk::Label {
    let chip = gtk::Label::new(None);
    chip.set_valign(gtk::Align::Center);
    chip.add_css_class("caption");
    chip.add_css_class("tegami-zoom-chip");
    chip.set_cursor_from_name(Some("pointer"));
    chip.set_tooltip_text(Some(
        i18n("Zoom of the text on screen, not of the message sent. A click here, or Ctrl+0, puts it back to 100%.")
            .as_str(),
    ));
    let click = gtk::GestureClick::new();
    let s = sender.input_sender().clone();
    click.connect_released(move |_, _, _, _| {
        let _ = s.send(ComposeInput::Zoom(0));
    });
    chip.add_controller(click);
    show_zoom(&chip, session_zoom());
    chip
}

pub(super) fn show_zoom(chip: &gtk::Label, zoom: u32) {
    chip.set_label(&format!("{zoom}%"));
    chip.set_visible(zoom != 100);
}

/// What TEGAMI_SHOWCASE_PLAIN_LOG reports about the body (demo only).
pub(super) const SHOWCASE_PLAIN_LOG: &str = r#"JSON.stringify({plain:!!window.__tegamiPlain,
font:document.body?getComputedStyle(document.body).fontFamily:'',
bold:document.body&&document.body.querySelector('b,strong')?getComputedStyle(document.body.querySelector('b,strong')).fontWeight:'',
html:document.body?document.body.innerHTML:document.documentElement.outerHTML.slice(0,400),
text:window.__tegamiBodyText?window.__tegamiBodyText():''})"#;

impl Compose {
    /// The More actions menu's Format section (docs/DESIGN.md, "Menus";
    /// mockup 06): its title, then the formats, the current one marked.
    /// Each format's one-line explanation is its tooltip.
    pub(super) fn format_entries(&self, sender: &ComponentSender<Self>) -> Vec<MenuEntry> {
        let formats = [ComposeFormat::Rich, ComposeFormat::Plain]
            .into_iter()
            .map(|format| {
                let s = sender.clone();
                MenuEntry::new(format_label(format), move || {
                    s.input_sender().emit(ComposeInput::SetFormat(format))
                })
                .tooltip(format_hint(format))
                .selected(format == self.format)
            });
        std::iter::once(MenuEntry::heading(i18n("Format")))
            .chain(formats)
            .collect()
    }

    /// Handles [`ComposeInput::Zoom`].
    pub(super) fn on_zoom(&mut self, step: i8) {
        use crate::config::READER_ZOOM_STEPS as STEPS;
        let now = session_zoom();
        let at = STEPS.iter().position(|&z| z == now).unwrap_or(5) as i32;
        let zoom = match step {
            0 => 100,
            s => STEPS[(at + i32::from(s.signum())).clamp(0, STEPS.len() as i32 - 1) as usize],
        };
        ZOOM.store(zoom, std::sync::atomic::Ordering::Relaxed);
        self.editor.set_zoom(zoom);
        show_zoom(&self.zoom_chip, zoom);
    }

    /// Handles [`ComposeInput::SetFormat`].
    pub(super) fn on_set_format(&mut self, to: ComposeFormat, sender: &ComponentSender<Self>) {
        if to == self.format {
            return;
        }
        // Plain text shows only what is sent (#424), so moving a
        // formatted body there drops what it cannot show, and the
        // way back does not bring it back.
        if self.format == ComposeFormat::Rich && to == ComposeFormat::Plain {
            let s = sender.clone();
            self.editor.eval(
                "String(!!(window.__tegamiPlainLoses&&window.__tegamiPlainLoses()))",
                move |loses| {
                    s.input_sender().emit(if loses == "true" {
                        ComposeInput::AskPlain
                    } else {
                        ComposeInput::ApplyFormat(ComposeFormat::Plain)
                    });
                },
            );
            return;
        }
        sender.input_sender().emit(ComposeInput::ApplyFormat(to));
    }

    /// Handles [`ComposeInput::AskPlain`].
    pub(super) fn on_ask_plain(
        &mut self,
        sender: &ComponentSender<Self>,
        root: &adw::BreakpointBin,
    ) {
        let dialog = adw::AlertDialog::new(
            Some(&i18n("Switch to plain text?")),
            Some(&i18n("Formatting, images and links are removed.")),
        );
        dialog.add_response("cancel", &i18n("Cancel"));
        dialog.add_response("switch", &i18n("Switch"));
        dialog.set_response_appearance("switch", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");
        let s = sender.input_sender().clone();
        dialog.connect_response(None, move |_, resp| {
            if resp == "switch" {
                let _ = s.send(ComposeInput::ApplyFormat(ComposeFormat::Plain));
            }
        });
        dialog.present(Some(root));
        // TEGAMI_SHOWCASE_PLAIN_ANSWER=switch|cancel answers it two
        // seconds on (demo only), for a check without a pointer.
        if let (Ok(answer), Some(_)) = (
            std::env::var("TEGAMI_SHOWCASE_PLAIN_ANSWER"),
            std::env::var_os("TEGAMI_DEMO"),
        ) {
            tracing::info!("showcase plain: asked before switching");
            let (d, s) = (dialog.clone(), sender.input_sender().clone());
            gtk::glib::timeout_add_seconds_local_once(2, move || {
                d.force_close();
                if answer == "switch" {
                    let _ = s.send(ComposeInput::ApplyFormat(ComposeFormat::Plain));
                }
            });
        }
    }

    /// Handles [`ComposeInput::ApplyFormat`].
    pub(super) fn on_apply_format(&mut self, to: ComposeFormat, sender: &ComponentSender<Self>) {
        let from = self.format;
        if to == from {
            return;
        }
        self.format = to;
        self.editor
            .set_formatting_visible(to == ComposeFormat::Rich);
        if let Some(zones) = &self.drop_zones {
            zones.set_allow_inline(to == ComposeFormat::Rich);
        }
        // Every change reloads the body, plain text included: it
        // shows the message flattened to what is sent (#424), so it
        // cannot share the rich document's undo history.
        let s = sender.clone();
        self.read_body(false, move |body, _| {
            s.input_sender().emit(ComposeInput::LoadAs { to, body });
        });
    }

    /// Handles [`ComposeInput::LoadAs`].
    pub(super) fn on_load_as(&mut self, to: ComposeFormat, body: String) {
        self.editor.set_plain(to == ComposeFormat::Plain);
        self.editor.set_html(&body);
        self.editor.grab_focus();
        // TEGAMI_SHOWCASE_PLAIN_LOG logs the reloaded body, its font
        // and the text it sends (demo only).
        if std::env::var_os("TEGAMI_SHOWCASE_PLAIN_LOG").is_some()
            && std::env::var_os("TEGAMI_DEMO").is_some()
        {
            let ed = self.editor.clone();
            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || {
                ed.eval(SHOWCASE_PLAIN_LOG, move |v| {
                    tracing::info!("showcase plain: {to:?} {v}")
                });
            });
        }
    }
}
