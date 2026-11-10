//! privacy.toml: every app preference in one file, its defaults for a new
//! install and for an upgrade, and the one-time migrations applied when it
//! is read.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::look::{default_list_columns, AppTheme, ReadingPane};
use super::prefs::{
    default_files_limit_mb, default_undo_send_secs, ComposeFormat, FilesAction, FilesLarge,
    LauncherBadge, NotificationButtons, PrintOptions, ReadMark, ReceiptPolicy, ReturnedReceipts,
    SignaturePosition,
};
use super::sections::{FolderSort, StartView};
use super::{config_file, default_on, is_zero, save_private_toml};

/// Every preference privacy.toml holds. The name is historical: the file
/// began as the remote-content allowlist (`allowed_senders`).
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PrivacyFile {
    #[serde(default)]
    pub(crate) allowed_senders: Vec<String>,
    /// Whether remote content (images, trackers) is auto-loaded for every new
    /// message, not just those from allowed senders. Off by default, since
    /// remote content can be used to track when and where a message is read.
    #[serde(default)]
    pub(crate) auto_remote_content: bool,
    /// Whether to load sender avatars from Gravatar (off by default — it sends
    /// a hash of each sender's email to a third party).
    #[serde(default)]
    pub(crate) gravatar: bool,
    /// Whether the colored avatars are drawn in the message list and the
    /// reader (#29 — they cost horizontal room on a small screen).
    #[serde(default = "default_on")]
    pub(crate) avatars: bool,
    /// Whether a sender's site icon may be fetched to fill their circle (#30).
    #[serde(default)]
    pub(crate) sender_logos: bool,
    /// The address book new contacts go to: an EDS source UID, or the
    /// former Tegami Mail book's, which is no longer offered. Empty picks the
    /// first writable EDS book.
    #[serde(default)]
    pub(crate) contact_book: String,
    /// Seconds between automatic mail checks; 0 = manual only.
    #[serde(default = "default_fetch_interval")]
    pub(crate) fetch_interval_secs: u64,
    /// Whether to use IMAP IDLE push for instant new-mail delivery.
    #[serde(default = "default_on")]
    pub(crate) push: bool,
    /// Addresses or whole domains whose incoming mail is auto-deleted (to Trash).
    /// Stored lowercased; a bare domain like "spam.com" matches any sender there.
    #[serde(default)]
    pub(crate) blacklist: Vec<String>,
    /// Group messages into conversation threads in the list.
    #[serde(default = "default_on")]
    pub(crate) threading: bool,
    /// Whether the reading pane shows a conversation newest-message-first.
    #[serde(default)]
    pub(crate) thread_newest_first: bool,
    /// Whether deleting a whole selected conversation asks for confirmation
    /// first.
    #[serde(default = "default_on")]
    pub(crate) confirm_thread_delete: bool,
    /// Whether to post desktop notifications (new mail, error alerts).
    #[serde(default = "default_on")]
    pub(crate) notifications: bool,
    /// Whether new-mail notifications name the sender and subject. On by
    /// default — that is what makes them useful — but GNOME draws notifications
    /// on the lock screen, so turning it off is worth offering.
    #[serde(default = "default_on")]
    pub(crate) notification_content: bool,
    /// Which action buttons a new-mail notification carries (#244), by
    /// name: `mark_read`, `archive`, `delete`, `reply`, `forward`, `spam`;
    /// at most three count. Absent = mark_read, archive, delete.
    #[serde(default = "default_notification_buttons")]
    pub(crate) notification_buttons: Vec<String>,
    /// Whether message rows take a sideways swipe at all (#92, PR #135).
    #[serde(default = "default_on")]
    pub(crate) swipe_enabled: bool,
    /// The identity new messages are sent from (#157): an account's or
    /// alias's address, or empty for the account of the open folder (the
    /// original behaviour). Replies keep answering from the address the
    /// original was sent to, whatever this says.
    #[serde(default)]
    pub(crate) compose_default_from: String,
    /// Whether pasting into the composer strips the clipboard's formatting
    /// (the default). Off, a paste keeps its formatting. The editor's context
    /// menu always offers both, whichever way this is set.
    #[serde(default = "default_on")]
    pub(crate) paste_plain: bool,
    /// Whether Return in the composer starts a new paragraph (a hard
    /// return) rather than breaking the line (the default). Shift+Return
    /// does whichever this does not.
    #[serde(default)]
    pub(crate) return_paragraph: bool,
    /// Whether the composer's format bar starts with its extra tools shown
    /// (paragraph styles, fonts, colors, indents, emoji and pictures, #358)
    /// rather than behind its chevron, the default.
    #[serde(default)]
    pub(crate) toolbar_expanded: bool,
    /// The font rich-text messages are written and sent in, as a CSS family
    /// list. Empty, the editor shows the interface font and the message
    /// names none, so it reads in the recipient's own.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) compose_font: String,
    /// Their text size in points; 0 leaves it to the recipient.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) compose_font_size: u32,
    /// The zoom a new message opens at, in percent; 0 is 100 (#535).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub(crate) compose_zoom: u32,
    /// New messages start as plain text, without formatting (#180). Kept
    /// written so a version that predates `compose_format` still opens its
    /// composer the way this one was left.
    #[serde(default)]
    pub(crate) compose_plain: bool,
    /// What new messages are written in: rich text or plain text (a saved
    /// Markdown or HTML reads as rich text, see `ComposeFormat`). Absent on installs that predate the choice,
    /// where `compose_plain` above still says which of the two it is.
    #[serde(default)]
    pub(crate) compose_format: Option<ComposeFormat>,
    /// Whether a reply or forward to mail that had no HTML part is written
    /// in plain text whatever `compose_format` says (#436). Mailing lists
    /// and the people on them often write plain text and expect it back.
    #[serde(default)]
    pub(crate) follow_plain: bool,
    /// Where the signature sits in a reply or forward (#237): above the
    /// quoted original, or below it.
    #[serde(default)]
    pub(crate) signature_position: SignaturePosition,
    /// Whether the signature follows a `-- ` line, the delimiter some mail
    /// programs use to dim a signature and leave it out of their quotes.
    /// Off by default, as in most mail programs today.
    #[serde(default)]
    pub(crate) signature_dashes: bool,
    /// Whether a reply or forward opens with its conversation history
    /// hidden under the history's header (#549). It is sent either way. Off
    /// by default: the composer opens in a window of its own, where nothing
    /// beside it shows the conversation (docs/DESIGN.md, "Replying and
    /// composing").
    #[serde(default)]
    pub(crate) fold_quote: bool,
    /// The line above the quote in a reply, with `{name}`, `{address}`,
    /// `{date}` and `{time}` filled in (#528). Empty for the default, in the
    /// app's language.
    #[serde(default)]
    pub(crate) reply_header: String,
    /// Undo Send (#434): seconds a sent message waits, with Undo on offer,
    /// before it is handed to the server. 0 sends at once.
    #[serde(default = "default_undo_send_secs")]
    pub(crate) undo_send_secs: u32,
    /// Whether the composer underlines misspelled words as you type.
    #[serde(default = "default_on")]
    pub(crate) spellcheck: bool,
    /// Languages to check against, comma-separated (e.g. "en_US, de_DE").
    /// Empty = follow the session locale (WebKit's own default).
    #[serde(default)]
    pub(crate) spellcheck_langs: String,
    /// The app chrome's theme: follow the system, or force light/dark.
    #[serde(default)]
    pub(crate) app_theme: AppTheme,
    /// Lines of message text shown under the subject in the list: 0 turns the
    /// preview off entirely, and stops it being fetched.
    #[serde(default = "default_preview_lines")]
    pub(crate) preview_lines: u32,
    /// The columns a single-line list shows, in order, by name (#334).
    /// Names, not the enum, so a column a later version adds does not make
    /// an older one throw the whole file away.
    #[serde(default = "default_list_columns")]
    pub(crate) list_columns: Vec<String>,
    /// Headings over those columns, which sort the list when clicked.
    #[serde(default)]
    pub(crate) list_headings: bool,
    /// Columns keep their widths and a single line too wide for the pane
    /// scrolls sideways (#443). Off, the default and the way it was, the
    /// name columns narrow to fit.
    #[serde(default)]
    pub(crate) list_column_scroll: bool,
    /// The widths columns were dragged to, in pixels, by column name.
    #[serde(default)]
    pub(crate) list_column_widths: std::collections::BTreeMap<String, i32>,
    /// Single-key shortcuts (j/k, r, a, d…) without a modifier. On for a new
    /// install, so the menus' key hints name keys that work; a file from
    /// before the default changed keeps its own choice.
    #[serde(default)]
    pub(crate) single_key_shortcuts: bool,
    /// Gmail's keys in place of Tegami Mail's own for the single-key shortcuts
    /// (#392): j/k move without opening, o opens, e archives, # deletes, g
    /// then a letter goes to a folder. Off by default.
    #[serde(default)]
    pub(crate) gmail_keys: bool,
    /// How many days downloaded mail (bodies and attachment files) is kept
    /// on this computer (#394); 0 keeps it. The message index always stays.
    #[serde(default)]
    pub(crate) keep_downloads_days: u32,
    /// Keep running after the window is closed, so new mail still arrives and
    /// notifies. Off by default: closing a window is expected to quit.
    #[serde(default)]
    pub(crate) run_in_background: bool,
    /// Start at login (only meaningful with `run_in_background`).
    #[serde(default)]
    pub(crate) autostart: bool,
    /// Where the reader sits (#475): beside the list, below it, or off,
    /// the list then taking the whole window and messages opening in
    /// their own windows.
    #[serde(default)]
    pub(crate) reading_pane: ReadingPane,
    /// Whether the launcher icon in a dock or task manager shows the unread
    /// count (#271). Kept written so a version that predates
    /// `launcher_badge` still shows a badge when this one does.
    #[serde(default = "default_on")]
    pub(crate) launcher_count: bool,
    /// What that count counts (#431). Absent on installs that predate the
    /// choice, where `launcher_count` above says whether there is one.
    #[serde(default)]
    pub(crate) launcher_badge: Option<LauncherBadge>,
    /// Whether to say anything at all when remote content is blocked. Off hides
    /// the banner; it never changes what is blocked, only whether you're told.
    #[serde(default = "default_on")]
    pub(crate) show_remote_banner: bool,
    /// Console mode (#status-bar): the verbose activity console is offered in
    /// the status bar. Off by default.
    #[serde(default)]
    pub(crate) console_mode: bool,
    /// The status bar at the top of the window rather than its foot (#433).
    /// Off by default, for everyone: the key is new in 1.44.
    #[serde(default)]
    pub(crate) status_bar_top: bool,
    /// Read-marking policy (#100).
    #[serde(default)]
    pub(crate) read_mark: ReadMark,
    /// Answering a request for a read receipt (#520).
    #[serde(default)]
    pub(crate) receipts: ReceiptPolicy,
    /// New messages ask for a read receipt (#520).
    #[serde(default)]
    pub(crate) request_receipts: bool,
    /// Where a receipt that comes back goes (#520).
    #[serde(default)]
    pub(crate) returned_receipts: ReturnedReceipts,
    /// A desktop notification when a receipt comes back (#520).
    #[serde(default = "default_on")]
    pub(crate) notify_receipts: bool,
    /// GNOME Files hand-off (#188 follow-up): what the files open into,
    /// what happens over the size limit, and the limit itself in MB.
    #[serde(default)]
    pub(crate) files_action: FilesAction,
    #[serde(default)]
    pub(crate) files_large: FilesLarge,
    #[serde(default = "default_files_limit_mb")]
    pub(crate) files_limit_mb: u32,
    /// What the main window shows at launch (#256).
    #[serde(default)]
    pub(crate) start_view: StartView,
    /// How accounts' folders are sorted, unless an account chooses.
    #[serde(default)]
    pub(crate) folder_sort: FolderSort,
    /// What a printed page carries besides the message (#359).
    #[serde(default)]
    pub(crate) print: PrintOptions,
}

fn default_fetch_interval() -> u64 {
    300
}

fn default_notification_buttons() -> Vec<String> {
    NotificationButtons::default().to_list()
}

fn default_preview_lines() -> u32 {
    1
}

impl Default for PrivacyFile {
    fn default() -> Self {
        Self {
            allowed_senders: Vec::new(),
            auto_remote_content: false,
            show_remote_banner: true,
            // Under the accounts' folders, as the redesign has it (mockup 01).
            // Above the accounts (owner's decision, 2026-11-04): one
            // section for every account, in sight however many there are.
            console_mode: false,
            status_bar_top: false,
            read_mark: ReadMark::default(),
            receipts: ReceiptPolicy::default(),
            request_receipts: false,
            returned_receipts: ReturnedReceipts::default(),
            notify_receipts: true,
            files_action: FilesAction::default(),
            files_large: FilesLarge::default(),
            files_limit_mb: default_files_limit_mb(),
            start_view: StartView::default(),
            folder_sort: FolderSort::default(),
            gravatar: false,
            avatars: true,
            contact_book: String::new(),
            sender_logos: false,
            fetch_interval_secs: default_fetch_interval(),
            push: true,
            blacklist: Vec::new(),
            threading: true,
            thread_newest_first: false,
            confirm_thread_delete: true,
            notifications: true,
            notification_content: true,
            notification_buttons: default_notification_buttons(),
            swipe_enabled: true,
            compose_default_from: String::new(),
            // A new install takes today's default, not the 2026-08-31 one.
            paste_plain: true,
            return_paragraph: false,
            toolbar_expanded: false,
            compose_font: String::new(),
            compose_font_size: 0,
            compose_zoom: 0,
            compose_plain: false,
            compose_format: None,
            follow_plain: false,
            signature_position: SignaturePosition::default(),
            signature_dashes: false,
            fold_quote: false,
            reply_header: String::new(),
            undo_send_secs: default_undo_send_secs(),
            spellcheck: true,
            spellcheck_langs: String::new(),
            app_theme: AppTheme::default(),
            preview_lines: default_preview_lines(),
            list_columns: default_list_columns(),
            list_headings: false,
            list_column_scroll: false,
            list_column_widths: Default::default(),
            single_key_shortcuts: true,
            gmail_keys: false,
            keep_downloads_days: 0,
            run_in_background: false,
            autostart: false,
            reading_pane: ReadingPane::Right,
            launcher_count: true,
            launcher_badge: None,
            print: PrintOptions::default(),
        }
    }
}

pub(super) fn privacy_path() -> Option<PathBuf> {
    config_file("privacy.toml")
}

pub(crate) fn load_privacy() -> PrivacyFile {
    let Some(path) = privacy_path() else {
        return PrivacyFile::default();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return PrivacyFile::default();
    };
    toml::from_str::<PrivacyFile>(&text).unwrap_or_default()
}

impl PrivacyFile {
    /// The launcher badge choice, falling back to the on/off switch it
    /// replaced.
    pub(crate) fn launcher_badge(&self) -> LauncherBadge {
        self.launcher_badge.unwrap_or(if self.launcher_count {
            LauncherBadge::AllUnread
        } else {
            LauncherBadge::Off
        })
    }
}

/// Persist all app settings together (so no field is clobbered).
pub(crate) fn save_privacy(file: PrivacyFile) {
    save_private_toml(privacy_path(), &file, "privacy settings", true);
}
