//! How the app looks: theme, text size, and the message list's layout and columns.

use serde::{Deserialize, Serialize};

use super::privacy::load_privacy;

/// The app's own theme: follow the system, or force light/dark regardless.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AppTheme {
    #[default]
    System,
    Light,
    Dark,
}

/// Where the reader sits (#475): beside the list, below it, or nowhere,
/// messages then opening in windows of their own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", from = "ReadingPaneSaved")]
pub(crate) enum ReadingPane {
    #[default]
    Right,
    Below,
    Off,
}

/// What a settings file may hold: the pane's name, or the on/off switch
/// that came before Below and was never released.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ReadingPaneSaved {
    Name(String),
    On(bool),
}

impl From<ReadingPaneSaved> for ReadingPane {
    fn from(saved: ReadingPaneSaved) -> Self {
        match saved {
            ReadingPaneSaved::Name(key) => ReadingPane::from_key(&key).unwrap_or_default(),
            ReadingPaneSaved::On(true) => ReadingPane::Right,
            ReadingPaneSaved::On(false) => ReadingPane::Off,
        }
    }
}

impl ReadingPane {
    /// The main menu's name for it, the action's state.
    pub(crate) fn key(self) -> &'static str {
        match self {
            ReadingPane::Right => "right",
            ReadingPane::Below => "below",
            ReadingPane::Off => "off",
        }
    }

    pub(crate) fn from_key(key: &str) -> Option<Self> {
        [ReadingPane::Right, ReadingPane::Below, ReadingPane::Off]
            .into_iter()
            .find(|p| p.key() == key)
    }
}

/// One column of the single-line message list (#334).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ListColumn {
    Star,
    /// Replied to, forwarded, or both (#454).
    Replied,
    Sender,
    Recipients,
    Correspondents,
    Subject,
    Tags,
    Attachment,
    Importance,
    Account,
    /// A Microsoft 365 follow-up flag's due date, the one kind of mail
    /// that has one: the column takes no room in a list without any.
    Due,
    Date,
}

impl ListColumn {
    pub(crate) const ALL: [ListColumn; 12] = [
        ListColumn::Star,
        ListColumn::Replied,
        ListColumn::Sender,
        ListColumn::Recipients,
        ListColumn::Correspondents,
        ListColumn::Subject,
        ListColumn::Tags,
        ListColumn::Attachment,
        ListColumn::Importance,
        ListColumn::Account,
        ListColumn::Due,
        ListColumn::Date,
    ];

    /// The columns a single line shows until they are changed: the ones it
    /// had before they could be, and the replied arrows (#454).
    pub(crate) const DEFAULT: [ListColumn; 7] = [
        ListColumn::Star,
        ListColumn::Replied,
        ListColumn::Sender,
        ListColumn::Subject,
        ListColumn::Tags,
        ListColumn::Attachment,
        ListColumn::Date,
    ];

    /// The stable name written to `privacy.toml`.
    pub(crate) fn key(self) -> &'static str {
        match self {
            ListColumn::Star => "star",
            ListColumn::Replied => "replied",
            ListColumn::Sender => "sender",
            ListColumn::Recipients => "recipients",
            ListColumn::Correspondents => "correspondents",
            ListColumn::Subject => "subject",
            ListColumn::Tags => "tags",
            ListColumn::Attachment => "attachment",
            ListColumn::Importance => "importance",
            ListColumn::Account => "account",
            ListColumn::Due => "due",
            ListColumn::Date => "date",
        }
    }

    pub(crate) fn from_key(key: &str) -> Option<ListColumn> {
        ListColumn::ALL.iter().copied().find(|c| c.key() == key)
    }

    /// The untranslated label (callers pass it through `i18n`).
    pub(crate) fn label(self) -> &'static str {
        match self {
            ListColumn::Star => crate::i18n::i18n_noop("Star"),
            ListColumn::Replied => crate::i18n::i18n_noop("Replied"),
            ListColumn::Sender => crate::i18n::i18n_noop("Sender"),
            ListColumn::Recipients => crate::i18n::i18n_noop("Recipients"),
            ListColumn::Correspondents => crate::i18n::i18n_noop("Correspondents"),
            ListColumn::Subject => crate::i18n::i18n_noop("Subject"),
            ListColumn::Tags => crate::i18n::i18n_noop("Tags"),
            ListColumn::Attachment => crate::i18n::i18n_noop("Attachment"),
            ListColumn::Importance => crate::i18n::i18n_noop("Importance"),
            ListColumn::Account => crate::i18n::i18n_noop("Account"),
            ListColumn::Due => crate::i18n::i18n_noop("Due date"),
            ListColumn::Date => crate::i18n::i18n_noop("Date"),
        }
    }

    /// The shown columns, in order, from their saved names. Names this
    /// version does not know are skipped, and the subject is always there:
    /// it is the column that takes the room the others leave.
    pub(crate) fn from_keys(keys: &[String]) -> Vec<ListColumn> {
        let mut out: Vec<ListColumn> = Vec::new();
        for c in keys.iter().filter_map(|k| ListColumn::from_key(k)) {
            if !out.contains(&c) {
                out.push(c);
            }
        }
        if !out.contains(&ListColumn::Subject) {
            let at = out
                .iter()
                .position(|c| *c == ListColumn::Sender)
                .map(|i| i + 1)
                .unwrap_or(0);
            out.insert(at, ListColumn::Subject);
        }
        out
    }

    pub(crate) fn to_keys(columns: &[ListColumn]) -> Vec<String> {
        columns.iter().map(|c| c.key().to_string()).collect()
    }
}

pub(super) fn default_list_columns() -> Vec<String> {
    ListColumn::to_keys(&ListColumn::DEFAULT)
}

/// The widths single-line columns were dragged to (#334), by column.
pub(crate) fn load_list_column_widths() -> std::collections::HashMap<ListColumn, i32> {
    list_column_widths_from(&load_privacy().list_column_widths)
}

pub(super) fn list_column_widths_from(
    saved: &std::collections::BTreeMap<String, i32>,
) -> std::collections::HashMap<ListColumn, i32> {
    saved
        .iter()
        .filter_map(|(k, px)| Some((ListColumn::from_key(k)?, (*px).clamp(32, 800))))
        .collect()
}

pub(crate) fn list_column_widths_to(
    widths: &std::collections::HashMap<ListColumn, i32>,
) -> std::collections::BTreeMap<String, i32> {
    widths
        .iter()
        .map(|(c, px)| (c.key().to_string(), *px))
        .collect()
}

/// The single-line list's columns, in order (#334).
pub(crate) fn load_list_columns() -> Vec<ListColumn> {
    ListColumn::from_keys(&load_privacy().list_columns)
}

/// Lines of message text shown under the subject in the list; 0 means previews
/// are off. Clamped in case the file was edited by hand.
pub(crate) fn load_preview_lines() -> u32 {
    load_privacy().preview_lines.min(3)
}
