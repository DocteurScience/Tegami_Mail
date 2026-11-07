//! privacy.toml: every app preference in one file, its defaults for a new
//! install and for an upgrade, and the one-time migrations applied when it
//! is read.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::look::{
    default_list_columns, default_text_scale, AppTheme, ClockStyle, DateStyle, ListLayout,
    MessageTheme, ReadingPane, TrayIcon,
};
use super::prefs::{
    default_files_limit_mb, default_reader_zoom, default_swipe_sensitivity, default_undo_send_secs,
    ComposeFormat, FilesAction, FilesLarge, FoldMessages, LauncherBadge, NotificationButtons,
    PrintOptions, ReadMark, ReceiptPolicy, ReturnedReceipts, SignaturePosition,
};
use super::sections::{FolderSort, RailFold, SectionPlacement, SectionStyle, StartView};
use super::{config_file, default_on, is_zero, save_private_toml, write_private};

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
    /// Whether the remote content of the next messages in the list is
    /// fetched while one is read (#474), where remote content is allowed.
    /// Off by default: a sender's tracking learns of a message never opened.
    #[serde(default)]
    pub(crate) remote_ahead: bool,
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
    /// Whether the mail you sent wears its mailbox's face — the account's
    /// Gravatar, picture or emoji (#189) — instead of the circle any other
    /// sender would get.
    #[serde(default = "default_on")]
    pub(crate) own_mailbox_face: bool,
    /// Whether a tagged message's row in the list is washed with its first
    /// tag's color (PR #383). Off unless switched on: the design's cards
    /// keep the card's own ground and show a tag as a named chip, so that
    /// the tag is never told by colour alone (docs/DESIGN.md, "Message
    /// list").
    #[serde(default)]
    pub(crate) tag_row_tint: bool,
    /// The address book new contacts go to: an EDS source UID, or the
    /// Tegami Mail book's. Empty picks the first writable EDS book, else Tegami Mail.
    #[serde(default)]
    pub(crate) contact_book: String,
    /// How dates are written (#32).
    #[serde(default)]
    pub(crate) date_style: DateStyle,
    /// Whether the clock runs to 12 or 24 (#32).
    #[serde(default)]
    pub(crate) clock_style: ClockStyle,
    /// Seconds between automatic mail checks; 0 = manual only.
    #[serde(default = "default_fetch_interval")]
    pub(crate) fetch_interval_secs: u64,
    /// Whether to use IMAP IDLE push for instant new-mail delivery.
    #[serde(default = "default_on")]
    pub(crate) push: bool,
    /// On a metered connection, hold back the work done ahead of the user:
    /// prefetching bodies and attachments, indexing old mail, watching
    /// folders other than the Inbox, fetching sender logos (#442).
    #[serde(default = "default_on")]
    pub(crate) metered_save_data: bool,
    /// Addresses or whole domains whose incoming mail is auto-deleted (to Trash).
    /// Stored lowercased; a bare domain like "spam.com" matches any sender there.
    #[serde(default)]
    pub(crate) blacklist: Vec<String>,
    /// Seconds the message-list actions palette stays open after the cursor
    /// leaves it before auto-collapsing. (A prior `palette_delay_ms` setting in
    /// milliseconds is intentionally not migrated — its meaning has changed.)
    #[serde(default = "default_palette_collapse")]
    pub(crate) palette_collapse_secs: u64,
    /// Seconds a message card's actions palette stays open after the cursor
    /// leaves it. Separate from the list's: cards are read at a different
    /// pace from a list being skimmed.
    #[serde(default = "default_palette_collapse")]
    pub(crate) card_palette_collapse_secs: u64,
    /// Group messages into conversation threads in the list.
    #[serde(default = "default_on")]
    pub(crate) threading: bool,
    /// Whether a conversation's row in the message list speaks for the newest
    /// message anywhere in the account rather than the newest one the folder
    /// itself holds: the reply you sent, which is filed in Sent (#236). Off by
    /// default, so the row goes on describing the last mail that arrived.
    #[serde(default)]
    pub(crate) thread_row_newest: bool,
    /// Whether the reading pane shows a conversation newest-message-first.
    #[serde(default)]
    pub(crate) thread_newest_first: bool,
    /// Which messages of a conversation open folded to one line (#326).
    #[serde(default)]
    pub(crate) fold_messages: FoldMessages,
    /// Whether the reader always shows the recipients line under the sender.
    #[serde(default)]
    pub(crate) always_show_recipients: bool,
    /// Whether the OpenPGP chip beside a sender's name says its verdict in
    /// words (#300); off, the default, shows the lock and shield icons alone.
    #[serde(default)]
    pub(crate) pgp_labels: bool,
    /// The reader's find bar opens under the message instead of above it
    /// (#392). Off by default.
    #[serde(default)]
    pub(crate) reader_find_bottom: bool,
    /// Whether a lone message renders as an inset card like a conversation's
    /// messages (#57); off, the default, keeps the full-bleed view.
    #[serde(default = "default_single_message_card")]
    pub(crate) single_message_card: bool,
    /// Whether the chip naming the message's account is shown above the
    /// subject in the reader header. On by default.
    #[serde(default = "default_on")]
    pub(crate) account_chip: bool,
    /// Draw icons from the desktop's icon theme instead of Tegami Mail's own
    /// (Settings → Appearance). Off: the app looks the same everywhere.
    #[serde(default)]
    pub(crate) desktop_icons: bool,
    /// The message zoom every launch starts at, in percent (Settings →
    /// Reading). Ctrl+ and Ctrl- move away from it for the session; the
    /// next launch is back here.
    #[serde(default = "default_reader_zoom")]
    pub(crate) reader_zoom: u32,
    /// Each conversation message lists its own attachments beneath its body
    /// (#213), so which file came with which message is never in doubt.
    #[serde(default = "default_on")]
    pub(crate) card_attachments: bool,
    /// The attachment drawer beneath the reader, gathering every attachment
    /// in the open conversation (#213). Off by default: each message shows
    /// its own files in a section under its body (docs/DESIGN.md,
    /// "Attachments"), and the drawer would show them a second time.
    #[serde(default)]
    pub(crate) attachment_drawer: bool,
    /// Whether deleting a whole selected conversation asks for confirmation
    /// first.
    #[serde(default = "default_on")]
    pub(crate) confirm_thread_delete: bool,
    /// How email content is themed (independent of the app UI theme).
    #[serde(default)]
    pub(crate) message_theme: MessageTheme,
    /// Set every message in the reader's own font instead of the sender's
    /// (#56).
    #[serde(default)]
    pub(crate) override_fonts: bool,
    /// That font, as a Pango description ("Cantarell 11"); empty = the
    /// interface font.
    #[serde(default)]
    pub(crate) reader_font: String,
    /// Ignore the sender's text and background colors (#56).
    #[serde(default)]
    pub(crate) override_colors: bool,
    /// Show plain-text messages in a monospace font (#181).
    #[serde(default)]
    pub(crate) plain_monospace: bool,
    /// That font, as a Pango description; empty = the desktop's monospace font.
    #[serde(default)]
    pub(crate) plain_font: String,
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
    /// Whether the combined Accounts & Preferences window opens showing the
    /// Accounts view instead of Preferences (the default).
    #[serde(default)]
    pub(crate) settings_open_accounts: bool,
    /// Whether a conversation card's action icons stay hidden until the card
    /// is hovered (expanded via their ⋯ toggle). Off = always shown, unless
    /// `card_actions_auto` shows them automatically on hover.
    #[serde(default = "default_on")]
    pub(crate) card_actions_hover: bool,
    /// With the ⋯ toggle off: show the action icons automatically while the
    /// card is hovered (rather than always).
    #[serde(default = "default_card_actions_auto")]
    pub(crate) card_actions_auto: bool,
    /// Whether the message list rows carry an actions palette at all. Off
    /// removes the ⋯ line entirely, returning its space to the row.
    #[serde(default = "default_on")]
    pub(crate) list_palette: bool,
    /// Whether the message list's actions palette opens on row hover, without
    /// needing the ⋯ click.
    #[serde(default)]
    pub(crate) list_palette_hover: bool,
    /// Whether the ⋯ on a message card opens the card's menu in place of
    /// sliding its actions palette out. On for new installs (2026-10-06);
    /// an install that saved its settings before then keeps its own value,
    /// since every save writes every key.
    #[serde(default = "default_on")]
    pub(crate) card_palette_menu: bool,
    /// Whether message rows take a sideways swipe at all (#92, PR #135).
    #[serde(default = "default_on")]
    pub(crate) swipe_enabled: bool,
    /// Swap the message list's swipe-gesture sides: off (default) swipes
    /// left to delete and right to archive, on reverses them.
    #[serde(default)]
    pub(crate) swipe_reversed: bool,
    /// How far a trackpad's two-finger swipe has to travel before a row
    /// commits (higher = shorter swipe). Trackpads differ enough that one
    /// fixed figure suits nobody, hence the knob; mouse and touchscreen
    /// drags follow the finger 1:1 and ignore this.
    #[serde(default = "default_swipe_sensitivity")]
    pub(crate) swipe_sensitivity: f64,
    /// The identity new messages are sent from (#157): an account's or
    /// alias's address, or empty for the account of the open folder (the
    /// original behaviour). Replies keep answering from the address the
    /// original was sent to, whatever this says.
    #[serde(default)]
    pub(crate) compose_default_from: String,
    /// The inset-card default for lone messages (#57, #153) was applied
    /// once to installs that predate it. Set on the first load that did so.
    #[serde(default)]
    pub(crate) single_card_default_applied: bool,
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
    /// The zoom a new message opens at, in percent; 0 follows the
    /// reader's `reader_zoom` (#535).
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
    /// Hovering the icon rail (narrow-window or user-collapsed) floats the
    /// full sidebar out over the panes without needing the expand button.
    #[serde(default)]
    pub(crate) sidebar_hover_expand: bool,
    /// Reopen with the accounts, folders and sections as they were left.
    /// Off starts every launch with everything folded up.
    #[serde(default = "default_on")]
    pub(crate) remember_sidebar: bool,
    /// Reopen collapsed to the icon rail if that is how it was left. Off
    /// starts every launch with the full sidebar.
    #[serde(default = "default_on")]
    pub(crate) remember_rail: bool,
    /// Icon rail: a dot for unread mail in place of the count. On by
    /// default, as are the fold-ups below.
    #[serde(default = "default_on")]
    pub(crate) rail_dots: bool,
    /// Unread counts in the accent color only for mail that arrived since
    /// the folder was last looked at; the rest neutral (#343). Off by
    /// default.
    #[serde(default)]
    pub(crate) seen_counts: bool,
    /// Icon rail: the sections folded up by themselves when the sidebar
    /// collapses, one switch each.
    #[serde(default)]
    pub(crate) rail_fold: RailFold,
    /// The app chrome's theme: follow the system, or force light/dark.
    #[serde(default)]
    pub(crate) app_theme: AppTheme,
    /// The app's text size, in percent of the desktop's (#267).
    #[serde(default = "default_text_scale")]
    pub(crate) text_scale: u32,
    /// The appearance theme's id: a bundled palette (see `theme.rs`) painted
    /// over libadwaita's colors, or "system" for the stock GNOME look.
    /// Empty — a file written before themes existed — means the same.
    #[serde(default)]
    pub(crate) theme: String,
    /// Lines of message text shown under the subject in the list: 0 turns the
    /// preview off entirely, and stops it being fetched.
    #[serde(default = "default_preview_lines")]
    pub(crate) preview_lines: u32,
    /// How the message list lays out a message: as a card, on one line, or
    /// whichever the pane's width suits (#334).
    #[serde(default)]
    pub(crate) list_layout: ListLayout,
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
    /// A list sorted by date is divided under Today, Yesterday, Last 7 days,
    /// Last 14 days and Older (#435). On by default since 1.44, for
    /// existing installs too (see `date_groups_default_applied`).
    #[serde(default = "default_on")]
    pub(crate) list_group_by_date: bool,
    /// A card's tags and Gmail labels go on a line of their own under its
    /// text, instead of at the end of the subject (#374). On by default:
    /// the design puts them below the preview, where a long subject does
    /// not push them off the card (docs/DESIGN.md, "Message list").
    #[serde(default = "default_on")]
    pub(crate) list_pills_below: bool,
    /// Grouping by date was turned on once for an install from before it
    /// became the default: every save writes every key, so those files
    /// carry the old `false` without anyone having chosen it. A new
    /// install starts with this set, so switching grouping off sticks.
    #[serde(default)]
    pub(crate) date_groups_default_applied: bool,
    /// The search field stays above the list instead of hiding behind the
    /// search button (#446). Off by default.
    #[serde(default)]
    pub(crate) list_search_open: bool,
    /// A saved contact goes by the name in the address book, not the one
    /// its mail gives (#541). Off by default.
    #[serde(default)]
    pub(crate) contact_names: bool,
    /// The search field and the bulk bar open under the list instead of
    /// above it (#496). Off by default.
    #[serde(default)]
    pub(crate) list_search_bottom: bool,
    #[serde(default)]
    pub(crate) list_bulk_bar_bottom: bool,
    /// The reader toolbar's right group stays in the ⋯ menu at every
    /// width. On for a new install; a file from before the key keeps the
    /// group on the bar while it fits.
    #[serde(default)]
    pub(crate) toolbar_fold_right: bool,
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
    /// The main window opens fullscreen (#511).
    #[serde(default)]
    pub(crate) start_fullscreen: bool,
    /// Where the reader sits (#475): beside the list, below it, or off,
    /// the list then taking the whole window and messages opening in
    /// their own windows.
    #[serde(default)]
    pub(crate) reading_pane: ReadingPane,
    /// Publish a tray icon (StatusNotifierItem) for desktops that draw one
    /// (issue #116). Off by default: GNOME has no tray without an extension.
    #[serde(default)]
    pub(crate) tray: bool,
    /// Which icon the tray item shows.
    #[serde(default)]
    pub(crate) tray_icon: TrayIcon,
    /// Whether the tray menu lists unread inbox mail, each a row that opens
    /// the message.
    #[serde(default = "default_on")]
    pub(crate) tray_mail: bool,
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
    /// Whether a "Check this sender" verdict (the addressing points somewhere
    /// the From: line doesn't) puts the red banner over the message. The badge
    /// still shows the verdict, and a failed check always gets the banner.
    #[serde(default = "default_on")]
    pub(crate) show_spoof_banner: bool,
    /// How Filters, Tags, Favorites and Contacts are drawn wherever they sit.
    #[serde(default)]
    pub(crate) section_style: SectionStyle,
    /// Where the Tags section sits, the same choices.
    #[serde(default)]
    pub(crate) tags_placement: SectionPlacement,
    /// Where Favorites sit, the same choices.
    #[serde(default)]
    pub(crate) favorites_placement: SectionPlacement,
    /// Whether the sidebar's disclosure chevrons (merged rows, account
    /// headers) LEAD their rows; off puts them back at the row's end.
    #[serde(default = "default_chevrons_left")]
    pub(crate) chevrons_left: bool,
    /// Whether a click anywhere on an account's header folds it, as it
    /// once did. Off (the default): its chevron, or a double click.
    #[serde(default)]
    pub(crate) account_click_folds: bool,
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
    /// Which browser a link in a message opens in (#232). Empty = whatever
    /// the desktop opens links with; `"ask"` = the system's app chooser,
    /// every time; anything else is a desktop entry id
    /// ("brave-browser.desktop") launched directly.
    #[serde(default)]
    pub(crate) link_browser: String,
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

fn default_chevrons_left() -> bool {
    // Right for new installs (Jason, 2026-08-30): the classic trailing
    // placement; Left is the opt-in. Only a privacy.toml missing the key
    // sees this — every save writes all fields.
    false
}

fn default_fetch_interval() -> u64 {
    300
}

fn default_single_message_card() -> bool {
    // On: a lone message sits in a card like every message of a
    // conversation, so the reader looks the same whatever it opens (owner's
    // decision, 2026-11-07, reversing the flat default of 2026-11-03). Only
    // a privacy.toml MISSING this key sees the default — every save writes
    // all fields, so an existing install's choice is pinned.
    true
}

fn default_palette_collapse() -> u64 {
    5
}

fn default_notification_buttons() -> Vec<String> {
    NotificationButtons::default().to_list()
}

fn default_card_actions_auto() -> bool {
    // Off for new installs (2026-08-30): card actions wait behind the ⋯
    // toggle (card_actions_hover) rather than appearing on hover.
    false
}

fn default_preview_lines() -> u32 {
    1
}

impl Default for PrivacyFile {
    fn default() -> Self {
        Self {
            allowed_senders: Vec::new(),
            auto_remote_content: false,
            remote_ahead: false,
            show_remote_banner: true,
            show_spoof_banner: true,
            section_style: SectionStyle::default(),
            // Under the accounts' folders, as the redesign has it (mockup 01).
            // Above the accounts (owner's decision, 2026-11-04): one
            // section for every account, in sight however many there are.
            tags_placement: SectionPlacement::AboveAccounts,
            favorites_placement: SectionPlacement::AboveAccounts,
            chevrons_left: default_chevrons_left(),
            account_click_folds: false,
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
            link_browser: String::new(),
            start_view: StartView::default(),
            folder_sort: FolderSort::default(),
            gravatar: false,
            avatars: true,
            own_mailbox_face: true,
            tag_row_tint: false,
            contact_book: String::new(),
            sender_logos: false,
            date_style: DateStyle::default(),
            clock_style: ClockStyle::default(),
            fetch_interval_secs: default_fetch_interval(),
            push: true,
            metered_save_data: true,
            blacklist: Vec::new(),
            palette_collapse_secs: default_palette_collapse(),
            card_palette_collapse_secs: default_palette_collapse(),
            threading: true,
            thread_row_newest: false,
            thread_newest_first: false,
            fold_messages: FoldMessages::Read,
            always_show_recipients: false,
            pgp_labels: false,
            reader_find_bottom: false,
            single_message_card: default_single_message_card(),
            account_chip: true,
            desktop_icons: false,
            reader_zoom: default_reader_zoom(),
            card_attachments: true,
            attachment_drawer: false,
            confirm_thread_delete: true,
            message_theme: MessageTheme::default(),
            override_fonts: false,
            reader_font: String::new(),
            override_colors: false,
            plain_monospace: false,
            plain_font: String::new(),
            notifications: true,
            notification_content: true,
            notification_buttons: default_notification_buttons(),
            settings_open_accounts: false,
            card_actions_hover: true,
            card_actions_auto: default_card_actions_auto(),
            list_palette: true,
            list_palette_hover: false,
            card_palette_menu: true,
            swipe_enabled: true,
            swipe_reversed: false,
            swipe_sensitivity: default_swipe_sensitivity(),
            compose_default_from: String::new(),
            // A new install takes today's default, not the 2026-08-31 one.
            single_card_default_applied: true,
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
            sidebar_hover_expand: false,
            remember_sidebar: true,
            remember_rail: true,
            rail_dots: true,
            seen_counts: false,
            rail_fold: RailFold::default(),
            app_theme: AppTheme::default(),
            text_scale: default_text_scale(),
            theme: String::new(),
            preview_lines: default_preview_lines(),
            list_layout: ListLayout::Cards,
            list_columns: default_list_columns(),
            list_headings: false,
            list_column_scroll: false,
            list_group_by_date: true,
            list_pills_below: true,
            date_groups_default_applied: true,
            list_search_open: false,
            contact_names: false,
            list_search_bottom: false,
            list_bulk_bar_bottom: false,
            toolbar_fold_right: true,
            list_column_widths: Default::default(),
            single_key_shortcuts: true,
            gmail_keys: false,
            keep_downloads_days: 0,
            run_in_background: false,
            autostart: false,
            start_fullscreen: false,
            reading_pane: ReadingPane::Right,
            tray: false,
            tray_icon: TrayIcon::default(),
            tray_mail: true,
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
    let mut file = toml::from_str::<PrivacyFile>(&text).unwrap_or_default();
    let moved = toml::from_str::<LegacySwitches>(&text).is_ok_and(|old| old.place(&mut file));
    if apply_new_defaults(&mut file) || moved {
        if let Ok(toml) = toml::to_string_pretty(&file) {
            let _ = write_private(&path, &toml);
        }
    }
    file
}

/// The switches that once kept Filters, Tags, Favorites and Contacts in
/// the unified section. That section no longer holds them: a file from
/// then has them placed above the accounts where their switch was on, and
/// off where it was off.
#[derive(serde::Deserialize)]
pub(super) struct LegacySwitches {
    #[serde(default = "default_on")]
    unified_tags: bool,
    #[serde(default = "default_on")]
    unified_favorites: bool,
}

impl LegacySwitches {
    /// Whether any placement moved.
    pub(super) fn place(&self, file: &mut PrivacyFile) -> bool {
        let mut moved = false;
        for (at, on) in [
            (&mut file.tags_placement, self.unified_tags),
            (&mut file.favorites_placement, self.unified_favorites),
        ] {
            if *at == SectionPlacement::AllInboxes {
                *at = if on {
                    SectionPlacement::AboveAccounts
                } else {
                    SectionPlacement::Off
                };
                moved = true;
            }
        }
        moved
    }
}

/// Bring a file from before a default changed up to it, once: every save
/// writes every key, so an older install carries the old value without
/// anyone having chosen it. Whether anything changed.
pub(super) fn apply_new_defaults(file: &mut PrivacyFile) -> bool {
    let mut changed = false;
    // The inset card for a lone message became the default on 2026-08-31
    // (#153).
    if !file.single_card_default_applied {
        file.single_card_default_applied = true;
        file.single_message_card = true;
        changed = true;
    }
    // Grouping the list by date, the default since 1.44 (#435).
    if !file.date_groups_default_applied {
        file.date_groups_default_applied = true;
        file.list_group_by_date = true;
        changed = true;
    }
    changed
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
