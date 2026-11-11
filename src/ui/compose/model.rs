//! The composer's model: everything one open composer holds.

use super::attachments::AttachThumbs;
use super::types::{CloudLink, ComposeAccount, ComposeUndoEntry, Field, OwnMail};
use crate::config::{ComposeFormat, SignaturePosition};
use crate::contacts::Suggestion;
use crate::models::DraftOrigin;
use crate::ui::drop_zones::DropZones;
use crate::ui::rich_editor::RichEditor;

pub(crate) struct Compose {
    pub(super) accounts: Vec<ComposeAccount>,
    /// The rich-text (HTML) body editor.
    pub(super) editor: RichEditor,
    /// Signature currently appended to the body (so it can be swapped out).
    pub(super) current_sig: String,
    /// Whether signatures go in under a `-- ` line (Settings, read when the
    /// composer opens).
    pub(super) sig_dashes: bool,
    /// Where that signature sits against a quoted original (#237), so a
    /// signature added on an account switch lands in the same place.
    pub(super) signature_position: SignaturePosition,
    /// Files to attach.
    pub(super) attachments: Vec<std::path::PathBuf>,
    /// Thumbnails of the attached pictures and PDFs by path (#299), kept
    /// so a chip rebuild does not read and decode the file again. `None`
    /// is a file that would not decode.
    pub(super) attach_thumbs: AttachThumbs,
    /// The surfaces shown while files are dragged over the composer, set up
    /// once the view exists.
    pub(super) drop_zones: Option<std::rc::Rc<DropZones>>,
    /// When editing a queued Outbox message, the row this replaces once sent.
    pub(super) outbox_origin: Option<u32>,
    /// Recipient suggestions, filtered as the user types.
    pub(super) suggestions: Vec<Suggestion>,
    /// The Microsoft 365 accounts whose organization directory is searched
    /// too (#410), found once the composer opens. The LDAP directories are
    /// not: one switched on in Evolution while the composer is open counts
    /// as well (#307), and [`crate::contacts::directory::search`] keeps the list.
    pub(super) graph_directories: std::sync::Arc<Vec<crate::config::AccountConfig>>,
    /// The accounts that suggest only from their directory and their own
    /// mail (#410), with the addresses in that mail.
    pub(super) own_mail: OwnMail,
    /// The account the From row has picked, which decides whether
    /// `own_mail` narrows the suggestions.
    pub(super) from_account: u32,
    /// What the directories answered for the recipient being typed.
    pub(super) directory_matches: Vec<Suggestion>,
    /// Bumped on every keystroke, so a slow directory's answer to an
    /// earlier fragment is dropped.
    pub(super) directory_generation: u64,
    /// The directory failures this composer has put in the status bar. A
    /// directory that keeps failing says so once here, not at every pause
    /// in the typing, and still once in the next composer (#307).
    pub(super) directory_errors_shown: std::collections::HashSet<String>,
    /// Shared autocomplete popover and which field it's currently attached to.
    pub(super) completion: gtk::Popover,
    pub(super) completion_field: Option<Field>,
    /// The list inside the popover + the keyboard-highlighted row, for arrow-key nav.
    pub(super) completion_list: Option<gtk::ListBox>,
    pub(super) completion_selected: usize,
    pub(super) completion_count: usize,
    /// Whether the popover is showing (read synchronously by the key handler).
    pub(super) completion_open: std::rc::Rc<std::cell::Cell<bool>>,
    /// Threading headers carried from the message being replied to.
    pub(super) in_reply_to: String,
    pub(super) references: String,
    pub(super) forwards: String,
    /// When editing an existing draft, its origin (replaced on save/send).
    pub(super) draft_origin: Option<DraftOrigin>,
    /// Editing a template (#360), kept at `draft_origin`: Save puts it back,
    /// and Send leaves it where it is.
    pub(super) template: bool,
    /// Stable id the app uses to track this composer across host moves.
    pub(super) compose_id: u32,
    /// The identity the message is sent from: an index into `accounts`,
    /// picked in the From button's Send from menu.
    pub(super) from_index: usize,
    /// A From that is none of the identities, kept as it came (#347): a
    /// reply to a `+tag` address of one, or a draft saved from one. The
    /// From button shows it and it is sent as is; picking an identity
    /// drops it. It can no longer be typed (owner's decision,
    /// docs/SUIVI.md).
    pub(super) custom_from: Option<String>,
    /// The From button and its face, made by `fill_from_row`.
    pub(super) from_face: Option<super::send::FromFace>,
    /// The pane is too narrow for the full toolbar: everything but Cancel,
    /// Send and the fields chevron folds into the ⋯ menu (like the reader's
    /// header), so the window controls at the end never leave the canvas.
    pub(super) narrow: bool,
    /// A recipient/subject field was edited since open (body edits are tracked
    /// separately by the editor itself). Used for save-if-dirty.
    pub(super) fields_dirty: bool,
    /// The save-or-discard question is on screen (or about to be), so a
    /// second Escape in the moment before it appears asks nothing twice.
    pub(super) asking_discard: bool,
    /// OpenPGP (#133): sign the message; encrypt it to every recipient.
    pub(super) sign: bool,
    pub(super) encrypt: bool,
    /// Ask for a read receipt (#520).
    pub(super) request_receipt: bool,
    /// The OpenPGP send check is running on a worker thread.
    pub(super) pgp_checking: bool,
    /// What the last passing OpenPGP check covered (From, key, encrypt,
    /// recipients); a Send with the same passes straight on.
    pub(super) pgp_cleared: Option<Vec<String>>,
    /// Sign was set by hand, so it no longer follows the From account's
    /// default (#267).
    pub(super) sign_touched: bool,
    /// The state the composer itself last put the Sign toggle in, until its
    /// `toggled` arrives: that one is not the user's.
    pub(super) sign_expected: Option<bool>,
    /// What this message is written in. Plain text hides the
    /// formatting toolbar and sends no HTML part (#180).
    pub(super) format: ComposeFormat,
    /// The body's zoom, shown at the start of the format bar while it is
    /// not 100%; see [`super::format::zoom_chip`].
    pub(super) zoom_chip: gtk::Label,
    /// The preview toggle, at the end of the editor's formatting row.
    /// Translate what is being written (#327), beside the format chooser;
    /// shown only with a translation service set up.
    pub(super) translate_btn: gtk::Button,
    /// Insert Signature (#385), beside Translate: Edit as New leaves the
    /// signature out, and this puts it back by hand. With several (#445)
    /// it lists them. Shown only while the From account has one.
    pub(super) signature_btn: gtk::Button,
    /// A translation is out: the button waits and the body is locked.
    pub(super) translating: bool,
    /// Saving on its own (#340): a save on its way, what the last one held
    /// (a hash of the fields and body), whether the Drafts copy is one only
    /// these saves made (so Discard removes it), and whether they have
    /// stopped (the copy could not be found again to replace it).
    pub(super) autosaving: bool,
    pub(super) autosaved: Option<u64>,
    pub(super) autosave_pending: Option<u64>,
    pub(super) autosave_made: bool,
    pub(super) autosave_off: bool,
    /// The person was asked about a mentioned but missing attachment and
    /// chose to send anyway, or nothing was mentioned: the next Send goes
    /// through without asking (docs/DESIGN.md, "Attachments").
    pub(super) attach_check_passed: bool,
    /// Attached files on their way to the cloud through Send a Link: each
    /// is taken off the message once its link is in, and stays if the
    /// upload fails or is called off.
    pub(super) link_instead: Vec<std::path::PathBuf>,
    /// The header bar's title: "New Message", `"Reply to <name>"` or
    /// "Forward" (docs/DESIGN.md, "Replying and composing").
    pub(super) heading: String,
    /// When the draft was last saved on its own, for the header bar's
    /// `"Draft saved at <time>"`: the save happens unseen, and the line is
    /// how the person knows it did.
    pub(super) saved_at: Option<i64>,
    /// The subject as written and as translated with the body, so Show
    /// Original and Show Translation can swap it back and forth (#327).
    pub(super) tr_subject: Option<(String, String)>,
    /// Cloud attachments (#144): the accounts files can be uploaded to, the
    /// links already placed in the body, and how many uploads are running.
    pub(super) cloud_accounts: Vec<crate::services::cloud::CloudAccount>,
    pub(super) cloud_links: Vec<CloudLink>,
    pub(super) cloud_busy: u32,
    /// Download passwords made for this message's links, to pass on
    /// separately: (file name, password).
    pub(super) cloud_passwords: Vec<(String, String)>,
    /// The composer's own undo history (#200): the body's typing and
    /// formatting, and the attachments alongside it. Ctrl+Z takes the top
    /// one, applies it, and puts what reverses it onto `redo_stack` — the
    /// same shape as the message list's history. See
    /// [`Compose::push_history`] for the one place the two kinds of change
    /// are ordered against each other.
    pub(super) undo_stack: Vec<ComposeUndoEntry>,
    pub(super) redo_stack: Vec<ComposeUndoEntry>,
}
