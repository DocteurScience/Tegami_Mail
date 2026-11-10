//! Small HTML pieces of a card: action buttons, inline icons, recipients,
//! attachment and tag chips, and the banner containers.

use super::html::attr_escape;
use super::types::CardAttachment;
use crate::encoding::escape_html;
use crate::i18n::{i18n, i18n_f, ni18n_f};
use crate::models::Message;
use relm4::prelude::*;

/// A template card's banner (#360): what the message is, and its two uses.
pub(super) fn template_row_html(key: (u32, u32)) -> String {
    let button = |verb: &str, label: &str, title: &str| {
        format!(
            "<button type=\"button\" class=\"tegami-unsub-btn tegami-tpl-btn\" data-key=\"{}:{}\" \
             data-tpl=\"{verb}\" title=\"{}\">{}</button>",
            key.0,
            key.1,
            attr_escape(title),
            escape_html(label),
        )
    };
    format!(
        "<div class=\"tegami-unsub tegami-tpl\"><span class=\"tegami-unsub-text\">{}</span>{}{}</div>",
        escape_html(&i18n("This message is a template.")),
        button(
            "edit",
            &i18n("Edit template"),
            &i18n("Change the template itself")
        ),
        button(
            "use",
            &i18n("Use template"),
            &i18n("Start a new message from a copy of the template")
        ),
    )
}

/// A card's Unsubscribe banner: the container, always emitted so a verdict
/// arriving later can be patched into it (empty, it is hidden by the
/// stylesheet). `inner` is what [`MessageView::unsub_inner_html`] rendered.
pub(super) fn unsub_row_html(key: (u32, u32), inner: &str) -> String {
    format!(
        "<div class=\"tegami-unsub\" data-key=\"{}:{}\">{inner}</div>",
        key.0, key.1
    )
}

/// A card's attachment row (#213): the container, always emitted so a later
/// arrival can be patched into it (an empty row is hidden by the stylesheet).
pub(super) fn att_row_html(key: (u32, u32), atts: Option<&[CardAttachment]>) -> String {
    format!(
        "<div class=\"tegami-atts\" data-key=\"{}:{}\">{}</div>",
        key.0,
        key.1,
        att_chips_html(key, atts.unwrap_or(&[]))
    )
}

/// A card's attachments (#213), as the design's section under the body
/// (docs/DESIGN.md, "Attachments"): a header with how many files there are,
/// their total size and Save All, then a card per file with its type icon
/// in its colour, its name, its type and size, and Save As and More actions
/// buttons, both always shown. A file that may run code says so in words.
/// The icon and the name open the file; every button posts the message key
/// and the attachment's index back. Empty when there are none, so the row
/// takes no room.
pub(super) fn att_chips_html(key: (u32, u32), atts: &[CardAttachment]) -> String {
    use crate::ui::thumbnails::{icon_color_class, icon_for, is_executable, type_label};
    if atts.is_empty() {
        return String::new();
    }
    let (aid, id) = key;
    let attr = |s: String| gtk::glib::markup_escape_text(&s).to_string();
    let n = atts.len();
    let total: u64 = atts.iter().map(|a| a.size).sum();
    let mut out = format!(
        "<div class=\"tegami-atth\"><div><div class=\"tegami-attcount\">{count}</div>\
         <div class=\"tegami-atttotal\">{total}</div></div>\
         <button type=\"button\" class=\"tegami-attall\" data-key=\"{aid}:{id}\" \
         data-idx=\"0\">{dl}<span>{save_all}</span></button></div>\
         <div class=\"tegami-attg\">",
        count = escape_html(&ni18n_f(
            "{n} attachment",
            "{n} attachments",
            n as u32,
            &[("n", &n.to_string())]
        )),
        total = escape_html(&i18n_f(
            "{size} in total",
            &[("size", &crate::models::human_size(total))]
        )),
        dl = inline_icon_svg("folder-download-symbolic"),
        save_all = escape_html(&i18n("Save all\u{2026}")),
    );
    for (idx, a) in atts.iter().enumerate() {
        let warning = if is_executable(&a.name) {
            format!(
                "<div class=\"tegami-attwarn\">{}<span>{}</span></div>",
                inline_icon_svg("dialog-warning-symbolic"),
                escape_html(&i18n("Executable file: open it only if you trust it."))
            )
        } else {
            String::new()
        };
        out.push_str(&format!(
            "<div class=\"tegami-attcard\">\
             <button type=\"button\" class=\"tegami-attc {cls}\" data-key=\"{aid}:{id}\" \
             data-idx=\"{idx}\" title=\"{open}\"><span class=\"tegami-attp\">{preview}</span>\
             <span class=\"tegami-attn\">{name}</span>\
             <span class=\"tegami-attz\">{kind} \u{b7} {size}</span></button>{warning}\
             <div class=\"tegami-attb\">\
             <button type=\"button\" class=\"tegami-attsave\" data-key=\"{aid}:{id}\" \
             data-idx=\"{idx}\" title=\"{save}\" aria-label=\"{save}\">{dl}</button>\
             <button type=\"button\" class=\"tegami-attmore\" data-key=\"{aid}:{id}\" \
             data-idx=\"{idx}\" title=\"{more}\" aria-label=\"{more}\">{dots}</button>\
             </div></div>",
            cls = icon_color_class(&a.name),
            open = attr(i18n("Open")),
            // The picture or the PDF's first page once the app has made it,
            // the type's icon until then (or for any other file). The URI is
            // the app's own PNG, escaped as any attribute is.
            preview = match &a.preview {
                Some(uri) => format!("<img src=\"{}\" alt=\"\">", attr(uri.clone())),
                None => inline_icon_svg(icon_for(&a.name)),
            },
            // The name comes from the sender: text content, escaped like
            // every other header field.
            name = escape_html(&a.name),
            kind = escape_html(&type_label(&a.name)),
            size = crate::models::human_size(a.size),
            save = attr(i18n("Save as\u{2026}")),
            dl = inline_icon_svg("folder-download-symbolic"),
            more = attr(i18n("More actions")),
            dots = inline_icon_svg("view-more-horizontal-symbolic"),
        ));
    }
    out.push_str("</div>");
    out
}

/// The tag chips of a card header (#71): one pill per keyword naming a tag,
/// colored inline (the document has no access to the app's stylesheet), in
/// the list's colours: the dot in the tag's colour, the name in its dark
/// shade on its tint (mockups 02 and 07).
pub(super) fn tag_chips_html(
    tags: &[crate::config::Tag],
    keywords: &[String],
    labels: &[String],
) -> String {
    let chip = |color: &str, name: &str, extra: &str| -> String {
        let (bg, fg) = match crate::config::chip_colors(color) {
            Some((fill, text)) => (fill.to_string(), text.to_string()),
            None => (
                color.to_string(),
                crate::color::readable_text(color).to_string(),
            ),
        };
        let color = gtk::glib::markup_escape_text(color);
        let name = gtk::glib::markup_escape_text(name);
        format!(
            "<span class=\"tegami-tag{extra}\" style=\"background:{bg};color:{fg}\"{title}>\
             <span class=\"tegami-tagdot\" style=\"background:{color}\"></span>{name}</span>",
            title = if extra.is_empty() {
                String::new()
            } else {
                format!(" title=\"{name}\"")
            },
        )
    };
    let mut out = String::new();
    for pill in crate::models::pills(keywords, labels, tags) {
        match pill {
            crate::models::Pill::Tag(t) => out.push_str(&chip(&t.color, &t.name, "")),
            crate::models::Pill::Label(name) => {
                let color = crate::config::TAG_COLORS[crate::models::label_hue(&name)];
                out.push_str(&chip(color, &name, " tegami-label"));
            }
        }
    }
    out
}

/// An embedded symbolic icon's SVG, inlined for the wrapper document (its
/// paths carry no fill, so the document's `fill:currentColor` recolors it);
/// empty when the resource bundle isn't registered (tests).
pub(super) fn inline_icon_svg(icon: &str) -> String {
    let path = format!("/io/github/DocteurScience/TegamiMail/icons/scalable/actions/{icon}.svg");
    gtk::gio::resources_lookup_data(&path, gtk::gio::ResourceLookupFlags::NONE)
        .ok()
        .and_then(|b| String::from_utf8(b.to_vec()).ok())
        .map(|s| {
            s.replacen("<?xml version=\"1.0\" encoding=\"UTF-8\"?>", "", 1)
                .trim()
                .to_string()
        })
        .unwrap_or_default()
}

/// Whether a message body paints a background of its own. The colon/equals
/// forms keep prose that merely mentions "background" from counting.
pub(super) fn paints_own_background(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    [
        "background:",
        "background-color:",
        "background-image:",
        "bgcolor=",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

/// The collapsible To/Cc/Bcc block inside a conversation card's header.
/// Starts hidden; the card's "To:" line toggles it. Empty when the
/// message names no recipients at all. Bcc is there only on the sender's own
/// copy (#476).
pub(super) fn recipients_html(m: &Message) -> String {
    let mut lines = String::new();
    for (label, list) in [
        ("To", m.to.trim()),
        ("Cc", m.cc.trim()),
        ("Bcc", m.bcc.trim()),
    ] {
        if list.is_empty() {
            continue;
        }
        // Each recipient becomes a `.tegami-mail` span (click composes to it,
        // right-click offers copy/new-message), keyed by its bare address.
        let mut parts = String::new();
        for entry in split_addr_list(list) {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let mail = entry
                .rfind('<')
                .and_then(|s| entry[s + 1..].find('>').map(|e| &entry[s + 1..s + 1 + e]))
                .unwrap_or(entry)
                .trim();
            if !parts.is_empty() {
                parts.push_str(", ");
            }
            // A saved contact by its saved name alone (#541): the address
            // is in the right-click menu.
            let shown = crate::contacts::contact_name(mail);
            parts.push_str(&format!(
                "<span class=\"tegami-mail\" data-mail=\"{}\">{}</span>",
                escape_html(mail).replace('"', "&quot;"),
                escape_html(shown.as_deref().unwrap_or(entry))
            ));
        }
        lines.push_str(&format!("<div><b>{label}:</b> {parts}</div>"));
    }
    if lines.is_empty() {
        return String::new();
    }
    // Folded away until the card's To: toggle opens it.
    format!("<div class=\"tegami-rcpt\" hidden>{lines}</div>")
}

/// Who a message went to, for its card's "To:" line (mockup 03): the To
/// and Cc recipients by their saved or written names, each once, the
/// user's own addresses as "me". A copy of one's own sent in Bcc alone
/// names those (#476), or the line, the way to the details, would go.
pub(super) fn to_line_names(m: &Message, is_own: impl Fn(&str) -> bool) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut names: Vec<String> = Vec::new();
    let lists = if m.to.trim().is_empty() && m.cc.trim().is_empty() {
        vec![m.bcc.trim()]
    } else {
        vec![m.to.trim(), m.cc.trim()]
    };
    for list in lists {
        for entry in split_addr_list(list) {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let (written, mail) = match entry.rfind('<') {
                Some(s) => (
                    entry[..s].trim().trim_matches('"').trim(),
                    entry[s + 1..].trim_end_matches('>').trim(),
                ),
                None => ("", entry),
            };
            if !seen.insert(mail.to_ascii_lowercase()) {
                continue;
            }
            names.push(if is_own(mail) {
                i18n("me")
            } else {
                crate::contacts::shown_name_or_addr(written, mail).into_owned()
            });
        }
    }
    names.join(", ")
}

/// Split an address-list header on commas, honouring double-quoted display
/// names (`"Doe, Jane" <j@x>` stays one entry).
pub(super) fn split_addr_list(list: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    for (i, c) in list.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                out.push(&list[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&list[start..]);
    out
}
