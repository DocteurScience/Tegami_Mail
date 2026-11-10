//! The conversation document: cards, headers, selection and escaping.

use super::*;

/// A conversation is a stack of cards in the order it is handed, so the
/// reader shows the message that started it first and the newest last.
#[test]
fn a_conversation_renders_one_card_per_message_in_order() {
    let mut first = msg_for_print();
    first.body = "<p>opening</p>".into();
    let mut second = msg_for_print();
    second.id = 2;
    second.from_name = "Grace Hopper".into();
    second.body = "<p>reply</p>".into();

    let doc = MessageView::conversation_document(
        &[first, second],
        &std::collections::HashMap::new(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert_eq!(
        doc.matches("<section class=\"tegami-msg\"").count(),
        2,
        "each message gets its own card: {doc}"
    );
    let ada = doc.find("Ada Lovelace").expect("first sender present");
    let grace = doc.find("Grace Hopper").expect("second sender present");
    assert!(ada < grace, "cards must keep the order they were handed");
    assert!(
        doc.contains("<body class=\"tegami-conv tegami-thread\">"),
        "conversation padding, and its tags kept off the cards"
    );
    // The cards sit on the deeper page — the same color the spinner and the
    // cover behind the WebView are painted, so the handover is invisible.
    assert!(
        doc.contains(&format!("background:{}", PAGE.0)),
        "page ground: {doc}"
    );
}

/// Each card names who the message went to on its "To:" line (mockup 03),
/// which opens the collapsed To/Cc block — and a recipient list is header
/// text an attacker controls, so it must land escaped in the trusted
/// wrapper.
#[test]
fn every_card_carries_its_recipients_escaped_and_collapsed() {
    let mut first = msg_for_print();
    first.to = "me@example.com, <script>alert(1)</script>@evil.test".into();
    let mut second = msg_for_print();
    second.id = 2;
    second.cc = String::new();

    let doc = MessageView::conversation_document(
        &[first, second],
        &std::collections::HashMap::new(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert_eq!(
        doc.matches("class=\"tegami-rcpt\" hidden").count(),
        2,
        "each card gets a collapsed To/Cc block: {doc}"
    );
    assert_eq!(
        doc.matches("class=\"tegami-rcpt-toggle tegami-to\"")
            .count(),
        2,
        "each card gets a To line that opens the block: {doc}"
    );
    assert!(doc.contains(">To: "), "the line says To: {doc}");
    assert!(
        !doc.contains("<script>alert(1)</script>"),
        "recipients must be escaped: {doc}"
    );
    assert!(
        doc.contains("&lt;script&gt;"),
        "escaped form present: {doc}"
    );
}

/// A message that names no recipients at all gets neither To line nor block.
#[test]
fn a_card_with_no_recipients_shows_no_to_line() {
    let mut only = msg_for_print();
    only.to = String::new();
    only.cc = " ".into();
    let mut second = msg_for_print();
    second.id = 2;
    let doc = MessageView::conversation_document(
        &[only, second],
        &std::collections::HashMap::new(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert_eq!(
        doc.matches("<button type=\"button\" class=\"tegami-rcpt-toggle tegami-to\"")
            .count(),
        1,
        "only the second card: {doc}"
    );
}

/// Each card carries its own ⋯, keyed to that message, so its menu can't
/// act on the wrong one.
#[test]
fn every_card_carries_its_own_menu_button_keyed_to_that_message() {
    let first = msg_for_print();
    let mut second = msg_for_print();
    second.id = 2;
    let doc = MessageView::conversation_document(
        &[first, second],
        &std::collections::HashMap::new(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert_eq!(
        doc.matches("class=\"tegami-acts-toggle\"").count(),
        2,
        "{doc}"
    );
    // Keyed per message, so the menu can't land on the wrong one.
    assert!(doc.contains("data-key=\"1:1\">"), "{doc}");
    assert!(doc.contains("data-key=\"1:2\">"), "{doc}");
    // The palette that slid out of it is gone.
    assert!(!doc.contains("class=\"tegami-act\""), "{doc}");
}

/// An unread message in a conversation is marked with its dot; reading is
/// click-driven now, so no scroll sentinel exists for any message.
#[test]
fn only_unread_conversation_messages_are_marked() {
    let read = msg_for_print();
    let mut unread = msg_for_print();
    unread.id = 2;
    unread.unread = true;

    let doc = MessageView::conversation_document(
        &[read, unread],
        &std::collections::HashMap::new(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert_eq!(
        doc.matches("class=\"tegami-dot\"").count(),
        1,
        "one dot: {doc}"
    );
    assert_eq!(
        doc.matches("class=\"tegami-end\"").count(),
        0,
        "no sentinel: {doc}"
    );
    // Keyed to the unread message, so reading it can't clear another's.
    assert!(
        doc.contains("class=\"tegami-dot\" data-key=\"1:2\""),
        "{doc}"
    );
}

/// Marking a message unread while its conversation is open must survive the
/// message being in view: it keeps its dot, but no sentinel, so nothing
/// reads it back until the conversation is opened afresh.
#[test]
fn a_deliberately_unread_message_keeps_its_mark() {
    let read = msg_for_print();
    let mut unread = msg_for_print();
    unread.id = 2;
    unread.unread = true;
    let suppressed: std::collections::HashSet<(u32, u32)> = [(1u32, 2u32)].into();

    let doc = MessageView::conversation_document(
        &[read, unread],
        &std::collections::HashMap::new(),
        &suppressed,
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert_eq!(
        doc.matches("class=\"tegami-dot\"").count(),
        1,
        "still marked: {doc}"
    );
    assert_eq!(
        doc.matches("class=\"tegami-end\"").count(),
        0,
        "no sentinel: {doc}"
    );
}

/// A frame opens at the height it had last time, so a reopened conversation
/// lays out on its first frame rather than every card jumping once measured.
#[test]
fn a_known_frame_height_is_used_on_reopen() {
    let a = msg_for_print();
    let mut b = msg_for_print();
    b.id = 2;
    let heights: std::collections::HashMap<(u32, u32), u32> = [((1u32, 2u32), 640u32)].into();

    let doc = MessageView::conversation_document(
        &[a, b],
        &Default::default(),
        &Default::default(),
        &heights,
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert!(
        doc.contains("style=\"height:640px\""),
        "known height used: {doc}"
    );
    // The unmeasured one opens collapsed rather than at the browser's
    // 150px default, so its card shows nothing until its document is in.
    assert_eq!(
        doc.matches("style=\"height:0px\"").count(),
        1,
        "unknown one collapsed: {doc}"
    );
    assert_eq!(doc.matches("style=\"height:").count(), 2, "one each");
    // Keyed per message, so a height can't be applied to the wrong frame.
    assert!(doc.contains("data-key=\"1:2\""), "{doc}");
}

/// A selected message is outlined in the accent color, and only that one.
#[test]
fn a_selected_card_is_outlined_in_the_accent() {
    let a = msg_for_print();
    let mut b = msg_for_print();
    b.id = 2;
    let doc = MessageView::conversation_document(
        &[a, b],
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &[(1, 2)],
        "#ff8800",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert!(
        doc.contains("class=\"tegami-msg selected\" data-key=\"1:2\""),
        "{doc}"
    );
    assert_eq!(
        doc.matches("tegami-msg selected").count(),
        1,
        "only the selected one"
    );
    assert!(
        doc.contains("box-shadow:0 0 0 2px #ff8800"),
        "outlined in the accent: {doc}"
    );
}

/// A lone message gets the same in-document header a conversation card
/// does — its own action icons included, since the toolbar carries no
/// per-message actions — but goes full-bleed: no card gutter, so it reads
/// as a message, not a card.
#[test]
fn a_single_message_keeps_the_header_but_goes_full_bleed() {
    let doc = MessageView::conversation_document(
        &[msg_for_print()],
        &std::collections::HashMap::new(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert!(
        doc.contains("<section class=\"tegami-msg\""),
        "message chrome: {doc}"
    );
    assert!(
        doc.contains("class=\"tegami-msg-hdr\""),
        "in-document header: {doc}"
    );
    assert!(
        !doc.contains("<body class=\"tegami-conv\">"),
        "no card gutter: {doc}"
    );
    assert!(
        doc.contains("class=\"tegami-acts-toggle\""),
        "the card's menu button: {doc}"
    );
    // Full-bleed also means the plain ground, not the cards' deeper page.
    assert!(
        doc.contains(&format!("background:{}", GROUND.0)),
        "plain ground: {doc}"
    );
}

/// With the "single messages as cards" preference on (#57), a lone
/// message renders exactly like a one-message conversation: carded, on
/// the deeper page ground.
#[test]
fn a_single_message_cards_up_when_asked() {
    let doc = MessageView::conversation_document(
        &[msg_for_print()],
        &std::collections::HashMap::new(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        true,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert!(
        doc.contains("<body class=\"tegami-conv\">"),
        "card gutter: {doc}"
    );
    assert!(
        doc.contains(&format!("background:{}", PAGE.0)),
        "deeper page ground: {doc}"
    );
}

#[test]
fn a_message_from_another_folder_is_labelled_with_it() {
    let a = msg_for_print(); // the message on screen
    let mut b = msg_for_print();
    b.id = 2;
    b.folder_id = 3; // pulled in from Sent
    let labels = std::collections::HashMap::from([((1u32, 2u32), "Sent".to_string())]);
    let doc = MessageView::conversation_document(
        &[a, b],
        &labels,
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert_eq!(
        doc.matches("class=\"tegami-folder\"").count(),
        // on the message that came from Sent, and never on the one the
        // reader is already showing the folder of.
        1,
        "only the message from another folder should carry a folder badge"
    );
    assert!(
        doc.contains(">Sent</span>"),
        "the badge should name the folder"
    );
}

#[test]
fn a_folder_label_cannot_smuggle_markup_into_the_header() {
    // Folder names come from the server (a mailbox can be called anything),
    // and this document is the trusted wrapper around the sandboxed frames.
    let a = msg_for_print();
    let mut b = msg_for_print();
    b.id = 2;
    let labels =
        std::collections::HashMap::from([((1u32, 2u32), "<img src=x onerror=alert(1)>".into())]);
    let doc = MessageView::conversation_document(
        &[a, b],
        &labels,
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );
    assert!(
        !doc.contains("<img src=x"),
        "the label must be escaped, not rendered"
    );
    assert!(doc.contains("&lt;img src=x"));
}

#[test]
fn a_sender_cannot_put_markup_in_the_conversation_header() {
    // The wrapper document has JavaScript enabled (it sizes the message
    // iframes) and the iframes are same-origin, so anything that executes
    // here can read every message body in the thread. A `From:` display name
    // is attacker-controlled and RFC 2047-decoded, so it can carry any bytes
    // at all — it has to reach the page as text, never as markup.
    let mut a = msg_for_print();
    a.from_name = "<script>x=1</script>".into();
    a.from_addr = "<img src=y onerror=z>@example.com".into();
    let mut b = msg_for_print();
    b.id = 2;
    // Two messages: the per-message headers only render in conversation mode.
    let doc = MessageView::conversation_document(
        &[a, b],
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &[],
        "#3584e4",
        true,
        false,
        false,
        false,
        &crate::config::ReaderStyle::NONE,
        &Default::default(),
        100,
    );

    assert!(!doc.contains("<script>x=1"), "{doc}");
    assert!(!doc.contains("<img src=y"), "{doc}");
    assert!(doc.contains("&lt;script&gt;x=1&lt;/script&gt;"), "{doc}");
    assert!(doc.contains("onerror=z&gt;"), "{doc}");
}
