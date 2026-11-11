//! What is injected into each message document: a stylesheet of ours (the
//! zoom), the Content-Security-Policy and the text direction.

/// Put a stylesheet at the end of a document's body, after everything the
/// sender wrote, so its rules are the last word.
pub(super) fn inject_reader_style(doc: &str, css: &str) -> String {
    if css.is_empty() {
        return doc.to_string();
    }
    let block = format!("<style>{css}</style>");
    let lower = doc.to_ascii_lowercase();
    for close in ["</body", "</html"] {
        if let Some(at) = lower.rfind(close) {
            return format!("{}{block}{}", &doc[..at], &doc[at..]);
        }
    }
    format!("{doc}{block}")
}

/// Where a document's real `tag` opens ("<body", say), skipping longer
/// names that merely start the same ("<bodyguard"). `lower` is the
/// document lowercased, so offsets match.
fn find_tag(lower: &str, tag: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(at) = lower[from..].find(tag).map(|i| from + i) {
        let next = lower
            .as_bytes()
            .get(at + tag.len())
            .copied()
            .unwrap_or(b'>');
        if next == b'>' || next == b'/' || next.is_ascii_whitespace() {
            return Some(at);
        }
        from = at + tag.len();
    }
    None
}

/// Whether the tag opening at `at` carries a `dir` attribute.
fn tag_has_dir(lower: &str, at: usize) -> bool {
    let end = lower[at..].find('>').map_or(lower.len(), |i| at + i);
    let tag = &lower[at..end];
    tag.match_indices("dir").any(|(i, _)| {
        tag[..i].ends_with(|c: char| c.is_ascii_whitespace())
            && tag[i + 3..].trim_start().starts_with('=')
    })
}

/// Let a message take the direction of its own text (#366): `dir="auto"`
/// on its body, so Persian or Hebrew mail reads right to left whatever the
/// interface language, unless the sender set a direction on `<html>` or
/// `<body>`, which stands. Plain text is set line by line (`plaintext`
/// bidi), so an English reply over a Persian quote reads right in both.
pub(super) fn auto_direction(doc: &str) -> String {
    let lower = doc.to_ascii_lowercase();
    let html = find_tag(&lower, "<html");
    let body = find_tag(&lower, "<body");
    let doc = if html.is_some_and(|at| tag_has_dir(&lower, at))
        || body.is_some_and(|at| tag_has_dir(&lower, at))
    {
        doc.to_string()
    } else if let Some(at) = body.or(html) {
        // Both "<body" and "<html" are five bytes long.
        let at = at + 5;
        format!("{} dir=\"auto\"{}", &doc[..at], &doc[at..])
    } else {
        doc.to_string()
    };
    let plain = if crate::mail::quote::body_is_pre_wrap(&doc) {
        "body,.tegami-plain"
    } else if lower.contains("tegami-plain") {
        ".tegami-plain"
    } else {
        return doc;
    };
    inject_reader_style(&doc, &format!("{plain}{{unicode-bidi:plaintext;}}"))
}

/// Inject a Content-Security-Policy `<meta>` into the document head. When remote
/// content is disallowed only inline styles and `data:` URIs are permitted.
///
/// `allow_remote` is the user's own choice, never the output of
/// `has_remote_resources`. That is what makes this an independent second line
/// of defence: if the detector fails to spot a reference, the engine still
/// refuses the fetch.
pub(super) fn inject_csp(html: &str, allow_remote: bool, dark: bool) -> String {
    let policy = if allow_remote {
        "default-src 'none'; img-src http: https: tegami-http: tegami-https: data: cid:; \
         style-src 'unsafe-inline' http: https: tegami-http: tegami-https: data:; \
         font-src http: https: tegami-http: tegami-https: data:; \
         media-src http: https: tegami-http: tegami-https: data:"
    } else {
        "default-src 'none'; img-src data: cid:; style-src 'unsafe-inline' data:; \
         font-src data:; media-src data:"
    };
    let lower = html.to_ascii_lowercase();
    // Every message gets a comfortable default inset: the UA's 8px body margin
    // is reset so content sits at exactly 20px of breathing room. Injected
    // ahead of the email's own CSS, so a message that styles its body (a
    // full-bleed design, say) still wins.
    let body_pad = "body{margin:0;padding:20px;box-sizing:border-box;}";
    // An image with no size of its own (a photo sent from a phone is a bare
    // `<img>`) would otherwise lay out at its pixel width, and the frame
    // widens to whatever its content needs, so the card ran off the pane.
    // Capped at the card instead; a message that sizes its images keeps its
    // own rules, which come later. No `height:auto`: that would stretch a
    // spacer image whose `height` attribute is all that keeps it thin.
    let image_fit = "img{max-width:100%;}";
    // The frame's own document must never scroll: WebKit latches a wheel
    // gesture to the innermost scrollable area under the pointer, and a
    // frame document that can scroll sideways (a message wider than the
    // pane whose width grows as the frame is widened, so the widening
    // never quite catches it) swallows the vertical deltas the page
    // needed. The wrapper widens the frame to the content and scrolls it
    // in `.tegami-pan`; whatever is left over is clipped rather than
    // scrollable. `scrollWidth`/`scrollHeight` still measure the content.
    // `color-scheme` makes the browser's default colors (for content that sets
    // none of its own) follow the app's light/dark setting; styled emails keep
    // their own colors untouched.
    let scheme = if dark { "dark" } else { "light" };
    let supported = if dark { "dark light" } else { "light dark" };
    let theme = format!(
        "<meta name=\"color-scheme\" content=\"{supported}\">\
         <style>:root{{color-scheme:{scheme};}}{body_pad}{image_fit}\
         html{{overflow:hidden !important;}}\
         @media print{{:root{{color-scheme:light;}}html{{overflow:visible !important;}}html,body{{background:#fff !important;}}}}\
         </style>"
    );
    // `no-referrer` keeps the synthetic `tegami.localhost` base URI from leaking as
    // a Referer/Origin header — both for privacy and because hotlink-protected
    // servers (e.g. some DreamHost sites) return 403 to foreign referrers, which
    // otherwise blocks legitimate images even once the sender is trusted.
    let meta = format!(
        "{theme}<meta name=\"referrer\" content=\"no-referrer\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"{policy}\">"
    );

    if let Some(head) = lower.find("<head") {
        if let Some(close) = html[head..].find('>') {
            let at = head + close + 1;
            return format!("{}{meta}{}", &html[..at], &html[at..]);
        }
    }
    if let Some(htmltag) = lower.find("<html") {
        if let Some(close) = html[htmltag..].find('>') {
            let at = htmltag + close + 1;
            return format!("{}<head>{meta}</head>{}", &html[..at], &html[at..]);
        }
    }
    format!("<!doctype html><html><head>{meta}</head><body>{html}</body></html>")
}
