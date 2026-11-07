//! The New Tag dialog (docs/DESIGN.md, "Tags and favourite"; mockup 09): a
//! name, one of the palette's eight colours, each with its name under it so
//! the choice never rests on colour alone, and a preview of the chip. Cancel
//! and Create in the header bar, Create the suggested one and only once
//! there is a name. The same dialog edits a tag, with Save.
//!
//! It only asks: the caller makes the tag (its keyword, saving it) from
//! what it is handed.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;

use crate::config::TAG_COLORS;
use crate::i18n::i18n;

/// The palette colour `hex` is, by name; `None` for a colour from outside
/// the palette (one set by an older Tegami Mail).
pub(crate) fn color_name(hex: &str) -> Option<String> {
    TAG_COLORS
        .iter()
        .position(|c| c.eq_ignore_ascii_case(hex))
        .map(|i| color_names()[i].clone())
}

/// The palette colours' names, in `TAG_COLORS` order.
fn color_names() -> [String; 8] {
    [
        i18n("Blue"),
        i18n("Green"),
        i18n("Yellow"),
        i18n("Orange"),
        i18n("Red"),
        i18n("Purple"),
        i18n("Brown"),
        i18n("Gray"),
    ]
}

/// Ask for a new tag over `parent`. `color` is the one picked at first (the
/// first the existing tags leave free, say); `on_create` gets the name, as
/// typed but trimmed, and the colour.
pub(crate) fn show(
    parent: &impl IsA<gtk::Widget>,
    color: &str,
    on_create: impl Fn(String, String) + 'static,
) {
    open(
        parent,
        &i18n("New tag"),
        &i18n("Create"),
        "",
        color,
        on_create,
    );
}

/// Edit a tag named `name` in `color` over `parent`; `on_save` gets the
/// name and colour as the dialog leaves them. A colour from outside the
/// palette stays until a swatch is picked.
pub(crate) fn edit(
    parent: &impl IsA<gtk::Widget>,
    name: &str,
    color: &str,
    on_save: impl Fn(String, String) + 'static,
) {
    open(
        parent,
        &i18n("Edit tag"),
        &i18n("Save"),
        name,
        color,
        on_save,
    );
}

fn open(
    parent: &impl IsA<gtk::Widget>,
    title: &str,
    verb: &str,
    initial_name: &str,
    color: &str,
    on_create: impl Fn(String, String) + 'static,
) {
    let dialog = adw::Dialog::new();
    dialog.set_title(title);
    dialog.set_content_width(420);

    let header = adw::HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
    let cancel = gtk::Button::with_label(&i18n("Cancel"));
    let create = gtk::Button::with_label(verb);
    create.add_css_class("suggested-action");
    create.set_sensitive(!initial_name.trim().is_empty());
    header.pack_start(&cancel);
    header.pack_end(&create);

    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_margin_top(12);
    body.set_margin_bottom(18);
    body.set_margin_start(18);
    body.set_margin_end(18);

    let name_label = heading(&i18n("Name"));
    body.append(&name_label);
    let name = gtk::Entry::new();
    name.set_placeholder_text(Some(&i18n("Tag name")));
    name.set_text(initial_name);
    name.update_relation(&[gtk::accessible::Relation::LabelledBy(&[
        name_label.upcast_ref()
    ])]);
    body.append(&name);

    let color_label = heading(&i18n("Color"));
    color_label.set_margin_top(8);
    body.append(&color_label);
    let grid = gtk::Grid::new();
    grid.set_column_homogeneous(true);
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    // A new tag starts on a palette colour; an edited one keeps its own,
    // even one from outside the palette, until a swatch is picked.
    let chosen = Rc::new(RefCell::new(
        if color.is_empty() {
            TAG_COLORS[0]
        } else {
            color
        }
        .to_string(),
    ));

    let preview_label = heading(&i18n("Preview"));
    preview_label.set_margin_top(8);
    // Where the preview chip goes; rebuilt as the name and colour change.
    let preview = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    preview.set_halign(gtk::Align::Start);
    let show_preview = {
        let (preview, chosen, name) = (preview.clone(), chosen.clone(), name.clone());
        move || {
            let typed = name.text().trim().to_string();
            let shown = if typed.is_empty() {
                i18n("Tag name")
            } else {
                typed
            };
            // The list's own chip (mockup 09): the palette colours have a
            // rule each, the one a Gmail label wears (app/tags.rs).
            let hue = TAG_COLORS
                .iter()
                .position(|hex| hex.eq_ignore_ascii_case(&chosen.borrow()))
                .map(|i| format!("label-hue-{i}"));
            let mut classes = vec!["label-chip"];
            classes.extend(hue.as_deref());
            while let Some(child) = preview.first_child() {
                preview.remove(&child);
            }
            preview.append(&crate::ui::message_row::chip_box(&shown, &classes, 30));
        }
    };

    let mut first: Option<gtk::ToggleButton> = None;
    for (i, (hex, label)) in TAG_COLORS.iter().zip(color_names()).enumerate() {
        let cell = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let button = gtk::ToggleButton::new();
        button.add_css_class("circular");
        button.add_css_class("flat");
        button.add_css_class("tag-swatch");
        button.set_halign(gtk::Align::Center);
        // The choice is shown by a tick on the disc and a ring around it,
        // not by the button's own pressed shade, which the disc hides.
        let face = gtk::Overlay::new();
        face.set_child(Some(&disc(hex)));
        let tick = gtk::Image::from_icon_name("object-select-symbolic");
        tick.add_css_class("tag-swatch-tick");
        tick.set_accessible_role(gtk::AccessibleRole::Presentation);
        tick.set_visible(false);
        face.add_overlay(&tick);
        button.set_child(Some(&face));
        button.set_tooltip_text(Some(&label));
        button.update_property(&[gtk::accessible::Property::Label(&label)]);
        match &first {
            Some(f) => button.set_group(Some(f)),
            None => first = Some(button.clone()),
        }
        {
            let (chosen, show_preview, hex) =
                (chosen.clone(), show_preview.clone(), hex.to_string());
            let tick = tick.clone();
            button.connect_toggled(move |b| {
                tick.set_visible(b.is_active());
                if b.is_active() {
                    *chosen.borrow_mut() = hex.clone();
                    show_preview();
                }
            });
        }
        // After the handler is connected, so the first choice is ticked.
        if hex.eq_ignore_ascii_case(&chosen.borrow()) {
            button.set_active(true);
        }
        cell.append(&button);
        let name_below = gtk::Label::new(Some(&label));
        name_below.add_css_class("caption");
        cell.append(&name_below);
        grid.attach(&cell, (i % 4) as i32, (i / 4) as i32, 1, 1);
    }
    body.append(&grid);
    body.append(&preview_label);
    body.append(&preview);
    show_preview();

    {
        let (create, show_preview) = (create.clone(), show_preview.clone());
        name.connect_changed(move |e| {
            create.set_sensitive(!e.text().trim().is_empty());
            show_preview();
        });
    }

    let view = adw::ToolbarView::new();
    view.add_top_bar(&header);
    view.set_content(Some(&body));
    dialog.set_child(Some(&view));
    dialog.set_default_widget(Some(&create));
    name.set_activates_default(true);

    {
        let dialog = dialog.clone();
        cancel.connect_clicked(move |_| {
            dialog.close();
        });
    }
    {
        let (dialog, name, chosen) = (dialog.clone(), name.clone(), chosen.clone());
        create.connect_clicked(move |_| {
            let typed = name.text().trim().to_string();
            if typed.is_empty() {
                return;
            }
            on_create(typed, chosen.borrow().clone());
            dialog.close();
        });
    }
    dialog.present(Some(parent.as_ref()));
    name.grab_focus();
    crate::ui::tags_panel::showcase_capture(view.upcast_ref());
}

/// A small heading over a part of the dialog.
fn heading(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.add_css_class("caption-heading");
    label
}

/// A 32px disc of `hex`, drawn rather than styled: the palette is data, not
/// CSS, and the disc carries no meaning its tooltip and name do not.
fn disc(hex: &str) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.set_content_width(32);
    area.set_content_height(32);
    area.set_accessible_role(gtk::AccessibleRole::Presentation);
    let rgba = gtk::gdk::RGBA::parse(hex).unwrap_or(gtk::gdk::RGBA::new(0.5, 0.5, 0.5, 1.0));
    area.set_draw_func(move |_, cr, w, h| {
        cr.set_source_rgba(
            rgba.red().into(),
            rgba.green().into(),
            rgba.blue().into(),
            rgba.alpha().into(),
        );
        cr.arc(
            f64::from(w) / 2.0,
            f64::from(h) / 2.0,
            f64::from(w.min(h)) / 2.0,
            0.0,
            std::f64::consts::TAU,
        );
        let _ = cr.fill();
    });
    area
}
