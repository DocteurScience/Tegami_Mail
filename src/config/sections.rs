//! The sidebar's sections: where each section is placed, the start view,
//! folder order and the icon rail's fold-ups.

use super::default_on;

/// Where a sidebar section (Filters, Tags, Favorites, Contacts) is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SectionPlacement {
    /// In the unified section, as older files have it. Only
    /// ever read: loading moves it (see `LegacySwitches`), and a file
    /// missing the key reads as this so that its old switch decides.
    #[default]
    AllInboxes,
    /// In the scrolling sidebar, above the first account.
    AboveAccounts,
    /// In the scrolling sidebar, after the last account.
    BelowAccounts,
    /// A row of the pinned footer, above Attachments. Only Contacts
    /// offers it, and has it by default.
    Footer,
    /// Not in the sidebar at all.
    Off,
}

/// How Filters, Tags, Favorites and Contacts are drawn (Settings → Sidebar → Sections).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SectionStyle {
    /// As the unified section's rows: a pill whose header opens the
    /// combined view, with a chevron on the right.
    #[default]
    Unified,
    /// As a tree: the caret that opens and closes the list in the icon's
    /// place, no chevron at the row's end; the name still opens the view.
    #[serde(alias = "headings")]
    Tree,
}

impl SectionPlacement {
    /// Whether the section is in the sidebar's scrolling part, where it
    /// lists what it holds under its row.
    pub(crate) fn shown(self) -> bool {
        matches!(self, Self::AboveAccounts | Self::BelowAccounts)
    }
}

/// What the main window shows at launch (#256).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StartView {
    /// The first account's inbox. Named after the All Inboxes view it
    /// opened before that view went, so saved settings still read.
    #[default]
    AllInboxes,
    /// The inbox of the account whose mail was open last.
    AccountInbox,
    /// Whichever folder was open last.
    LastFolder,
}

/// How an account's custom folders are ordered in the sidebar: Settings →
/// Sidebar sets it for every account, and an account can choose its own
/// (`AccountConfig::folder_sort`). The main folders (Inbox, Sent…) keep
/// their order whichever is chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FolderSort {
    /// By name, with the folders the user dragged kept where they were put.
    #[default]
    Custom,
    /// By the name shown, A to Z, ignoring any dragging.
    NameAsc,
    /// By the name shown, Z to A.
    NameDesc,
    /// By the whole path on the server, as Gmail on the web lists labels.
    Path,
}

impl FolderSort {
    pub(crate) const ALL: [FolderSort; 4] = [
        FolderSort::Custom,
        FolderSort::NameAsc,
        FolderSort::NameDesc,
        FolderSort::Path,
    ];

    pub(crate) fn label(self) -> String {
        match self {
            FolderSort::Custom => crate::i18n::i18n("Custom Order"),
            FolderSort::NameAsc => crate::i18n::i18n("Name (A to Z)"),
            FolderSort::NameDesc => crate::i18n::i18n("Name (Z to A)"),
            FolderSort::Path => crate::i18n::i18n("Full Path"),
        }
    }

    /// Its place in [`FolderSort::ALL`], for combo rows.
    pub(crate) fn index(self) -> u32 {
        FolderSort::ALL.iter().position(|s| *s == self).unwrap_or(0) as u32
    }
}

/// "Fold up expanded items" (Settings → Sidebar → Icon rail): while the
/// sidebar is collapsed to its icon rail, the items ticked here show folded
/// up and stay so — their saved expansion is untouched and returns when the
/// sidebar expands. Each item has its own switch under the master; all are
/// on until switched off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RailFold {
    /// The master switch: off, nothing folds for the rail.
    #[serde(default = "default_on")]
    pub(crate) enabled: bool,
    /// Every account's folder list.
    #[serde(default = "default_on")]
    pub(crate) accounts: bool,
    /// The Tags section.
    #[serde(default = "default_on")]
    pub(crate) tags: bool,
}

impl Default for RailFold {
    fn default() -> Self {
        RailFold {
            enabled: true,
            accounts: true,
            tags: true,
        }
    }
}

impl RailFold {
    /// Whether the accounts fold for the rail.
    pub(crate) fn folds_accounts(self) -> bool {
        self.enabled && self.accounts
    }

    pub(crate) fn folds_tags(self) -> bool {
        self.enabled && self.tags
    }
}
