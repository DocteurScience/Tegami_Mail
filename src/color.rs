//! Small color helpers for per-account theming (avatar circles and their
//! rings, text on a colored ground).

fn rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let c = gtk::gdk::RGBA::parse(hex).ok()?;
    Some((
        (c.red() * 255.0).round() as u8,
        (c.green() * 255.0).round() as u8,
        (c.blue() * 255.0).round() as u8,
    ))
}

pub(crate) fn readable_text(hex: &str) -> &'static str {
    if let Some((r, g, b)) = rgb(hex) {
        let lum = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
        if lum > 150.0 {
            return "rgba(0,0,0,0.85)";
        }
    }
    "#ffffff"
}

/// Format a `gdk::RGBA` as a `#rrggbb` string.
pub(crate) fn to_hex(c: &gtk::gdk::RGBA) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (c.red() * 255.0).round() as u8,
        (c.green() * 255.0).round() as u8,
        (c.blue() * 255.0).round() as u8,
    )
}

/// `hex` pulled towards black by `amount` (0.0 leaves it alone, 1.0 is black).
///
/// Used for hairlines that a palette pitches brighter than the surfaces
/// they divide — see the theme's list/reader separator.
pub(crate) fn darken(hex: &str, amount: f64) -> String {
    let Some((r, g, b)) = rgb(hex) else {
        return hex.to_string();
    };
    let keep = (1.0 - amount).clamp(0.0, 1.0);
    let dim = |c: u8| (c as f64 * keep).round() as u8;
    format!("#{:02x}{:02x}{:02x}", dim(r), dim(g), dim(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_text_is_dark_only_on_a_light_ground() {
        assert_eq!(readable_text("#ffffff"), "rgba(0,0,0,0.85)");
        assert_eq!(readable_text("#ffff00"), "rgba(0,0,0,0.85)");
        assert_eq!(readable_text("#808080"), "#ffffff");
        assert_eq!(readable_text("#000000"), "#ffffff");
        assert_eq!(readable_text("no color"), "#ffffff");
    }

    #[test]
    fn darken_scales_each_channel_and_clamps_the_amount() {
        assert_eq!(darken("#ff8000", 0.5), "#804000");
        assert_eq!(darken("#102030", 2.0), "#000000");
        assert_eq!(darken("#102030", -1.0), "#102030");
        assert_eq!(darken("#102030", 0.0), "#102030");
        // Unparseable input comes back as it was.
        assert_eq!(darken("no color", 0.5), "no color");
    }

    #[test]
    fn to_hex_rounds_each_channel() {
        assert_eq!(to_hex(&gtk::gdk::RGBA::new(1.0, 0.5, 0.0, 0.3)), "#ff8000");
        assert_eq!(to_hex(&gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0)), "#000000");
    }
}
