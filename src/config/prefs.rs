//! The smaller preferences kept in privacy.toml (reading, composing,
//! notifications, files, printing) and the accessors that read them.

use super::default_on;
use super::privacy::load_privacy;

/// What a printed page carries besides the message (#359), from Settings →
/// Reading → Printing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct PrintOptions {
    /// "Page 2 of 5" at the foot of every page.
    #[serde(default = "default_on")]
    pub(crate) page_numbers: bool,
    /// The date and time it was printed, at the foot of every page.
    #[serde(default)]
    pub(crate) date: bool,
    /// A wider left margin, so a hole punch misses the text.
    #[serde(default)]
    pub(crate) punch_margin: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        PrintOptions {
            page_numbers: true,
            date: false,
            punch_margin: false,
        }
    }
}

/// How much of an account's mail is kept on disk ahead of being opened
/// (#536). `Newest` is the newest messages of each folder, as before; the
/// others keep every message of the period, or all of it, so it opens with
/// no network.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OfflineMail {
    #[default]
    Newest,
    Week,
    Month,
    Year,
    All,
}

impl OfflineMail {
    pub(crate) const ALL: [OfflineMail; 5] = [
        OfflineMail::Newest,
        OfflineMail::Week,
        OfflineMail::Month,
        OfflineMail::Year,
        OfflineMail::All,
    ];

    pub(crate) fn label(self) -> String {
        use crate::i18n::i18n;
        match self {
            OfflineMail::Newest => i18n("Newest messages"),
            OfflineMail::Week => i18n("Last week"),
            OfflineMail::Month => i18n("Last month"),
            OfflineMail::Year => i18n("Last year"),
            OfflineMail::All => i18n("All mail"),
        }
    }

    /// Its place in [`OfflineMail::ALL`], for combo rows.
    pub(crate) fn index(self) -> u32 {
        OfflineMail::ALL
            .iter()
            .position(|s| *s == self)
            .unwrap_or(0) as u32
    }

    /// The oldest date (a Unix time) kept offline as of `now`; `None` for
    /// `Newest`, which keeps no period, and `i64::MIN` for all of it.
    pub(crate) fn since(self, now: i64) -> Option<i64> {
        let days = match self {
            OfflineMail::Newest => return None,
            OfflineMail::Week => 7,
            OfflineMail::Month => 31,
            OfflineMail::Year => 366,
            OfflineMail::All => return Some(i64::MIN),
        };
        Some(now - days * 86_400)
    }

    pub(super) fn is_default(&self) -> bool {
        *self == OfflineMail::Newest
    }
}

/// Where an account files the copy of a reply it sends (#468).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReplyCopies {
    /// Wherever the rest of sent mail goes.
    #[default]
    WithSent,
    /// In the folder of the message replied to.
    Original,
    /// The same, but a reply to mail in the Inbox goes with sent mail:
    /// what the Inbox holds is waiting to be dealt with, and a reply is not.
    OriginalNotInbox,
}

impl ReplyCopies {
    pub(crate) const ALL: [ReplyCopies; 3] = [
        ReplyCopies::WithSent,
        ReplyCopies::Original,
        ReplyCopies::OriginalNotInbox,
    ];

    pub(crate) fn label(self) -> String {
        match self {
            ReplyCopies::WithSent => crate::i18n::i18n("With sent mail"),
            ReplyCopies::Original => crate::i18n::i18n("Folder of the original"),
            ReplyCopies::OriginalNotInbox => {
                crate::i18n::i18n("Folder of the original, not the Inbox")
            }
        }
    }

    /// Its place in [`ReplyCopies::ALL`], for combo rows.
    pub(crate) fn index(self) -> u32 {
        ReplyCopies::ALL
            .iter()
            .position(|s| *s == self)
            .unwrap_or(0) as u32
    }

    pub(super) fn is_default(&self) -> bool {
        *self == ReplyCopies::WithSent
    }
}

/// When an opened message is marked read (#100).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadMark {
    /// The moment it is shown (conversation members as they scroll into view).
    #[default]
    Shown,
    /// After it has been in view for a couple of seconds.
    Delay,
    /// Never automatically; only an explicit mark-as-read.
    Manual,
}

/// What to do when a message asks for a read receipt (#520).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReceiptPolicy {
    /// A banner on the message offers to send one.
    #[default]
    Ask,
    /// Sent when the message is shown, unless RFC 8098 says to ask: the
    /// receipt would go somewhere other than where the mail came from, or
    /// the message was not addressed to you.
    Always,
    /// Never sent, and never asked about.
    Never,
}

/// Where a read receipt goes once the message it answers shows that it
/// was read (#520).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReturnedReceipts {
    /// Marked read and moved to Trash: what it says is on the message now.
    #[default]
    Trash,
    /// Marked read and archived.
    Archive,
    /// Left in the Inbox as it came.
    Keep,
}

/// What a hand-off of files from GNOME Files ("Send with Tegami Mail", "Open
/// With Tegami Mail", "Email…") opens them into.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FilesAction {
    /// A dialog offers the three below, every time.
    #[default]
    Ask,
    /// A new message with the files attached.
    New,
    /// Pick one of the saved drafts and attach the files to it.
    Draft,
    /// Pick a message to reply to, with the files attached.
    Reply,
}

/// What happens to handed-in files that add up to more than the limit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FilesLarge {
    /// A dialog asks, every time.
    #[default]
    Ask,
    /// Attach them regardless.
    Attach,
    /// Upload them to cloud storage and put the links in the message.
    Cloud,
}

/// The GNOME Files hand-off preferences (Settings → System → GNOME Files).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FilesPrefs {
    pub(crate) action: FilesAction,
    pub(crate) large: FilesLarge,
    /// The size, in MB (decimal, like the sizes the composer shows), that
    /// the files together must stay under to be attached without a word.
    pub(crate) limit_mb: u32,
}

impl Default for FilesPrefs {
    fn default() -> Self {
        Self {
            action: FilesAction::Ask,
            large: FilesLarge::Ask,
            limit_mb: default_files_limit_mb(),
        }
    }
}

impl FilesPrefs {
    /// The limit in bytes.
    pub(crate) fn limit_bytes(&self) -> u64 {
        self.limit_mb as u64 * 1_000_000
    }
}

pub(super) fn default_files_limit_mb() -> u32 {
    20
}

pub(crate) fn load_files_prefs() -> FilesPrefs {
    let f = load_privacy();
    FilesPrefs {
        action: f.files_action,
        large: f.files_large,
        limit_mb: f.files_limit_mb.max(1),
    }
}

/// The zoom steps Ctrl+ and Ctrl- walk, and Settings offers, in percent.
pub(crate) const READER_ZOOM_STEPS: [u32; 12] =
    [50, 60, 70, 80, 90, 100, 110, 125, 150, 175, 200, 250];

/// The zoom a new message opens at: Settings → Composing's, or 100 while
/// that says "Same as reading pane zoom" (0) or anything off the step list.
pub(crate) fn load_compose_zoom() -> u32 {
    let z = load_privacy().compose_zoom;
    if READER_ZOOM_STEPS.contains(&z) {
        z
    } else {
        100
    }
}

/// What a message is written in: rich text, the WYSIWYG editor, or plain
/// text, which sends no HTML part at all (docs/DESIGN.md, "Menus").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ComposeFormat {
    /// Also what a setting saved as Markdown or HTML source reads as: those
    /// formats were removed (owner's decision, docs/SUIVI.md), and an
    /// unknown value would fail the whole settings file.
    #[default]
    #[serde(alias = "markdown", alias = "html")]
    Rich,
    Plain,
}

#[cfg(test)]
mod compose_format_tests {
    use super::ComposeFormat;

    #[derive(serde::Deserialize)]
    struct Holder {
        f: ComposeFormat,
    }

    #[test]
    fn removed_formats_read_as_rich_text() {
        for (value, want) in [
            ("rich", ComposeFormat::Rich),
            ("plain", ComposeFormat::Plain),
            ("markdown", ComposeFormat::Rich),
            ("html", ComposeFormat::Rich),
        ] {
            let h: Holder = toml::from_str(&format!("f = \"{value}\"")).unwrap();
            assert_eq!(h.f, want, "{value}");
        }
    }
}

/// Where the signature goes in a reply or forward (#237). Above the quoted
/// original is what Apple Mail and Thunderbird do, and what a reader
/// expects: the signature closes the words above it, not the words
/// somebody else wrote. Below is the placement Tegami Mail had before.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SignaturePosition {
    #[default]
    AboveQuote,
    BelowQuote,
}

/// What new messages start out as, falling back to the plain-text
/// switch this setting replaced (#180).
pub(crate) fn load_compose_format() -> ComposeFormat {
    let p = load_privacy();
    p.compose_format.unwrap_or(if p.compose_plain {
        ComposeFormat::Plain
    } else {
        ComposeFormat::Rich
    })
}

/// The narrowest and widest trackpad swipe sensitivity a row takes (0 would
/// divide by zero).
pub(crate) const SWIPE_SENSITIVITY_MIN: f64 = 1.0;
pub(crate) const SWIPE_SENSITIVITY_MAX: f64 = 10.0;

/// How far a trackpad's two-finger swipe has to travel to fire the action
/// (higher = shorter swipe). libadwaita spends a fixed 400px of scroll on a
/// full swipe whatever the row asks for, so at 1.0 a trackpad needs 240px of
/// two-finger travel to reach the commit distance, more than most touchpads
/// give in one go; 3.5 puts the row roughly under the fingers instead.
pub(crate) const SWIPE_SENSITIVITY: f64 = 3.5;

/// The delays Settings → Composing offers for Undo Send (#434), in seconds.
pub(crate) const UNDO_SEND_CHOICES: [u32; 6] = [0, 3, 5, 10, 20, 30];

pub(super) fn default_undo_send_secs() -> u32 {
    5
}

/// The address book chosen for new contacts (empty: automatic).
pub(crate) fn load_contact_book() -> String {
    load_privacy().contact_book
}

/// Whether Tegami Mail starts at login (background running only).
pub(crate) fn load_autostart() -> bool {
    let p = load_privacy();
    p.run_in_background && p.autostart
}

#[cfg(test)]
mod files_prefs_tests {
    use super::*;
    use crate::config::PrivacyFile;

    #[test]
    fn missing_keys_default_to_asking() {
        let file: PrivacyFile = toml::from_str("compose_inline = true\n").unwrap();
        assert_eq!(file.files_action, FilesAction::Ask);
        assert_eq!(file.files_large, FilesLarge::Ask);
        assert_eq!(file.files_limit_mb, 20);
    }

    #[test]
    fn keys_round_trip_by_name() {
        let file: PrivacyFile = toml::from_str(
            "files_action = \"reply\"\nfiles_large = \"cloud\"\nfiles_limit_mb = 50\n",
        )
        .unwrap();
        assert_eq!(file.files_action, FilesAction::Reply);
        assert_eq!(file.files_large, FilesLarge::Cloud);
        assert_eq!(file.files_limit_mb, 50);
        let text = toml::to_string(&file).unwrap();
        assert!(text.contains("files_action = \"reply\""), "{text}");
        assert!(text.contains("files_large = \"cloud\""), "{text}");
        let prefs = FilesPrefs {
            action: FilesAction::New,
            large: FilesLarge::Attach,
            limit_mb: 3,
        };
        assert_eq!(prefs.limit_bytes(), 3_000_000);
    }
}
