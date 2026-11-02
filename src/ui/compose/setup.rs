//! Steps of the composer's `init` that belong to no one concern.

use super::messages::{ComposeInput, ComposeOutput};
use super::Compose;
use crate::config::ComposeFormat;
use crate::i18n::i18n;
use crate::ui::rich_editor::RichEditor;
use adw::prelude::*;
use relm4::prelude::*;

/// The buttons at the far end of the format bar: signature, translate and
/// preview, in that order; and Attach a File, after the formatting tools.
/// The format is chosen in More actions (docs/DESIGN.md, "Menus").
pub(super) fn add_toolbar_end_buttons(
    editor: &RichEditor,
    format: ComposeFormat,
    has_signatures: bool,
    sender: &ComponentSender<Compose>,
) -> (
    gtk::ToggleButton,
    adw::ButtonContent,
    gtk::Button,
    gtk::Button,
) {
    // The preview toggle belongs with the other formatting controls, at
    // the far end of the same row. It keeps its word. It is a state rather than an action,
    // and an eye on its own leaves which state to guesswork.
    let preview_content = adw::ButtonContent::builder()
        .icon_name("eye-open-negative-filled-symbolic")
        .label(i18n("Preview"))
        .build();
    let preview_btn = gtk::ToggleButton::builder().child(&preview_content).build();
    preview_btn.set_tooltip_text(Some(i18n("Show the message as it will be sent").as_str()));
    preview_btn.add_css_class("flat");
    preview_btn.set_can_focus(false);
    preview_btn.set_can_shrink(true);
    preview_btn.set_visible(format.is_source());
    {
        let s = sender.input_sender().clone();
        preview_btn.connect_toggled(move |b| {
            let _ = s.send(ComposeInput::TogglePreview(b.is_active()));
        });
    }
    // Preview goes last, the far end of the right-aligned group, where
    // its coming and going pushes nothing around. Translate (#327) goes
    // before it: it is there in every format, but only once a translation
    // service is set up.
    let translate_btn = gtk::Button::from_icon_name("translate-symbolic");
    translate_btn.set_tooltip_text(Some(i18n("Translate what you wrote").as_str()));
    translate_btn.add_css_class("flat");
    translate_btn.set_can_focus(false);
    translate_btn.set_visible(crate::services::translate::load().enabled());
    {
        let s = sender.input_sender().clone();
        translate_btn.connect_clicked(move |_| {
            let _ = s.send(ComposeInput::TranslateMenu);
        });
    }
    let signature_btn = gtk::Button::from_icon_name("signature-symbolic");
    signature_btn.set_tooltip_text(Some(i18n("Insert Signature").as_str()));
    signature_btn.add_css_class("flat");
    signature_btn.set_can_focus(false);
    signature_btn.set_visible(has_signatures);
    {
        let s = sender.input_sender().clone();
        signature_btn.connect_clicked(move |_| {
            let _ = s.send(ComposeInput::SignatureClicked);
        });
        let s = sender.input_sender().clone();
        let right = gtk::GestureClick::new();
        right.set_button(gtk::gdk::BUTTON_SECONDARY);
        right.connect_pressed(move |g, _, _, _| {
            g.set_state(gtk::EventSequenceState::Claimed);
            let _ = s.send(ComposeInput::SignatureMenu);
        });
        signature_btn.add_controller(right);
    }
    // Attach a File, with its word, right after the formatting tools as in
    // the design's composer (docs/DESIGN.md, "Replying and composing"). It
    // moved here from the header bar, which keeps to Cancel, the title,
    // More actions and Send. Kept when the formatting tools are hidden, so
    // that a message written as plain text or source can be attached to.
    let attach_btn = gtk::Button::builder()
        .child(
            &adw::ButtonContent::builder()
                .icon_name("mail-attachment-symbolic")
                .label(i18n("Attach a File…"))
                .can_shrink(true)
                .build(),
        )
        .build();
    attach_btn.set_tooltip_text(Some(i18n("Attach files").as_str()));
    attach_btn.add_css_class("flat");
    attach_btn.set_can_focus(false);
    {
        let s = sender.input_sender().clone();
        attach_btn.connect_clicked(move |_| {
            let _ = s.send(ComposeInput::AttachFiles);
        });
    }
    editor.append_kept_tool(&attach_btn);
    editor.toolbar_end().append(&signature_btn);
    editor.toolbar_end().append(&translate_btn);
    editor.toolbar_end().append(&preview_btn);
    (preview_btn, preview_content, translate_btn, signature_btn)
}

/// What the editor tells the composer: images lifted out as attachments,
/// its text history, and spelling language changes. The body is checked
/// for grammar too, when a server is set up for it (#452).
pub(super) fn connect_editor(editor: &RichEditor, sender: &ComponentSender<Compose>) {
    editor.check_grammar();

    // "Send as Attachment Instead" on an inline image: the editor lifts
    // it to a temp file and it joins the attachment chips here.
    {
        let s = sender.input_sender().clone();
        editor.connect_send_as_attachment(move |path| {
            let _ = s.send(ComposeInput::AddAttachments(vec![path]));
        });
    }

    // The body's text history joins the composer's order (#200): every
    // edit in there is one step, wherever the attachments' steps fall.
    {
        let s = sender.input_sender().clone();
        editor.connect_history_changed(move |can_undo, _| {
            let _ = s.send(ComposeInput::BodyHistory(can_undo));
        });
    }

    {
        let s = sender.output_sender().clone();
        editor.connect_spell_languages(move |langs| {
            s.emit(ComposeOutput::SetSpellcheckLangs(langs));
        });
    }
}
