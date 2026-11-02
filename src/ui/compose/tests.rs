use crate::contacts::{SuggestedGroup, Suggestion};

fn suggestion(name: &str, email: &str, score: u32) -> Suggestion {
    Suggestion {
        name: name.into(),
        email: email.into(),
        score,
        ..Default::default()
    }
}

#[test]
fn initial_suggestions_keep_addresses_added_while_loading() {
    let mut current = vec![
        suggestion("Ann", "ANN@example.org", 2),
        suggestion("Bob", "bob@example.org", 1),
    ];
    let mut contact = suggestion("Ann Contact", "ann@example.org", 5);
    contact.from_contacts = true;
    contact.own = true;
    let mut group = suggestion("Book Club", "", 0);
    group.group = Some(std::sync::Arc::new(SuggestedGroup {
        members: Vec::new(),
        hide_addresses: true,
    }));
    super::suggest::merge_loaded_suggestions(&mut current, vec![contact, group]);
    assert_eq!(current.len(), 3);
    let ann = &current[0];
    assert_eq!(ann.name, "Ann Contact");
    assert_eq!(ann.score, 5);
    assert!(ann.from_contacts && ann.own);
    assert!(current[1].group.is_some());
    assert_eq!(current[2].email, "bob@example.org");
}

#[test]
fn initial_suggestions_keep_newer_names_and_frequency() {
    let mut current = vec![suggestion("Ann", "ann@example.org", 3)];
    super::suggest::merge_loaded_suggestions(
        &mut current,
        vec![suggestion("ann@example.org", "ann@example.org", 1)],
    );
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].name, "Ann");
    assert_eq!(current[0].score, 3);
    super::suggest::merge_loaded_suggestions(&mut current, Vec::new());
    assert_eq!(current[0].name, "Ann");
}

use super::signature::sig_html;

#[test]
fn the_separator_line_follows_the_setting() {
    assert_eq!(
        sig_html("Ann", false),
        "<div class=\"tegami-sig\"><br>Ann</div>"
    );
    assert_eq!(
        sig_html("Ann", true),
        "<div class=\"tegami-sig\"><br>-- <br>Ann</div>"
    );
}

// Characterization tests (restructure, area D).

#[test]
fn pinned_froms_compare_by_name_and_address() {
    assert!(super::send::same_from(
        "Ann <ann@x.org>",
        "\"Ann\" <ANN@x.org>"
    ));
    assert!(super::send::same_from(
        "Ann <ann@x.org>",
        "Ann  <ann@x.org>"
    ));
    assert!(!super::send::same_from("Ann <ann@x.org>", "ann@x.org"));
    assert!(!super::send::same_from("", ""));
}

#[test]
fn pinned_attachment_labels() {
    let pdf = std::path::Path::new("/a/b.pdf");
    assert_eq!(
        super::attachments::attachment_label("Attach {name}", pdf, 1),
        "Attach b.pdf"
    );
    assert_eq!(
        super::attachments::attachment_label("Attach {name}", pdf, 3),
        "Attach 3 attachments"
    );
    assert_eq!(
        super::attachments::attachment_label("Attach {name}", std::path::Path::new("/"), 1),
        "Attach "
    );
}

#[test]
fn pinned_format_icons() {
    use crate::config::ComposeFormat;
    assert_eq!(
        super::format::format_icon(ComposeFormat::Rich),
        "format-text-rich-symbolic"
    );
    assert_eq!(
        super::format::format_icon(ComposeFormat::Plain),
        "text-x-generic-symbolic"
    );
}

#[test]
fn pinned_key_tables() {
    use gtk::gdk::{Key, ModifierType as M};
    let cs = M::CONTROL_MASK | M::SHIFT_MASK;
    assert!(matches!(
        super::keys::composer_key(Key::A, cs),
        Some(super::ComposeInput::AttachFiles)
    ));
    assert!(matches!(
        super::keys::composer_key(Key::t, cs),
        Some(super::ComposeInput::FocusField(super::types::Field::To))
    ));
    assert!(matches!(
        super::keys::composer_key(Key::c, cs),
        Some(super::ComposeInput::FocusField(super::types::Field::Cc))
    ));
    assert!(matches!(
        super::keys::composer_key(Key::b, cs),
        Some(super::ComposeInput::FocusField(super::types::Field::Bcc))
    ));
    assert!(super::keys::composer_key(Key::a, M::CONTROL_MASK).is_none());
    assert!(super::keys::composer_key(Key::a, cs | M::ALT_MASK).is_none());
    assert!(super::keys::composer_key(Key::x, cs).is_none());

    let c = M::CONTROL_MASK;
    for (key, step) in [
        (Key::plus, 1),
        (Key::equal, 1),
        (Key::KP_Add, 1),
        (Key::minus, -1),
        (Key::KP_Subtract, -1),
        (Key::_0, 0),
        (Key::KP_0, 0),
    ] {
        assert_eq!(super::keys::zoom_key(key, c), Some(step), "{key:?}");
        assert_eq!(super::keys::zoom_key(key, c | M::SHIFT_MASK), Some(step));
        assert_eq!(super::keys::zoom_key(key, c | M::ALT_MASK), None);
        assert_eq!(super::keys::zoom_key(key, M::empty()), None);
    }
    assert_eq!(super::keys::zoom_key(Key::_1, c), None);
}

/// The translate scripts are pasted into the page as they are: pin their
/// bytes so moving them cannot change them.
#[test]
fn pinned_translate_scripts_are_unchanged() {
    fn fnv(s: &str) -> (usize, u64) {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in s.bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x100000001b3);
        }
        (s.len(), h)
    }
    assert_eq!(
        fnv(super::format::SHOWCASE_PLAIN_LOG),
        (396, 0x09552826f050be02)
    );
    assert_eq!(
        fnv(super::translate::TR_QUOTE_TEXT),
        (154, 0xdbd6a7efb7080808)
    );
    assert_eq!(
        fnv(super::translate::TR_RICH_BEGIN),
        (1704, 0x434261e8ae315f2c)
    );
    assert_eq!(
        fnv(super::translate::TR_RICH_END),
        (743, 0x111f1e3ff5268947)
    );
    assert_eq!(fnv(super::translate::TR_CANCEL), (106, 0x2868c0588530129));
}
