//! Right pane: the reading view for a single message.
//!
//! Bodies are rendered in a sandboxed WebKit view: JavaScript is disabled and a
//! Content-Security-Policy blocks remote content. When remote content is
//! withheld, its URLs are also stripped from the HTML so nothing is requested;
//! the originals are only used once the user (or a trusted sender) allows them.
//! Link clicks open in the browser.

mod bridge;
mod card_drag;
mod cards;
mod chrome;
mod component;
mod dark;
mod document;
mod folds;
mod frame;
mod html;
mod init;
mod invite;
mod links;
mod look;
mod menu;
mod on_card_actions;
mod on_cards;
mod on_find;
mod on_prefs;
mod on_services;
mod on_show;
mod page_fade;
mod print;
mod receipt_banner;
mod remote;
mod render;
mod sender;
mod size_script;
mod snippets;
mod style;
mod translation;
mod types;
mod unsub_banner;
mod webview;
mod wrapper_css;

use self::chrome::ClassRules;
use self::page_fade::{PageFade, PendingLoad};
use crate::models::Message;

pub(crate) use frame::theme_grounds_for;
pub(crate) use menu::MenuHit;
pub(crate) use menu::{hit_menu_entries, strip_navigation_items};
pub(crate) use page_fade::{on_next_page_fade, PAGE_FADE_MS};
pub(crate) use remote::remote_urls;
pub(crate) use types::{
    AttVerb, CardAttachment, InviteAction, MessageViewInput, MessageViewOutput, SelectMode,
    TrState, UnsubState,
};
pub(crate) use webview::{keep_tab_out_of_key_translator, new_preview_webview, shared_web_context};

/// Each card's detected language, with the body length it was worked out
/// for.
type DetectedLanguages = std::collections::HashMap<(u32, u32), (usize, Option<String>)>;

pub(crate) struct MessageView {
    /// What each message on screen has attached (#213): names and sizes,
    /// handed over by the app as the files arrive. Listed beneath the card.
    card_atts: std::collections::HashMap<(u32, u32), Vec<CardAttachment>>,
    /// Preference: cards list their own attachments (#213).
    card_atts_shown: bool,
    /// Preference: the attachment drawer is in use. A lone message's files
    /// are listed on its card only when the drawer is not there to show them.
    drawer_on: bool,
    /// The newest message in the thread — drives the header, avatar and actions.
    current: Option<Message>,
    /// The whole conversation, newest first. One entry = a single message (shown
    /// exactly as before); more = a scrollable conversation in the body view.
    thread: Vec<Message>,
    /// Which of the conversation's messages came from another folder, and what to
    /// call that folder.
    folder_labels: std::collections::HashMap<(u32, u32), String>,
    /// The tags (#71): what a message's keywords are drawn as.
    tags: Vec<crate::config::Tag>,
    /// The keywords and Gmail labels the GTK header's chips were last built
    /// for (a lone full-bleed message; cards carry their chips in the
    /// document).
    header_tags_rendered: std::cell::RefCell<Option<(Vec<String>, Vec<String>)>>,
    /// Remote content was detected and is currently withheld — this drives the
    /// "blocked" banner, and nothing else.
    blocked: bool,
    /// Messages the user deliberately marked unread while this conversation was
    /// on screen. They keep their mark but get no scroll sentinel, so simply
    /// having them in view can't undo the thing the user just asked for. Cleared
    /// whenever a conversation is opened afresh.
    no_autoread: std::collections::HashSet<(u32, u32)>,
    /// Message zoom in percent (Ctrl+ / Ctrl-): the bodies alone are
    /// scaled; the chrome keeps its size.
    zoom: u32,
    /// Read-marking policy (#100), stamped on the document for the
    /// viewport observer.
    read_mark: crate::config::ReadMark,
    /// Rows Gmail's x has checked in the list (#392), and whether they are
    /// conversations: the empty pane counts them.
    checked: (usize, bool),
    /// Whether the blocked-content banner is shown at all. It gates only the notice: `blocked`
    /// still governs what is withheld, so hiding it never loads anything.
    show_banner: bool,
    /// Whether the user has actually permitted remote content for what is on
    /// screen (settings, "Load once", or "Always allow sender").
    ///
    /// Kept separate from `blocked` on purpose. `blocked` depends on a detector
    /// guessing whether a message references remote resources; this does not.
    /// Stripping and the CSP key off *this*, so a detector miss costs a banner
    /// rather than the protection itself.
    remote_allowed: bool,
    /// The owning account's name, written above the subject where the list
    /// mixes accounts (`None` elsewhere: the app decides).
    account_name: Option<String>,
    /// Paints the reader's spinner and its inter-document cover in the *message*
    /// theme rather than the app's. Reading a light message in a dark app used
    /// to mean a dark spinner giving way to a white page.
    cover_rules: ClassRules,
    /// The widgets those rules are applied to, by class.
    cover: Option<gtk::Stack>,
    /// Fingerprint of what the WebView is currently showing. Re-selecting a
    /// message that renders to the same document skips the load entirely —
    /// every load blanks the view for an instant, however briefly.
    shown_fingerprint: Option<u64>,
    /// The open conversation has already auto-scrolled to its first unread
    /// message: later renders of the same thread (bodies streaming in, a theme
    /// flip) carry a no-scroll stamp so the reader's place is kept.
    did_autoscroll: bool,
    /// That auto-scroll was spent on a lone message, before the rest of its
    /// conversation (a sent original, an archived part) was found.
    autoscroll_lone: bool,
    /// The wrapper document's last reported scroll anchor: the topmost card at
    /// the viewport top and the offset into it. A re-render replaces the whole
    /// document (scroll resets to 0) and can reflow everything above — an
    /// element anchor survives that where a raw pixel offset lands short.
    saved_anchor: Option<(u32, u32, u32)>,
    /// The list beside the pane shows Drafts: the empty state invites
    /// editing a draft rather than reading a message.
    drafts_view: bool,
    /// Every account's Templates folder, as (account, folder) (#360): a
    /// message from one wears a banner offering to use it or edit it.
    template_folders: std::collections::HashSet<(u32, u32)>,
    /// The saved anchor was set by the app, not by a scroll: the render
    /// lands it below the page's top gutter (where an unscrolled pane has
    /// its first card), not flush at the top the way a place the user
    /// scrolled to is restored.
    anchor_gutter: bool,
    /// The next document goes in without the page crossfade: a sent reply's
    /// card is arriving, and the app's stage covers the pane while it does.
    skip_fade: bool,
    /// What each message's frame measured last time it was shown, so reopening a
    /// conversation lays out right away instead of settling into place.
    frame_heights: std::collections::HashMap<(u32, u32), u32>,
    /// This conversation came ready-made, so there is nothing to cover.
    instant: bool,
    /// The messages the list has selected, outlined in the reader.
    selected_cards: Vec<(u32, u32)>,
    /// True while the body is being fetched (show a spinner instead).
    loading: bool,
    /// False from when a render starts until the WebView reports it finished
    /// loading — a themed cover hides the WebView's white inter-document gap.
    webview_ready: bool,
    /// In-message find (#103).
    find_open: bool,
    find_matches: Option<(u32, u32)>,
    find_entry: Option<gtk::SearchEntry>,
    /// Where the keyboard was when find opened, to hand it back on close:
    /// left to GTK, it fell to the message list, where the next Ctrl+F
    /// searched the list instead of the message (#392).
    find_return: Option<gtk::glib::WeakRef<gtk::Widget>>,
    find_bar: Option<(gtk::Revealer, gtk::Widget)>,
    /// The primary button's press over the reader: a click that selects a
    /// card, or a drag that carries one to a folder.
    pointer: card_drag::Pointer,
    /// The page shown while a message loads, and whether it is holding the
    /// keyboard for the web view meanwhile (#392). A web view whose document
    /// is being replaced drops the keys it is given, so a shortcut pressed
    /// then never arrived; this page lets them through.
    loading_page: Option<gtk::Box>,
    focus_held: std::cell::Cell<bool>,
    /// When find last closed itself (empty entry blur) — the toolbar click
    /// that caused the blur must not instantly reopen it.
    find_closed_at: Option<std::time::Instant>,
    /// The web view the wrapper document (`conversation_document`) is shown
    /// in.
    webview: webkit6::WebView,
    /// Bumped per render: each load gets a unique base URI so WebKit treats it
    /// as a fresh document and re-fetches resources (reusing `about:blank` does
    /// not). An https base also lets https images load without mixed-content.
    seq: std::cell::Cell<u64>,
    /// The document waiting to replace the one on screen (`load_after_paint`).
    pending_load: std::rc::Rc<std::cell::RefCell<PendingLoad>>,
    /// The outgoing page's still, faded out over the incoming one.
    page_fade: PageFade,
    /// Whether the current message's From: address survived its provider's
    /// authentication checks. `None` until the verdict arrives with the body.
    sender_check: Option<crate::models::SenderCheck>,
    /// Per-member verdicts for the conversation on screen, behind each card's
    /// header seal (#88). Keyed (account_id, id); cleared on Show.
    member_checks: std::collections::HashMap<(u32, u32), crate::models::SenderCheck>,
    /// The mailing lists left through the cards' Unsubscribe button, by
    /// [`crate::models::Unsubscribe::key`] with the time of the request, so a
    /// message from one says so instead of offering the button afresh.
    unsubscribed: std::collections::HashMap<String, i64>,
    /// Sent mail's read receipts (#520), for the cards' checkmarks.
    receipt_marks: crate::models::ReceiptMarks,
    /// Cards whose unsubscribe request is under way or has just failed.
    unsub_state: std::collections::HashMap<(u32, u32), UnsubState>,
    /// How many lines of preview a folded card shows under the sender, as
    /// the message list does (#326).
    fold_preview_lines: u32,
    /// A document was loaded while the view was not on screen (#332).
    painted_hidden: bool,
    /// The open scroll went to a document loaded while the view was hidden,
    /// which never painted (#332): the next document loaded on screen is owed
    /// it, unless the reader has a place to keep by then.
    scroll_owed: bool,
    /// The messages on screen filed in Sent, which wear a Sent chip.
    sent: std::collections::HashSet<(u32, u32)>,
    /// Cards opened or folded by hand: these win over the default for as
    /// long as they stay in the conversation on screen.
    folds: std::collections::HashMap<(u32, u32), bool>,
    /// Translations (#327), per card: under way, shown, set aside for the
    /// original, or failed.
    translations: std::collections::HashMap<(u32, u32), TrState>,
    /// Which language each card's message is in, for the Translate offer,
    /// worked out once per body (by its length) rather than on every paint.
    detected: std::cell::RefCell<DetectedLanguages>,
    /// The addresses each account answers to (its own and its aliases), so
    /// an invitation's attendee list can be searched for the reader (#223).
    identities: std::collections::HashMap<u32, Vec<String>>,
    /// The invitations already answered, by [`crate::models::Invite::key`]:
    /// the PARTSTAT sent and when, so a meeting opened again says so.
    invite_answers: std::collections::HashMap<String, (String, i64)>,
    /// Full URL of the link under the pointer, shown in a corner overlay so a
    /// link's real destination is visible before it is clicked.
    link_preview: gtk::Label,
}

#[cfg(test)]
mod tests;

impl Drop for MessageView {
    /// A pop-out's view goes with its window (the reader's own only ever
    /// drops at exit): see [`crate::desktop::memory_report::release_web_view`].
    fn drop(&mut self) {
        crate::desktop::memory_report::release_web_view(&self.webview);
    }
}

/// The card preview already made for these bytes, for a card being built
/// (see `request_card_previews`).
pub(crate) fn cached_card_preview(data: &[u8]) -> Option<String> {
    crate::ui::thumbnails::cached_card_preview(data).flatten()
}

/// One card preview to make: the card's key, the attachment's index on it,
/// its name and its bytes.
pub(crate) type CardPreviewRequest = ((u32, u32), usize, String, Vec<u8>);

/// Make the previews the cards of `view` still lack (docs/DESIGN.md,
/// "Attachments"): each item is a card's key, the attachment's index on it,
/// its name and its bytes. Pictures and PDFs only; each preview is handed to
/// the view as it is ready, the others never are. Already made, it is sent
/// at once.
pub(crate) fn request_card_previews(
    items: Vec<CardPreviewRequest>,
    view: &relm4::Sender<MessageViewInput>,
) {
    for (key, index, name, data) in items {
        if !crate::ui::thumbnails::has_card_preview(&name, &data) {
            continue;
        }
        let view = view.clone();
        crate::ui::thumbnails::card_preview(&name, data, move |uri| {
            if let Some(uri) = uri {
                view.emit(MessageViewInput::SetCardPreview { key, index, uri });
            }
        });
    }
}
