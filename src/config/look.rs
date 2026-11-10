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
    /// The columns a single line shows: the ones it had before they could
    /// be chosen, and the replied arrows (#454). The choice went with the
    /// list's settings (docs/DESIGN.md, "Removed or moved").
    pub(crate) const DEFAULT: [ListColumn; 7] = [
        ListColumn::Star,
        ListColumn::Replied,
        ListColumn::Sender,
        ListColumn::Subject,
        ListColumn::Tags,
        ListColumn::Attachment,
        ListColumn::Date,
    ];
}

/// Lines of message text shown under the subject in the list; 0 means previews
/// are off. Clamped in case the file was edited by hand.
pub(crate) fn load_preview_lines() -> u32 {
    load_privacy().preview_lines.min(3)
}
