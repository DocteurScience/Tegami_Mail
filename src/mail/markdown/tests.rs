use super::dom::plain_text;
use super::outgoing::sanitize_outgoing;
use super::writer::{from_html, html_to_text};

/// A preview is the body's words: no head, no styles, and a `<header>`
/// element is content like any other.
#[test]
fn plain_text_keeps_the_words_and_the_breaks() {
    let html = "<html><head><title>T</title><style>p{}</style></head><body>\
                <header>Hello   <b>there</b></header><p>One<br>two &amp; three</p>\
                <p></p><p></p><div>After</div><pre>a  b\n c</pre></body></html>";
    assert_eq!(
        plain_text(html),
        "Hello there\nOne\ntwo & three\n\nAfter\na  b\nc"
    );
}

/// Outlook nests a link in a link; a browser closes the first, leaving
/// it empty. Only the one with text is written.
#[test]
fn nested_links_leave_no_empty_link() {
    let html = r#"<p>Em: <a href="mailto:j@x.example"><a href="mailto:j@x.example">j@x.example</a></a></p>"#;
    assert_eq!(from_html(html), "Em: j@x.example");
    assert_eq!(
        from_html(r#"<p><a href="https://x.example/"></a>text</p>"#),
        "text"
    );
}

/// Assert `html` contains `needle`, with the whole rendering in the
/// failure so a broken case reads itself.
fn has(html: &str, needle: &str) {
    assert!(html.contains(needle), "expected {needle:?} in:\n{html}");
}

#[test]
fn converts_a_composed_reply() {
    // The shape the rich editor hands over for a reply.
    let html = "<div>Yes, agreed.</div><p class=\"tegami-quote-attr\">On Monday, Ada wrote:</p>\
                <blockquote><div>The original line.</div><div>And another.</div></blockquote>";
    let md = from_html(html);
    has(&md, "Yes, agreed.");
    has(&md, "> The original line.");
    has(&md, "> And another.");
}

#[test]
fn keeps_an_inline_image_and_its_name() {
    let md = from_html("<p><img src=\"data:image/png;base64,AAA\" alt=\"shot.png\"></p>");
    has(&md, "![shot.png](data:image/png;base64,AAA)");
}

#[test]
fn entities_and_breaks() {
    assert_eq!(from_html("<p>a &amp; b &lt;c&gt;</p>"), r"a & b \<c>");
    has(&from_html("<p>one<br>two</p>"), "one  \ntwo");
}

#[test]
fn a_bare_link_reads_as_itself() {
    assert_eq!(
        from_html("<p><a href=\"https://example.com\">https://example.com</a></p>"),
        "https://example.com"
    );
    assert_eq!(
        from_html("<p><a href=\"mailto:ada@example.com\">ada@example.com</a></p>"),
        "ada@example.com"
    );
}

#[test]
fn nested_lists_indent_under_their_marker() {
    let md = from_html("<ul><li>one<ul><li>inner</li></ul></li><li>two</li></ul>");
    has(&md, "- one");
    has(&md, "  - inner");
    has(&md, "- two");
}

#[test]
fn scripts_and_styles_are_not_text() {
    let md = from_html(
        "<p>before</p><script>alert('x')</script><style>p{color:red}</style><p>after</p>",
    );
    assert!(!md.contains("alert"), "{md}");
    assert!(!md.contains("color:red"), "{md}");
    has(&md, "before");
    has(&md, "after");
}

#[test]
fn unclosed_tags_survive() {
    let md = from_html("<p>one<p>two<div>three");
    has(&md, "one");
    has(&md, "two");
    has(&md, "three");
}

// -- Outgoing HTML ----------------------------------------------------

#[test]
fn sanitizer_keeps_mail_html_and_drops_the_rest() {
    let dirty = "<table style=\"border-collapse:collapse\"><tr><td bgcolor=\"#eee\">cell</td></tr></table>\
                 <img src=\"cid:pic\" alt=\"x\"><script>alert(1)</script><p onclick=\"go()\">text</p>\
                 <style>p{color:red}</style><mark>hi</mark>";
    let clean = sanitize_outgoing(dirty);
    has(&clean, "<table");
    has(&clean, "border-collapse");
    has(&clean, "bgcolor=\"#eee\"");
    has(&clean, "cid:pic");
    has(&clean, "<mark>hi</mark>");
    assert!(!clean.contains("script"), "{clean}");
    assert!(!clean.contains("onclick"), "{clean}");
    assert!(!clean.contains("color:red"), "{clean}");
}

#[test]
fn an_indent_is_not_a_quote() {
    // The composer's Indent (#358) keeps its words, without the "> " a
    // quote gets.
    let html = "<blockquote style=\"margin: 0 0 0 40px; border: none; padding: 0px;\"><p>tail</p></blockquote>\
                <blockquote><p>quoted</p></blockquote>";
    assert_eq!(html_to_text(html), "tail\n\n> quoted");
}

#[test]
fn pinned_links_and_images_are_written_unescaped() {
    assert_eq!(
        from_html("<p><a href=\"http://x.y/a b)\">t</a></p>"),
        "[t](http://x.y/a b))"
    );
    assert_eq!(
        from_html("<p><img alt=\"a]\" src=\"x y.png\"></p>"),
        "![a]](x y.png)"
    );
}

#[test]
fn pinned_block_markers_in_text_are_not_escaped() {
    assert_eq!(
        from_html("<p># not heading</p><p>1. not list</p><p>&gt; quote</p><p>- dash</p>"),
        "# not heading\n\n1. not list\n\n> quote\n\n- dash"
    );
    assert_eq!(
        from_html("<p>:tada: ==x== H~2~O x^2^ a_b_c</p>"),
        ":tada: ==x== H~2~O x^2^ a_b_c"
    );
}

#[test]
fn pinned_inline_markup_characters_are_backslashed() {
    let html = "<p>a*b [c] \\ d `e` &lt;f&gt;</p>";
    assert_eq!(from_html(html), "a\\*b \\[c\\] \\\\ d \\`e\\` \\<f>");
    assert_eq!(html_to_text(html), from_html(html));
}

#[test]
fn pinned_plain_text_trims_no_break_spaces() {
    assert_eq!(
        plain_text("<p>&nbsp;a&nbsp;&nbsp;b&nbsp;</p><p>c</p>"),
        "a b\n\nc"
    );
    assert_eq!(
        plain_text("<div>\u{a0}</div><p>x</p><br><br><br><p>y</p>"),
        "x\n\ny"
    );
}
