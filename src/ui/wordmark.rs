//! The wordmark art of the About window and the carry-over notice, and
//! the app icon those and the first-run assistant show.

use adw::prelude::*;

/// Wordmark art, embedded so nothing is needed on disk. A 1024px
/// PNG master rather than the SVG the website uses: neither librsvg nor
/// gdk-pixbuf draws the mark's soft shadow the way a browser does, and a
/// raster export of the same artwork is the one thing both agree on.
const WORDMARK_PNG: &[u8] = include_bytes!("../../data/welcome/wordmark-black.png");

/// The same wordmark with white lettering, for a dark ground.
const WORDMARK_DARK_PNG: &[u8] = include_bytes!("../../data/welcome/wordmark-white.png");

/// The name for the About window, under the app icon: "Tegami Mail" set
/// in Cantarell, dark or white, as an SVG on a 128x42 box (the pixbuf
/// loader tells the formats apart by their bytes).
const ABOUT_WORDMARK_SVG: &[u8] = include_bytes!("../../data/about/wordmark-black.svg");

const ABOUT_WORDMARK_DARK_SVG: &[u8] = include_bytes!("../../data/about/wordmark-white.svg");

/// Which wordmark art a picture shows.
#[derive(Clone, Copy)]
pub(crate) enum Wordmark {
    /// The script lettering alone (the carry-over notice).
    Plain,
    /// The Cantarell lettering, dark or white by scheme (the About
    /// window, under the app icon).
    Name,
}

impl Wordmark {
    fn art(self, dark: bool) -> &'static [u8] {
        match (self, dark) {
            (Wordmark::Plain, false) => WORDMARK_PNG,
            (Wordmark::Plain, true) => WORDMARK_DARK_PNG,
            (Wordmark::Name, false) => ABOUT_WORDMARK_SVG,
            (Wordmark::Name, true) => ABOUT_WORDMARK_DARK_SVG,
        }
    }

    /// The art's aspect (height over width).
    fn aspect(self) -> f64 {
        match self {
            Wordmark::Plain => 293.0 / 1024.0,
            Wordmark::Name => 42.0 / 128.0,
        }
    }
}

/// Render the script lettering at 2x for crisp HiDPI, displayed at
/// `width` px.
pub(crate) fn wordmark_picture(width: i32) -> gtk::Picture {
    wordmark_picture_of(Wordmark::Plain, width)
}

/// Render the given wordmark art at 2x for crisp HiDPI, displayed at
/// `width` px.
pub(crate) fn wordmark_picture_of(art: Wordmark, width: i32) -> gtk::Picture {
    let pic = gtk::Picture::new();
    pic.set_can_shrink(true);
    pic.set_content_fit(gtk::ContentFit::Contain);
    // Black lettering on a light ground, white on a dark one; follows the
    // style manager while the window is open.
    let style = adw::StyleManager::default();
    let apply = move |pic: &gtk::Picture, dark: bool| {
        if let Some(tex) = wordmark_texture(art, width, dark) {
            pic.set_paintable(Some(&tex));
        }
    };
    apply(&pic, style.is_dark());
    let weak = pic.downgrade();
    style.connect_dark_notify(move |s| {
        if let Some(pic) = weak.upgrade() {
            apply(&pic, s.is_dark());
        }
    });
    pic
}

/// The wordmark rendered at 2x `width` for crisp HiDPI.
fn wordmark_texture(art: Wordmark, width: i32, dark: bool) -> Option<gtk::gdk::Texture> {
    let loader = gtk::gdk_pixbuf::PixbufLoader::new();
    loader.connect_size_prepared(move |l, w, h| {
        let scale = (width * 2) as f64 / w.max(1) as f64;
        l.set_size(width * 2, (h as f64 * scale) as i32);
    });
    loader.write(art.art(dark)).ok()?;
    loader.close().ok()?;
    loader.pixbuf().map(|pb| gtk::gdk::Texture::for_pixbuf(&pb))
}

/// The script lettering's height for a given width.
pub(crate) fn wordmark_height(width: f64) -> i32 {
    wordmark_height_of(Wordmark::Plain, width)
}

/// A wordmark's height for a given width.
pub(crate) fn wordmark_height_of(art: Wordmark, width: f64) -> i32 {
    (width * art.aspect()) as i32
}

/// The app icon at `size` px, always the one bundled in the binary.
///
/// Looked up by name in the system's icon theme, `APP_ID` finds whatever an
/// earlier install left in `~/.local/share/icons` or `/usr/share/icons`
/// first, so the assistant and About showed an old icon on a machine that
/// had one. A theme of its own that only searches the app's resources
/// cannot be shadowed.
pub(crate) fn app_icon_image(size: i32) -> gtk::Image {
    let theme = gtk::IconTheme::new();
    theme.set_search_path(&[] as &[&std::path::Path]);
    theme.set_resource_path(&["/io/github/DocteurScience/TegamiMail/icons"]);
    let paintable = theme.lookup_icon(
        crate::APP_ID,
        &[],
        size,
        2,
        gtk::TextDirection::None,
        gtk::IconLookupFlags::empty(),
    );
    let icon = gtk::Image::from_paintable(Some(&paintable));
    icon.set_pixel_size(size);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    icon
}

#[cfg(test)]
mod tests {
    /// The icon bundled in the binary is a copy of the one installed with
    /// the app; when the app icon changed, the copy was left behind and the
    /// assistant and About showed the old icon.
    #[test]
    fn the_bundled_app_icon_is_the_installed_one() {
        assert!(
            include_bytes!(
                "../../resources/icons/scalable/apps/io.github.DocteurScience.TegamiMail.svg"
            ) == include_bytes!(
                "../../data/icons/hicolor/scalable/apps/io.github.DocteurScience.TegamiMail.svg"
            ),
            "run tools/gen-icon-gresource.sh"
        );
        assert!(
            include_bytes!(
                "../../resources/icons/scalable/apps/io.github.DocteurScience.TegamiMail.Beta.svg"
            ) == include_bytes!(
                "../../data/icons/hicolor/scalable/apps/io.github.DocteurScience.TegamiMail.Beta.svg"
            ),
            "run tools/gen-icon-gresource.sh"
        );
    }
}
