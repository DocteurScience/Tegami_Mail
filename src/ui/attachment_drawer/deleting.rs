//! The "Deleting…" treatment a file gets while it is removed from the server
//! (#289): its actions veiled, a label over them, and a fade once it is gone.

use adw::prelude::*;

use crate::i18n::i18n;

/// How long a file deleted from the server takes to fade out (#289).
const DELETE_FADE_MS: u32 = 420;

/// "Deleting…", in place of a file's actions while it is removed from the
/// server (#289).
pub(super) fn deleting_label() -> gtk::Label {
    let l = gtk::Label::new(Some(&i18n("Deleting…")));
    l.add_css_class("attachment-deleting-label");
    l.add_css_class("caption");
    l.set_valign(gtk::Align::Center);
    l
}

/// A row's actions while its file is removed from the server: still there,
/// unseen and out of reach, so the row keeps its size, with "Deleting…"
/// over them.
pub(super) fn deleting_over(actions: &impl IsA<gtk::Widget>) -> gtk::Overlay {
    actions.set_opacity(0.0);
    actions.set_can_target(false);
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(actions));
    let label = deleting_label();
    label.set_halign(gtk::Align::End);
    overlay.add_overlay(&label);
    overlay
}

/// A thumbnail's cover while its file is removed from the server: a pale
/// red veil, with "Deleting…" across the middle when `labelled` (a
/// thumbnail too small for it says so beneath instead).
pub(super) fn deleting_veil(labelled: bool) -> gtk::Widget {
    let veil = gtk::Box::new(gtk::Orientation::Vertical, 0);
    veil.add_css_class("attachment-deleting-veil");
    veil.set_hexpand(true);
    veil.set_vexpand(true);
    veil.set_overflow(gtk::Overflow::Hidden);
    if labelled {
        let label = deleting_label();
        label.set_halign(gtk::Align::Center);
        label.set_vexpand(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        veil.append(&label);
    }
    veil.upcast()
}

/// Fade a deleted file's cell or row out, then call `done` to take it away.
/// Libadwaita skips the fade when animations are off or the widget is not
/// on screen, and `done` comes at once.
pub(super) fn fade_out(widget: &impl IsA<gtk::Widget>, done: impl Fn() + 'static) {
    let w = widget.clone().upcast::<gtk::Widget>();
    let target = adw::CallbackAnimationTarget::new({
        let w = w.clone();
        move |v| w.set_opacity(v)
    });
    let anim = adw::TimedAnimation::new(&w, 1.0, 0.0, DELETE_FADE_MS, target);
    anim.set_easing(adw::Easing::EaseOutCubic);
    anim.connect_done(move |_| done());
    anim.play();
}
