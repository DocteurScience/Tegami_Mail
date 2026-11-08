//! The sidebar's data: what the app hands it per account, what is
//! selected, and the unified rows it draws.

use super::*;

/// A folder listed in a unified section (Favorites, #367): the account it
/// belongs to and the folder.
#[derive(Debug, Clone)]
pub(crate) struct UnifiedFolderRef {
    pub(crate) account_id: u32,
    pub(crate) folder: Folder,
}

/// Which copy of the Tags section a widget belongs to:
/// the unified section's, or an account's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    Unified,
    Account(u32),
}

impl Slot {
    /// The account a section is scoped to; `None` for the unified one.
    pub(super) fn account(self) -> Option<u32> {
        match self {
            Slot::Unified => None,
            Slot::Account(id) => Some(id),
        }
    }
}

/// An account's Tags section's widgets: rebuilt with the sidebar, toggled
/// in place between rebuilds.
pub(super) struct SectionWidgets {
    pub(super) revealer: gtk::Revealer,
    pub(super) chevron: gtk::Image,
    pub(super) list: gtk::ListBox,
}

/// One merged row (Favorites, Tags, Smart Folders): the header list (one
/// selectable row), the list under it, and their badges.
pub(super) struct KindWidgets {
    pub(super) header: gtk::ListBox,
    pub(super) revealer: gtk::Revealer,
    pub(super) chevron: Option<gtk::Image>,
    pub(super) badge: Option<gtk::Label>,
    pub(super) list: gtk::ListBox,
    pub(super) row_badges: HashMap<(u32, u32), gtk::Label>,
}

/// A row built like a section of its own: Favorites, the Tags row (every
/// tag) and the smart folders — all built alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum UnifiedRow {
    /// The folders picked with Add to Favorites (#367), from any account.
    Favorites,
    Tags,
    /// The smart folders (#417): saved searches, each a row of the list.
    Smart,
}

/// What a unified row is called.
pub(super) fn row_title(row: UnifiedRow) -> String {
    match row {
        UnifiedRow::Favorites => i18n("Favorites"),
        UnifiedRow::Tags => i18n("Tags"),
        UnifiedRow::Smart => i18n("Smart folders"),
    }
}

/// A unified row's icon.
pub(super) fn row_icon(row: UnifiedRow) -> &'static str {
    match row {
        UnifiedRow::Favorites => "user-bookmarks-symbolic",
        UnifiedRow::Tags => "tag-outline-symbolic",
        UnifiedRow::Smart => "folder-saved-search-symbolic",
    }
}

/// The message a unified row's header sends when selected.
pub(super) fn select_msg(row: UnifiedRow) -> SidebarInput {
    match row {
        UnifiedRow::Favorites => SidebarInput::UnifiedFavoritesSelected,
        UnifiedRow::Tags => SidebarInput::UnifiedTagsSelected,
        // Each smart folder is a view of its own; the header holds them.
        UnifiedRow::Smart => SidebarInput::ToggleSmartExpand,
    }
}

/// The message a unified row's chevron (or, in the rail, a long press) sends.
pub(super) fn toggle_msg(row: UnifiedRow) -> SidebarInput {
    match row {
        UnifiedRow::Favorites => SidebarInput::ToggleFavoritesExpand,
        UnifiedRow::Tags => SidebarInput::ToggleTagsExpand(Slot::Unified),
        UnifiedRow::Smart => SidebarInput::ToggleSmartExpand,
    }
}

/// The chevron glyph for a section that is open or folded.
pub(super) fn chevron_icon(open: bool) -> &'static str {
    if open {
        "pan-down-symbolic"
    } else {
        "pan-end-symbolic"
    }
}

/// One account's section data, as handed to the sidebar.
#[derive(Debug, Clone)]
pub(crate) struct SectionData {
    pub(crate) account: Account,
    pub(crate) folders: Vec<Folder>,
    pub(crate) collapsed: bool,
    /// Whether this account's custom-folders section is expanded (default hidden).
    pub(crate) custom_expanded: bool,
    /// Resolved avatar background color ("#rrggbb").
    pub(crate) color: String,
    /// Custom-folder paths whose tree node is collapsed (#51).
    pub(crate) tree_collapsed: Vec<String>,
    /// The user's own folder order (paths): a folder sorts among its
    /// siblings by where it stands here, unlisted ones after, as the server
    /// sorted them. Empty keeps the default order.
    pub(crate) folder_order: Vec<String>,
    /// How the server spells its folder hierarchy, to tell a folder's
    /// siblings on the server from the folders merely drawn beside it.
    pub(crate) hierarchy: Hierarchy,
    /// How the custom folders are sorted: the account's own choice
    /// (`own_folder_sort`) or else Settings' (`default_folder_sort`).
    pub(crate) folder_sort: crate::config::FolderSort,
    pub(crate) own_folder_sort: Option<crate::config::FolderSort>,
    pub(crate) default_folder_sort: crate::config::FolderSort,
    /// The folders this account's filter rules file into (every rule's
    /// destination).
    pub(crate) filtered: Vec<Folder>,
    /// Whether this account has any filter rule at all: only then is
    /// "Apply Filters" worth offering on its folders (#198).
    pub(crate) has_filters: bool,
    /// Whether this account's own "Tags" section is open.
    pub(crate) tags_expanded: bool,
    /// A Gmail account, whose own folders are labels and say so (#374).
    pub(crate) gmail: bool,
}

/// Initial state for the sidebar.
pub(crate) struct SidebarInit {
    /// Icon-only mode: hide all text, show just icons and account pills.
    pub(crate) collapsed: bool,
    /// A mirror instance (the floating peek panel): it never picks a view
    /// on its own when nothing is selected — the app hands it the primary
    /// sidebar's choice through `SidebarInput::MirrorSelection`.
    pub(crate) mirror: bool,
    /// The sections' open state as last left (persisted with the sidebar
    /// layout).
    pub(crate) tags_expanded: bool,
    pub(crate) favorites_expanded: bool,
    pub(crate) smart_expanded: bool,
    /// Where the first pick lands instead of the first Inbox (#256).
    pub(crate) start: Option<StartTarget>,
}

/// A view to open at launch (#256), by account address: an account's
/// inbox, or one of its folders by path.
#[derive(Clone, Debug)]
pub(crate) enum StartTarget {
    Inbox(String),
    Folder(String, String),
}

/// What is currently selected in the sidebar.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Sel {
    None,
    Outbox,
    Folder(u32, String),
    /// The Favorites row: every favorite folder, merged.
    UnifiedFavorites,
    /// The unified Tags row: every tagged message.
    UnifiedTags,
    /// A folder (account, path) picked in the Favorites list (#367).
    Favorite(u32, String),
    /// A tag (its keyword) selected in a Tags section (#71): scoped to an
    /// account when picked from that account's own section.
    Tag(Option<u32>, String),
    /// A smart folder (#417), by its id.
    Smart(String),
}

impl Sel {
    /// The account the selection belongs to, if it belongs to one.
    pub(super) fn account(&self) -> Option<u32> {
        match self {
            Sel::Folder(acc, _) | Sel::Tag(Some(acc), _) | Sel::Favorite(acc, _) => Some(*acc),
            _ => None,
        }
    }

    /// The same selection for the account now numbered `id`.
    pub(super) fn with_account(self, id: u32) -> Sel {
        match self {
            Sel::Folder(_, p) => Sel::Folder(id, p),
            Sel::Tag(Some(_), k) => Sel::Tag(Some(id), k),
            Sel::Favorite(_, p) => Sel::Favorite(id, p),
            other => other,
        }
    }
}
