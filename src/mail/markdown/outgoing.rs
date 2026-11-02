//! The HTML that goes on the wire.

/// The sanitizer HTML the user sends on is put through on its way out (a
/// filter's automatic forward, which quotes the message it passes on).
///
/// It is the user's own material, so this is not a defence against them —
/// it is a defence for them. A `<script>` that reaches a recipient will at
/// best be stripped by their client and at worst put the message in the
/// spam folder, and a `<style>` block restyles whatever it lands in. The
/// structure mail actually needs — tables, inline styles, images, the tags
/// rich text uses — all survives.
pub(crate) fn sanitize_outgoing(html: &str) -> String {
    use std::collections::HashSet;
    let mut b = ammonia::Builder::default();
    b.add_tags([
        "table",
        "thead",
        "tbody",
        "tfoot",
        "tr",
        "td",
        "th",
        "font",
        "center",
        "u",
        "span",
        "img",
        "mark",
        "sup",
        "sub",
        "del",
        "ins",
        "kbd",
        "samp",
        "var",
        "dl",
        "dt",
        "dd",
        "figure",
        "figcaption",
        "input",
        "section",
        "article",
        "details",
        "summary",
        "small",
        "big",
    ])
    .add_generic_attributes([
        "style",
        "class",
        "id",
        "align",
        "valign",
        "width",
        "height",
        "bgcolor",
        "border",
        "cellpadding",
        "cellspacing",
        "color",
        "face",
        "size",
        "colspan",
        "rowspan",
        "start",
        "dir",
        "lang",
    ])
    .add_tag_attributes("img", ["src", "alt", "width", "height", "title"])
    // Task lists: the box is an <input>, and a disabled one carries no
    // behaviour of any kind.
    .add_tag_attributes("input", ["type", "checked", "disabled"])
    .url_schemes(HashSet::from([
        "http", "https", "mailto", "tel", "data", "cid",
    ]))
    .link_rel(None);
    b.clean(html).to_string()
}
