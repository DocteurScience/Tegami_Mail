//! Dark-mode colour adaptation and the reader style.

use super::*;

// ===== Dark-mode color adaptation (issue #35) =====

/// The core failure: near-black text with no background of its own must
/// come out light, or it vanishes on the dark ground.
#[test]
fn dark_text_is_lightened() {
    let out = adapt_colors_for_dark(r#"<p style="color:#000">x</p>"#);
    assert_eq!(out, r#"<p style="color:#ffffff">x</p>"#);
    let out = adapt_colors_for_dark(r#"<p style="color: #333333;">x</p>"#);
    assert_eq!(out, r#"<p style="color: #cccccc;">x</p>"#);
    let out = adapt_colors_for_dark(r#"<p style="color:rgb(20, 20, 20)">x</p>"#);
    assert!(out.contains("color:#ebebeb"), "{out}");
}

/// Light backgrounds mirror down to surfaces; the floor keeps them above
/// pure black so they still read as cards.
#[test]
fn light_backgrounds_are_darkened() {
    let out = adapt_colors_for_dark(r#"<td style="background-color:#ffffff">x</td>"#);
    assert_eq!(out, r#"<td style="background-color:#141414">x</td>"#);
    let out = adapt_colors_for_dark(r#"<td style="background:#f6f6f6 no-repeat">x</td>"#);
    assert!(out.contains("background:#141414 no-repeat"), "{out}");
}

/// Mail already designed for a dark ground passes through unchanged.
#[test]
fn dark_designed_mail_is_untouched() {
    let doc = r#"<div style="color:#eeeeee;background-color:#222222">x</div>"#;
    assert_eq!(adapt_colors_for_dark(doc), doc);
}

/// Legacy attributes carry colors too — `<font color>` and `bgcolor`,
/// with or without the leading `#`.
#[test]
fn legacy_color_attributes_are_adapted() {
    let out = adapt_colors_for_dark(r##"<font color="#111111">x</font>"##);
    assert_eq!(out, r##"<font color="#eeeeee">x</font>"##);
    let out = adapt_colors_for_dark(r#"<table bgcolor="ffffff"><tr></tr></table>"#);
    assert_eq!(out, r##"<table bgcolor="#141414"><tr></tr></table>"##);
    let out = adapt_colors_for_dark(r#"<body text="black" bgcolor="white">x</body>"#);
    assert_eq!(out, r##"<body text="#ffffff" bgcolor="#141414">x</body>"##);
}

/// `<style>` blocks are rewritten rule by rule — selectors untouched,
/// declarations adapted, nesting (`@media`) survives.
#[test]
fn style_blocks_are_adapted() {
    let doc = "<style>p{color:black}@media screen{.x{background:white}}</style><p>x</p>";
    let out = adapt_colors_for_dark(doc);
    assert!(out.contains("p{color:#ffffff}"), "{out}");
    assert!(out.contains(".x{background:#141414}"), "{out}");
    assert!(out.contains("@media screen"), "{out}");
}

/// A light render must never let an email's `prefers-color-scheme: dark`
/// rules fire — that is the light-grey-on-white bug. A dark render wants
/// the reverse: dark rules on, light rules off. The surrounding query
/// (`only screen and …`) and the declarations inside must survive.
#[test]
fn color_scheme_queries_follow_the_chosen_ground() {
    let css = "@media only screen and (prefers-color-scheme: dark){.a{color:#9aa0a6}}\
               @media (prefers-color-scheme:light){.b{color:#000}}";
    let light = pin_color_scheme(css, false);
    assert!(
        light.contains("only screen and (min-width:999999px){.a"),
        "dark off: {light}"
    );
    assert!(light.contains("(min-width:0px){.b"), "light on: {light}");

    let dark = pin_color_scheme(css, true);
    assert!(dark.contains("(min-width:0px){.a"), "dark on: {dark}");
    assert!(
        dark.contains("(min-width:999999px){.b"),
        "light off: {dark}"
    );
}

/// The rewrite keys off the real media feature, not the substring: a class
/// or text that merely contains the words is left alone.
#[test]
fn color_scheme_rewrite_ignores_non_feature_text() {
    let doc = "<p>we support prefers-color-scheme detection</p>";
    assert_eq!(pin_color_scheme(doc, false), doc);
    let cls = "<div class=\"prefers-color-scheme-badge\">x</div>";
    assert_eq!(pin_color_scheme(cls, true), cls);
}

/// A data: URL inside a background shorthand contains semicolons and
/// base64 — it must pass through byte-for-byte while the color beside it
/// is still adapted.
#[test]
fn urls_survive_color_adaptation() {
    let doc = r#"<div style="background:url(data:image/png;base64,AAAA//12) #fff">x</div>"#;
    let out = adapt_colors_for_dark(doc);
    assert!(out.contains("url(data:image/png;base64,AAAA//12)"), "{out}");
    assert!(out.contains("#141414"), "{out}");
}

/// !important must survive, and colors in properties we don't understand
/// must be left alone rather than guessed at.
#[test]
fn important_kept_and_unknown_props_untouched() {
    let out = adapt_colors_for_dark(r#"<p style="color:#000 !important">x</p>"#);
    assert!(out.contains("color:#ffffff !important"), "{out}");
    let doc = r#"<p style="border-color:#000;box-shadow:0 0 2px #000">x</p>"#;
    assert_eq!(adapt_colors_for_dark(doc), doc);
}

/// A CSS comment carrying a semicolon must not split the declaration
/// after it — a real newsletter commented out one background and declared
/// another right behind it, and the white slipped through (issue #35).
#[test]
fn comments_with_semicolons_do_not_hide_declarations() {
    let doc = "<style>.t{ /* background-color: #f0f2f5; */ background-color: #fff; }</style>";
    let out = adapt_colors_for_dark(doc);
    assert!(out.contains("background-color: #141414;"), "{out}");
    assert!(
        out.contains("/* background-color: #f0f2f5; */"),
        "comment kept: {out}"
    );
    let out = adapt_colors_for_dark(r#"<p style="/* x; */ color: #000">x</p>"#);
    assert!(out.contains("/* x; */ color: #ffffff"), "{out}");
}

/// Mid-lightness brand colors sit fine on either ground: leave them.
#[test]
fn mid_tones_are_left_alone() {
    let doc = r#"<a style="color:#3584e4;background-color:#26a269">x</a>"#;
    assert_eq!(adapt_colors_for_dark(doc), doc);
}

/// The light path never rewrites anything.
#[test]
fn light_mode_frames_are_untouched() {
    let body = r#"<p style="color:#000">x</p>"#;
    let frame = message_frame(body, true, false, (1, 1), None, "#3584e4", false, 100);
    assert!(frame.contains("color:#000"), "{frame}");
    let frame = message_frame(body, true, true, (1, 1), None, "#3584e4", false, 100);
    assert!(!frame.contains("color:#000"), "{frame}");
}

/// With nothing overridden the frame carries no reader stylesheet.
#[test]
fn reader_style_none_adds_nothing() {
    let frame = message_frame("<p>x</p>", true, false, (1, 1), None, "#3584e4", false, 100);
    assert!(!frame.contains(":not(#tegami-a)"), "{frame}");
    assert_eq!(inject_reader_style("<p>x</p>", ""), "<p>x</p>");
}

/// The sheet goes in before `</body>` (or `</html>`), and at the very end
/// of a bare fragment.
#[test]
fn reader_style_is_injected_last() {
    assert_eq!(
        inject_reader_style("<html><body><p>x</p></body></html>", "p{}"),
        "<html><body><p>x</p><style>p{}</style></body></html>"
    );
    assert_eq!(
        inject_reader_style("<html><p>x</p></HTML>", "p{}"),
        "<html><p>x</p><style>p{}</style></HTML>"
    );
    assert_eq!(
        inject_reader_style("<p>x</p>", "p{}"),
        "<p>x</p><style>p{}</style>"
    );
}
