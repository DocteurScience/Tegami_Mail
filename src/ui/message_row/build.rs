//! Building one row's widgets, once, for the factory to bind over and over.

use super::row::RowWidgets;
use super::{SwipeSurface, SENDER_COLUMN_PX, TAGS_COLUMN_PX};
use crate::ui::column_bin::ColumnBin;
use adw::prelude::*;
use relm4::prelude::*;

/// The row's widget tree. The host is what the view wraps in its item
/// widget; everything else hangs off it.
pub(super) fn build_widgets() -> RowWidgets {
    let host = gtk::Box::new(gtk::Orientation::Vertical, 0);
    host.add_css_class("message-item");

    // Replies slide open and shut instead of the list jumping to a new row
    // count the instant a conversation toggles. (PR #79)
    let revealer = gtk::Revealer::new();
    revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(200);
    revealer.set_reveal_child(true);
    host.append(&revealer);

    // The swipe surface: the fixed action strip underneath, the row's real
    // content on top, slid away from it under a drag (#swipe).
    let surface = SwipeSurface::default();
    revealer.set_child(Some(&surface));

    let swipe_bg = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    swipe_bg.set_overflow(gtk::Overflow::Hidden);
    let swipe_inner = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    swipe_inner.set_valign(gtk::Align::Center);
    swipe_inner.set_hexpand(true);
    swipe_inner.set_margin_start(16);
    swipe_inner.set_margin_end(16);
    let swipe_icon = gtk::Image::new();
    let swipe_label = gtk::Label::new(None);
    swipe_inner.append(&swipe_icon);
    swipe_inner.append(&swipe_label);
    swipe_bg.append(&swipe_inner);
    surface.set_background(&swipe_bg);

    let overlay = gtk::Overlay::new();
    surface.set_foreground(&overlay);

    // The content: avatar, unread dot and the three lines of text.
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    content.add_css_class("message-row");
    content.set_overflow(gtk::Overflow::Hidden);
    overlay.set_child(Some(&content));

    let avatar = adw::Avatar::new(38, None, true);
    avatar.set_valign(gtk::Align::Center);
    content.append(&avatar);

    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("unread-dot");
    content.append(&dot);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_hexpand(true);
    content.append(&text);

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let name = gtk::Label::new(None);
    name.set_halign(gtk::Align::Start);
    name.set_hexpand(true);
    name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    name.add_css_class("message-sender");
    top.append(&name);
    let answer = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let replied = gtk::Image::from_icon_name("mail-reply-sender-symbolic");
    let forwarded = gtk::Image::from_icon_name("mail-forward-symbolic");
    for arrow in [&replied, &forwarded] {
        arrow.add_css_class("dim-icon");
        answer.append(arrow);
    }
    top.append(&answer);
    let clip = gtk::Image::from_icon_name("mail-attachment-symbolic");
    clip.add_css_class("dim-icon");
    top.append(&clip);
    let star = gtk::Image::from_icon_name("starred-symbolic");
    star.add_css_class("star-icon");
    top.append(&star);
    // Sent mail's read receipt (#520), beside the date it was sent.
    let receipt = gtk::Image::from_icon_name("tegami-read-receipt-symbolic");
    receipt.set_visible(false);
    top.append(&receipt);
    let date = gtk::Label::new(None);
    date.set_halign(gtk::Align::End);
    // Ellipsized so it stops being the row's floor: it is the one item on
    // this line with no give (#29).
    date.set_ellipsize(gtk::pango::EllipsizeMode::End);
    date.add_css_class("message-date");
    top.append(&date);
    // The conversation's size (thread heads only), a grey pill: a count,
    // not a control. The conversation opens in the reader, never in the
    // list (owner's decision, 2026-11-07).
    let chip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    chip.add_css_class("thread-chip");
    chip.set_valign(gtk::Align::Center);
    let chip_count = gtk::Label::new(None);
    chip_count.set_hexpand(true);
    chip.append(&chip_count);
    top.append(&chip);
    text.append(&top);

    // The subject and the tag chips at its end, where a long subject gives
    // way before the sender's name would.
    let subject_line = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let subject = gtk::Label::new(None);
    subject.set_halign(gtk::Align::Start);
    subject.set_hexpand(true);
    subject.set_ellipsize(gtk::pango::EllipsizeMode::End);
    subject.add_css_class("message-subject");
    subject_line.append(&subject);
    let tags_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    tags_box.set_valign(gtk::Align::Center);
    subject_line.append(&tags_box);
    text.append(&subject_line);

    // The message's own text, at full width.
    let preview_line = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    // An encrypted message (#133) shows a lock where its text would be, in
    // the preview's own dimmed color.
    let lock = gtk::Image::from_icon_name("channel-secure-symbolic");
    lock.set_pixel_size(12);
    lock.set_valign(gtk::Align::Center);
    lock.add_css_class("message-preview");
    preview_line.append(&lock);
    let preview = gtk::Label::new(None);
    // Fill (not Start): the layout width then matches the allocation, so
    // the ellipsis lands right where the text is cut.
    preview.set_halign(gtk::Align::Fill);
    preview.set_hexpand(true);
    preview.set_xalign(0.0);
    preview.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
    preview.add_css_class("message-preview");
    preview_line.append(&preview);
    text.append(&preview_line);

    // The pills' own line, under the text, when the setting puts them
    // there (#374).
    let pills_line = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    pills_line.set_visible(false);
    text.append(&pills_line);

    // The single-line layout's row (#334): empty and hidden until a row is
    // laid out that way, when the widgets above move into it.
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    line.set_hexpand(true);
    line.set_visible(false);
    content.append(&line);
    let name_col = ColumnBin::new(SENDER_COLUMN_PX);
    let subject_col = ColumnBin::new(0);
    let date_col = ColumnBin::new(-1);
    let tags_col = ColumnBin::new(TAGS_COLUMN_PX);
    tags_col.set_visible(false);
    // The subject takes whatever room the other columns leave.
    subject_col.set_hexpand(true);
    RowWidgets {
        host,
        revealer,
        surface,
        swipe_bg,
        swipe_inner,
        swipe_icon,
        swipe_label,
        overlay,
        content,
        tags_col,
        name_col,
        subject_col,
        date_col,
        avatar,
        dot,
        text,
        top,
        line,
        name,
        answer,
        replied,
        forwarded,
        clip,
        star,
        receipt,
        date,
        chip,
        chip_count,
        subject_line,
        pills_line,
        subject,
        tags_box,
        preview_line,
        lock,
        preview,
    }
}
