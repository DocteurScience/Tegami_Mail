//! The small labels a row draws: due dates, tag pills, the tag menu.

use crate::models::Message;
use crate::ui::context_menu::MenuEntry;
use adw::prelude::*;
use relm4::prelude::*;

/// A follow-up flag's due date for the Due column (#334): the day alone,
/// the year too when it is not this one. Empty when there is none.
pub(super) fn due_label(due: i64) -> String {
    if due <= 0 {
        String::new()
    } else if crate::datefmt::year(due) == crate::datefmt::year(crate::datefmt::now()) {
        crate::datefmt::day_month(due)
    } else {
        crate::datefmt::day_month_year(due)
    }
}

/// A chip: the dot of its colour, then the name, on a tint of the colour
/// (mockups 01, 07 and 09). The dot says "a tag" before the name is read;
/// the classes give both their colours (app/tags.rs). `max_chars` caps the
/// name, which ellipsizes beyond it.
pub(crate) fn chip_box(name: &str, classes: &[&str], max_chars: i32) -> gtk::Box {
    let chip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    chip.add_css_class("tag-chip");
    for c in classes {
        chip.add_css_class(c);
    }
    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("tag-dot");
    dot.set_valign(gtk::Align::Center);
    dot.set_accessible_role(gtk::AccessibleRole::Presentation);
    chip.append(&dot);
    let label = gtk::Label::new(Some(name));
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_max_width_chars(max_chars);
    chip.append(&label);
    chip.update_property(&[gtk::accessible::Property::Label(name)]);
    chip
}

/// The chip of a tag, or of a Gmail label no tag stands for (#374).
pub(crate) fn pill_chip(pill: &crate::models::Pill, max_chars: i32) -> gtk::Box {
    match pill {
        crate::models::Pill::Tag(t) => chip_box(&t.name, &[&t.css_class()], max_chars),
        crate::models::Pill::Label(name) => {
            let hue = format!("label-hue-{}", crate::models::label_hue(name));
            let chip = chip_box(name, &["label-chip", &hue], max_chars);
            chip.set_tooltip_text(Some(name));
            chip
        }
    }
}

/// The name a pill shows.
pub(crate) fn pill_name(pill: &crate::models::Pill) -> String {
    match pill {
        crate::models::Pill::Tag(t) => t.name.clone(),
        crate::models::Pill::Label(name) => name.clone(),
    }
}

/// The tag section of a message menu (#71): one entry per tag, its swatch
/// filled where the message carries it; choosing an entry toggles that tag
/// through `toggle(keyword, add)`. `tags` is the whole list in Settings
/// order, so each entry can name its number key.
pub(crate) fn tag_menu_entries(
    tags: &[crate::config::Tag],
    msg: &Message,
    toggle: impl Fn(String, bool) + Clone + 'static,
) -> Vec<MenuEntry> {
    tags.iter()
        .enumerate()
        .map(|(i, t)| {
            let on = msg.has_tag(t);
            let keyword = t.keyword.clone();
            let toggle = toggle.clone();
            MenuEntry::new(t.name.clone(), move || toggle(keyword.clone(), !on))
                .swatch(t.color.clone(), on)
                .keys(crate::ui::keymap::tag_key(i))
        })
        .collect()
}
