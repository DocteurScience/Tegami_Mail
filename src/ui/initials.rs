//! The initials circle a sender gets when no picture is known: drawn here
//! rather than by `adw::Avatar`'s label, so the letters sit centred by their
//! ink. A label centres its *logical* box, and a glyph's ink rarely fills
//! that box evenly — a lone "J" or "T" drifts, and pairs lean — which is
//! what this paintable corrects, at every size it is drawn at.
//!
//! Colors follow libadwaita's avatar palette (its fourteen hues, chosen
//! by the same hash of the name, so a sender keeps their hue), drawn as
//! mockup 01 does: a flat pastel ground and dark letters of the same hue,
//! not libadwaita's lit gradient under white letters.
//!
//! The same drawing serves the sidebar's account circles (a glyph alone
//! over the circle's own color — see [`glyph_picture`]) and the reader's
//! cards, where it is rendered to a PNG the document embeds
//! ([`png_data_uri`]).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use gtk::{gdk, glib, graphene, gsk, pango, prelude::*, subclass::prelude::*};

/// Ground and ink for each of libadwaita's `avatar.color1`…`color14`:
/// the hue of its gradient's lower stop, at 86 % lightness for the ground
/// and 24 % for the letters (mockup 01's blue is `#c1d8f7` on `#0f3a78`).
/// Every pair is above 5.5:1, past the 4.5:1 text needs; one table for
/// both themes, as libadwaita has.
const PALETTE: [(&str, &str); 14] = [
    ("#c2d9f5", "#123968"),
    ("#c1e9f6", "#095771"),
    ("#c5f1de", "#176342"),
    ("#d2e9cd", "#2d5525"),
    ("#f6e8c1", "#715609"),
    ("#f6e2c1", "#714909"),
    ("#f6d5c1", "#713109"),
    ("#f6c1c7", "#6d0d18"),
    ("#f6c1d0", "#6b0f29"),
    ("#e3cbeb", "#4b2259"),
    ("#d4c9ee", "#301e5d"),
    ("#e9dece", "#544126"),
    ("#e9dace", "#543a26"),
    ("#dbdbdc", "#3d3c3e"),
];

/// The palette entry `adw::Avatar` gives `text`: GLib's `g_str_hash`
/// (djb2) modulo the palette, as libadwaita does it.
fn palette_index(text: &str) -> usize {
    let hash = text.bytes().fold(5381u32, |h, b| {
        h.wrapping_mul(33).wrapping_add(u32::from(b))
    });
    (hash % PALETTE.len() as u32) as usize
}

/// The initials `adw::Avatar` would show for `text`: the first letter of
/// the first word and of the last, uppercased; nothing for an empty name.
fn initials_of(text: &str) -> String {
    let upper = text.trim().to_uppercase();
    let mut words = upper
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphanumeric));
    let Some(first) = words.next() else {
        return String::new();
    };
    let mut out: String = first.chars().take(1).collect();
    if let Some(last) = words.next_back() {
        out.extend(last.chars().take(1));
    }
    out
}

thread_local! {
    /// A label to borrow the theme font from (its Pango context carries
    /// the display's font settings), never shown.
    static FONT_SOURCE: gtk::Label = gtk::Label::new(None);
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub(crate) struct InitialsPaintable {
        pub(super) initials: RefCell<String>,
        /// The flat ground, or nothing (the widget beneath paints its own).
        pub(super) ground: RefCell<Option<gdk::RGBA>>,
        pub(super) fg: RefCell<Option<gdk::RGBA>>,
        /// Text height as a share of the shorter side; 0 = by the count
        /// of letters, as the avatar sizes its label.
        pub(super) scale: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InitialsPaintable {
        const NAME: &'static str = "TegamiInitialsPaintable";
        type Type = super::InitialsPaintable;
        type Interfaces = (gdk::Paintable,);
    }

    impl ObjectImpl for InitialsPaintable {}

    impl PaintableImpl for InitialsPaintable {
        fn flags(&self) -> gdk::PaintableFlags {
            gdk::PaintableFlags::SIZE | gdk::PaintableFlags::CONTENTS
        }

        fn snapshot(&self, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let Some(snapshot) = snapshot.downcast_ref::<gtk::Snapshot>() else {
                return;
            };
            // High contrast (mockup 16): the window's ground, the text
            // colour for the letters and a ring, never a tint.
            if let Some((ground, ink)) = high_contrast_pair() {
                let r = (width.min(height) / 2.0) as f32;
                let rect = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
                snapshot.append_color(&ground, &rect);
                let circle = gsk::RoundedRect::from_rect(rect, r);
                snapshot.append_border(&circle, &[1.5; 4], &[ink; 4]);
            } else if let Some(ground) = *self.ground.borrow() {
                // The circle's ground: the avatar clips this to its disc.
                snapshot.append_color(
                    &ground,
                    &graphene::Rect::new(0.0, 0.0, width as f32, height as f32),
                );
            }

            let initials = self.initials.borrow();
            if initials.is_empty() {
                return;
            }
            let fg = high_contrast_pair()
                .map(|(_, ink)| ink)
                .unwrap_or_else(|| self.fg.borrow().unwrap_or(gdk::RGBA::WHITE));
            // Bold, at a size that lets a pair sit comfortably inside the
            // disc: the same weight of presence the avatar's own label has.
            let size = width.min(height);
            let share = match self.scale.get() {
                s if s > 0.0 => s,
                _ if initials.chars().count() > 1 => 0.42,
                _ => 0.5,
            };
            let px = size * share;
            let layout = FONT_SOURCE.with(|l| l.create_pango_layout(Some(initials.as_str())));
            let mut desc = layout.context().font_description().unwrap_or_default();
            desc.set_weight(pango::Weight::Bold);
            desc.set_absolute_size(px * f64::from(pango::SCALE));
            layout.set_font_description(Some(&desc));

            // Centre the ink, not the logical box.
            let scale = f64::from(pango::SCALE);
            let (ink, _) = layout.extents();
            let ink_x = f64::from(ink.x()) / scale;
            let ink_y = f64::from(ink.y()) / scale;
            let ink_w = f64::from(ink.width()) / scale;
            let ink_h = f64::from(ink.height()) / scale;
            let dx = (width - ink_w) / 2.0 - ink_x;
            let dy = (height - ink_h) / 2.0 - ink_y;
            snapshot.save();
            snapshot.translate(&graphene::Point::new(dx as f32, dy as f32));
            snapshot.append_layout(&layout, &fg);
            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub(crate) struct InitialsPaintable(ObjectSubclass<imp::InitialsPaintable>)
        @implements gdk::Paintable;
}

impl InitialsPaintable {
    /// The circle for `name` (a sender's display name): its initials in the
    /// color libadwaita would give that name. `None` when there is no
    /// letter to show, so the avatar falls back to its silhouette.
    pub(super) fn for_name(name: &str) -> Option<Self> {
        let initials = initials_of(name);
        if initials.is_empty() {
            return None;
        }
        let (ground, ink) = sender_colors(name);
        Some(Self::solid(&initials, ground, ink, 0.0))
    }

    /// `text` alone (initials, or an emoji) in `fg`, over nothing: for a
    /// circle that paints its own color. `scale` is the text height as a
    /// share of the circle.
    pub(crate) fn glyph(text: &str, fg: gdk::RGBA, scale: f64) -> Self {
        let this: Self = glib::Object::new();
        this.imp().initials.replace(text.to_string());
        this.imp().fg.replace(Some(fg));
        this.imp().scale.set(scale);
        this
    }

    /// `text` in `fg` on a flat `bg`.
    pub(crate) fn solid(text: &str, bg: gdk::RGBA, fg: gdk::RGBA, scale: f64) -> Self {
        let this = Self::glyph(text, fg, scale);
        this.imp().ground.replace(Some(bg));
        this
    }
}

/// The ground and the letters' color of `name`'s initials circle, the same
/// in the message list and the reader.
pub(crate) fn sender_colors(name: &str) -> (gdk::RGBA, gdk::RGBA) {
    let (ground, ink) = PALETTE[palette_index(name)];
    let rgba = |s: &str| gdk::RGBA::parse(s).unwrap_or(gdk::RGBA::BLACK);
    (rgba(ground), rgba(ink))
}

/// The initials `adw::Avatar` would show for `name`, for a circle drawn
/// elsewhere (the reader's cards).
pub(crate) fn initials(name: &str) -> String {
    initials_of(name)
}

/// The ground and ink every circle takes under high contrast: white and
/// black, or the other way round on dark. `None` otherwise, and where there
/// is no display (tests).
fn high_contrast_pair() -> Option<(gdk::RGBA, gdk::RGBA)> {
    if !gtk::is_initialized_main_thread() {
        return None;
    }
    let style = adw::StyleManager::default();
    style.is_high_contrast().then(|| {
        if style.is_dark() {
            (gdk::RGBA::BLACK, gdk::RGBA::WHITE)
        } else {
            (gdk::RGBA::WHITE, gdk::RGBA::BLACK)
        }
    })
}

/// A sidebar circle's glyph: `text` ink-centred in the color that reads
/// on `bg_hex` (the circle's own ground, painted by its CSS), exactly
/// `size` square — sized outright rather than expanding, since an expand
/// flag would climb into the row and stretch it.
pub(super) fn glyph_picture(text: &str, bg_hex: &str, scale: f64, size: i32) -> gtk::Picture {
    let fg = gdk::RGBA::parse(crate::color::readable_text(bg_hex)).unwrap_or(gdk::RGBA::WHITE);
    let picture = gtk::Picture::for_paintable(&InitialsPaintable::glyph(text, fg, scale));
    picture.set_content_fit(gtk::ContentFit::Fill);
    picture.set_can_shrink(true);
    picture.set_size_request(size, size);
    picture.set_halign(gtk::Align::Center);
    picture.set_valign(gtk::Align::Center);
    picture
}

thread_local! {
    /// Reader-card circles already rendered this session, by what they
    /// show: a conversation repeats its senders, and every render of the
    /// document would otherwise draw them again.
    static PNG_CACHE: RefCell<HashMap<(String, String, i32), String>> = RefCell::new(HashMap::new());
}

/// The session's rendered reader-card circles, for the memory section of
/// an export: how many, and their `data:` URIs' bytes.
pub(crate) fn cache_stats() -> (usize, usize) {
    PNG_CACHE.with(|c| {
        let c = c.borrow();
        (c.len(), c.values().map(String::len).sum::<usize>())
    })
}

/// `text` in `fg` on a flat `bg`, as a `data:` PNG for an `<img>` of `size` CSS
/// pixels — rendered at the screen's scale, so it is as crisp as the
/// document's own text. Drawn through the window's renderer; `None` before
/// a window exists.
pub(super) fn png_data_uri(text: &str, bg: gdk::RGBA, fg: gdk::RGBA, size: i32) -> Option<String> {
    // High contrast draws another circle for the same name.
    let key = (
        text.to_string(),
        format!("{bg}{fg}{:?}", high_contrast_pair()),
        size,
    );
    if let Some(hit) = PNG_CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return Some(hit);
    }
    // No display (the document builder under test, say): there is nothing to
    // render with, and asking GTK for its toplevels would panic rather than
    // say so. The caller falls back to markup.
    if !gtk::is_initialized_main_thread() {
        return None;
    }
    let window = gtk::Window::list_toplevels()
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Window>().ok())?;
    let renderer = window.renderer()?;
    let scale = window.scale_factor().max(1);
    let px = size * scale;
    let paintable = InitialsPaintable::solid(text, bg, fg, 0.0);
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, f64::from(px), f64::from(px));
    let node = snapshot.to_node()?;
    let texture = renderer.render_texture(
        &node,
        Some(&graphene::Rect::new(0.0, 0.0, px as f32, px as f32)),
    );
    let uri = format!(
        "data:image/png;base64,{}",
        glib::base64_encode(&texture.save_to_png_bytes())
    );
    PNG_CACHE.with(|c| {
        c.borrow_mut().insert(key, uri.clone());
    });
    Some(uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_follow_the_avatar_rules() {
        assert_eq!(initials_of("Marcus Chen"), "MC");
        assert_eq!(initials_of("  sophie   turner "), "ST");
        assert_eq!(initials_of("Apple"), "A");
        assert_eq!(initials_of("GNOME Foundation Board"), "GB");
        assert_eq!(initials_of("émile zola"), "ÉZ");
        assert_eq!(initials_of("  "), "");
        // GLib's djb2, as libadwaita hashes the name for its color.
        assert_eq!(palette_index(""), (5381u32 % 14) as usize);
        assert!(palette_index("Marcus Chen") < PALETTE.len());
    }

    /// The letters stay readable on every ground (WCAG 4.5:1).
    #[test]
    fn every_palette_pair_reads() {
        fn lum(hex: &str) -> f64 {
            let c = gdk::RGBA::parse(hex).unwrap();
            let f = |v: f32| {
                let v = f64::from(v);
                if v <= 0.03928 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * f(c.red()) + 0.7152 * f(c.green()) + 0.0722 * f(c.blue())
        }
        for (ground, ink) in PALETTE {
            let ratio = (lum(ground) + 0.05) / (lum(ink) + 0.05);
            assert!(ratio >= 4.5, "{ink} on {ground}: {ratio:.1}");
        }
    }
}
