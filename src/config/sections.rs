//! The sidebar's sections: where each section is placed, the start view,
//! folder order and the icon rail's fold-ups.

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
            FolderSort::Custom => crate::i18n::i18n("Custom order"),
            FolderSort::NameAsc => crate::i18n::i18n("Name (A to Z)"),
            FolderSort::NameDesc => crate::i18n::i18n("Name (Z to A)"),
            FolderSort::Path => crate::i18n::i18n("Full path"),
        }
    }

    /// Its place in [`FolderSort::ALL`], for combo rows.
    pub(crate) fn index(self) -> u32 {
        FolderSort::ALL.iter().position(|s| *s == self).unwrap_or(0) as u32
    }
}
