//! Left pane: one section per account, with the merged Favorites, Tags and
//! Smart Folders rows placed above or below them. Each account has a colored avatar circle + header button (chevron
//! on the right) and an animated `gtk::Revealer` holding its folder list, so
//! expanding/collapsing slides smoothly. Exactly one thing is selected across
//! the merged rows and all account folder lists.
//!
//! Account order is managed in the Accounts window, not here; folder order is
//! set here (a folder dragged between its siblings) and handed to the app to keep, which sends it back in `SectionData`. Collapse state is
//! owned by the app (persisted); collapse is animated locally and reported for
//! persistence WITHOUT a full rebuild (which would interrupt the animation).

use std::collections::{HashMap, HashSet};

use adw::prelude::*;
use relm4::prelude::*;

use crate::i18n::{i18n, i18n_f, i18n_noop};
use crate::models::{Account, Folder, FolderKind};
use crate::ui::context_menu::{show_context_menu_with_header, MenuEntry};

mod account_section;
mod contents;
mod dnd;
mod menus;
mod messages;
mod open_state;
mod picks;
mod rail;
mod rebuild;
mod selection;
mod tree;
mod types;
mod unified;
mod widgets;

pub(crate) use messages::{CtxAction, SidebarInput, SidebarOutput};
pub(crate) use tree::Hierarchy;
pub(crate) use tree::{folder_depth, order_folders, placed_run, TREE_EXPANDER_WIDTH};
use types::{KindWidgets, SectionWidgets, UnifiedRow};
pub(crate) use types::{SectionData, Sel, SidebarInit, Slot, StartTarget, UnifiedFolderRef};
pub(crate) use widgets::{account_initials, build_unified_sub_row, pin_icon_size};

pub(crate) struct Sidebar {
    /// Last sections received, in display order.
    sections: Vec<SectionData>,
    /// Whether the disclosure chevrons LEAD their rows (Settings: Chevron
    /// placement). Off restores the classic trailing position.
    chevrons_left: bool,
    /// Per-account widgets, rebuilt on each SetContents.
    revealers: HashMap<u32, gtk::Revealer>,
    chevrons: HashMap<u32, gtk::Image>,
    folder_lists: HashMap<u32, gtk::ListBox>,
    /// Per-account list box holding just the custom (user-created) folders, shown
    /// under a collapsible "Folders" section. Selection indices into these are
    /// offset past the account's essential folders.
    custom_folder_lists: HashMap<u32, gtk::ListBox>,
    /// Per-account custom folders, in row order, for tree-visibility math.
    custom_folders: HashMap<u32, Vec<Folder>>,
    /// Collapsed tree nodes per account (paths), mirrored from SectionData and
    /// flipped locally as chevrons are clicked (#51).
    tree_collapsed: HashMap<u32, std::collections::HashSet<String>>,
    /// Each parent row's expander image, for flipping without a rebuild.
    tree_chevrons: HashMap<(u32, String), gtk::Image>,
    /// Per-account custom-row revealers (row order), for animated tree
    /// collapse/expand.
    tree_row_revealers: HashMap<u32, Vec<gtk::Revealer>>,
    /// The folders inside the Inbox, listed under its row (#345): each
    /// account's main-list folders in row order, and the revealers of the
    /// nested ones by row.
    main_folders: HashMap<u32, Vec<Folder>>,
    main_row_revealers: HashMap<u32, HashMap<usize, gtk::Revealer>>,
    /// The rebuild freeze-frame Picture and its pending lift timer.
    freeze_frame: Option<gtk::Picture>,
    freeze_timer: std::rc::Rc<std::cell::RefCell<Option<gtk::glib::SourceId>>>,
    /// The freeze-frame's fade during a rail toggle (see `rebuild_normal`).
    freeze_fade: std::rc::Rc<std::cell::RefCell<Option<adw::TimedAnimation>>>,
    /// The rail state the rows on screen were built for, so a rebuild can
    /// tell a full ↔ rail toggle from an in-place refresh.
    built_collapsed: bool,
    /// The "Folders" section revealer and its chevron, per account.
    custom_revealers: HashMap<u32, gtk::Revealer>,
    custom_chevrons: HashMap<u32, gtk::Image>,
    /// Refresh/spinner stack + spinner beside the "New Message" button.
    sync_stack: Option<gtk::Stack>,
    sync_spinner: Option<gtk::Spinner>,
    /// Whether any account is syncing (drives the refresh spinner).
    busy: bool,
    /// Whether the network is down (turns the refresh button orange).
    offline: bool,
    /// The rail's refresh button, for its offline face.
    sync_button: Option<gtk::Button>,
    /// The "Outbox" row list box (one row), while anything is queued.
    outbox_list: Option<gtk::ListBox>,
    /// Display-wide provider holding each account's avatar color rules.
    color_provider: crate::ui::DisplayCss,
    selected: Sel,
    /// Icon-only mode: hide all text, show just icons and account pills.
    collapsed: bool,
    /// Set while rows are selected *programmatically* (restoring after a
    /// rebuild, following the app's navigation): the list boxes' selection
    /// signals then stay silent. Otherwise a signal-driven input would be
    /// queued and judged against a selection that has since moved on — two
    /// programmatic selections in a row used to oscillate forever through
    /// the app that way.
    quiet: std::rc::Rc<std::cell::Cell<bool>>,
    /// See `SidebarInit::mirror`.
    mirror: bool,
    /// The launch view still to be picked (#256): kept until the account
    /// it names has its folders listed, then taken by the first pick.
    start: Option<StartTarget>,
    /// The view to pick again, and announce, once its account is listed
    /// again: the app dropped what it showed when the accounts reconnected
    /// (an account saved), while this list still showed it selected. The
    /// account is held by address: ids follow the account order, which a
    /// removal shifts.
    reannounce: Option<(Sel, Option<String>)>,
    /// How many messages are waiting in the Outbox across all accounts. The row
    /// only exists while this is non-zero — an empty Outbox is the normal state
    /// and does not deserve permanent furniture.
    outbox_count: u32,
    /// Unread badge labels by (account_id, folder_id), updated in place.
    folder_badges: HashMap<(u32, u32), gtk::Label>,
    /// The favorite folders (#367), in the order they were added.
    favorite_folders: Vec<UnifiedFolderRef>,
    favorites_expanded: bool,
    /// The smart folders (#417) as (id, name), in their saved order, with
    /// each one's unread count and the badge showing it.
    smart_folders: Vec<(String, String)>,
    smart_expanded: bool,
    smart_unread: HashMap<String, u32>,
    smart_badges: HashMap<String, gtk::Label>,
    /// Each tag's unread mail across the accounts in the combined views,
    /// by keyword, and the badge showing it.
    tag_unread: HashMap<String, u32>,
    tag_badges: HashMap<String, gtk::Label>,
    /// Inbox unread badge overlaid on each account's avatar circle, shown only
    /// while that account's section is collapsed (its Inbox row — and normal
    /// chip — is then hidden inside the revealer). Keyed by account_id.
    account_circle_badges: HashMap<u32, gtk::Label>,
    /// The tags (#71), listed in the unified Tags section and in each
    /// account's own; the sections exist only while there is a tag.
    tags: Vec<crate::config::Tag>,
    /// Whether the unified "Tags" section is open.
    tags_expanded: bool,
    tag_sections: HashMap<Slot, SectionWidgets>,
    /// The rail's own open states: anything opened or folded while the
    /// sidebar is the icon rail lands here, the saved state untouched, so
    /// the full sidebar comes back as it was left. Cleared whenever the
    /// sidebar changes width. An item absent here shows its saved state —
    /// or folded, when "Fold up expanded items" covers it.
    rail_open: HashMap<UnifiedRow, bool>,
    rail_open_accounts: HashMap<u32, bool>,
    /// The Favorites, Tags and Smart Folders rows' widgets.
    kind_widgets: HashMap<UnifiedRow, KindWidgets>,
}

#[relm4::component(pub(crate))]
impl Component for Sidebar {
    type Init = SidebarInit;
    type Input = SidebarInput;
    type Output = SidebarOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "body_overlay"]
            gtk::Overlay {
                set_vexpand: true,

                // The top block (the compose bar) is pinned above
                // the scroller so it never scrolls away; only the per-account
                // sections below scroll.
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "pinned_box"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                    },

                    #[name = "sidebar_scroller"]
                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        // External, not Never (see the message list's scroller): row
                        // content — a deeply indented folder tree, say — must not force
                        // the window's minimum width past what edge-tiling allows. The
                        // split view's min/max sidebar widths govern instead.
                        set_hscrollbar_policy: gtk::PolicyType::External,

                        #[name = "normal_box"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                        },
                    },

                },

                // Freeze-frame for rebuilds: the sidebar's last-rendered pixels,
                // shown over the swap so recreating every row never shimmers.
                // Anchored at the start and sized to the pixels it holds, so
                // the pane animating to or from the rail beneath it can never
                // stretch or squash the picture (the overlay clips it instead).
                #[name = "freeze_frame"]
                add_overlay = &gtk::Picture {
                    set_visible: false,
                    set_can_target: false,
                    set_halign: gtk::Align::Start,
                    set_content_fit: gtk::ContentFit::Fill,
                },
            },

        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let color_provider = crate::ui::DisplayCss::new();

        let mut model = Sidebar {
            sections: Vec::new(),
            chevrons_left: false,
            revealers: HashMap::new(),
            chevrons: HashMap::new(),
            folder_lists: HashMap::new(),
            custom_folder_lists: HashMap::new(),
            custom_folders: HashMap::new(),
            tree_collapsed: HashMap::new(),
            tree_chevrons: HashMap::new(),
            tree_row_revealers: HashMap::new(),
            main_folders: HashMap::new(),
            main_row_revealers: HashMap::new(),
            freeze_frame: None,
            freeze_timer: std::rc::Rc::new(std::cell::RefCell::new(None)),
            freeze_fade: std::rc::Rc::new(std::cell::RefCell::new(None)),
            built_collapsed: init.collapsed,
            custom_revealers: HashMap::new(),
            custom_chevrons: HashMap::new(),
            sync_stack: None,
            sync_spinner: None,
            busy: false,
            offline: !crate::desktop::power::online(),
            sync_button: None,
            outbox_list: None,
            color_provider,
            selected: Sel::None,
            collapsed: init.collapsed,
            quiet: std::rc::Rc::new(std::cell::Cell::new(false)),
            mirror: init.mirror,
            start: if init.mirror { None } else { init.start },
            reannounce: None,
            outbox_count: 0,
            folder_badges: HashMap::new(),
            favorite_folders: Vec::new(),
            favorites_expanded: init.favorites_expanded,
            smart_folders: Vec::new(),
            smart_expanded: init.smart_expanded,
            smart_unread: HashMap::new(),
            smart_badges: HashMap::new(),
            tag_unread: HashMap::new(),
            tag_badges: HashMap::new(),
            account_circle_badges: HashMap::new(),
            tags: Vec::new(),
            tags_expanded: init.tags_expanded,
            tag_sections: HashMap::new(),
            kind_widgets: HashMap::new(),
            rail_open: HashMap::new(),
            rail_open_accounts: HashMap::new(),
        };

        let widgets = view_output!();
        model.freeze_frame = Some(widgets.freeze_frame.clone());
        widgets
            .body_overlay
            .set_clip_overlay(&widgets.freeze_frame, true);
        // Never scroll-to-focus: a rebuild (folder drag-and-drop) destroys
        // the focused row, GTK hands focus to some early widget, and the
        // viewport would yank the sidebar to the top to show it — the
        // "jumps up then back down" on every drop. Sidebar scrolling is the
        // user's alone.
        if let Some(viewport) = widgets
            .sidebar_scroller
            .child()
            .and_downcast::<gtk::Viewport>()
        {
            viewport.set_scroll_to_focus(false);
        }

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.update_inner(widgets, msg, sender, root);
    }
}

impl Sidebar {
    fn update_inner(
        &mut self,
        widgets: &mut <Self as Component>::Widgets,
        msg: SidebarInput,
        sender: ComponentSender<Self>,
        _root: &<Self as Component>::Root,
    ) {
        match msg {
            SidebarInput::DropStart => self.on_drop_start(),
            SidebarInput::Reannounce => self.on_reannounce(sender),
            SidebarInput::MirrorSelection(sel) => self.on_mirror_selection(sel),
            msg @ SidebarInput::SetContents { .. } => self.on_set_contents(msg, widgets, sender),
            SidebarInput::OutboxRowSelected => self.on_outbox_row_selected(sender),
            SidebarInput::SetOutboxCount(count) => self.on_set_outbox_count(count, widgets, sender),
            SidebarInput::TagRowSelected { slot, index } => {
                self.on_tag_row_selected(slot, index, sender)
            }
            // The unified slot is a unified row; an account's is a heading.
            SidebarInput::ToggleTagsExpand(slot) => self.on_toggle_tags_expand(slot, sender),
            SidebarInput::ToggleFavoritesExpand => {
                self.toggle_row(UnifiedRow::Favorites, &sender);
            }
            SidebarInput::ToggleSmartExpand => self.on_toggle_smart_expand(sender),
            SidebarInput::SmartRowSelected(index) => self.on_smart_row_selected(index, sender),
            SidebarInput::SetSmartUnread(counts) => self.on_set_smart_unread(counts),
            SidebarInput::SetTagUnread(counts) => self.on_set_tag_unread(counts),
            SidebarInput::FavoriteRowSelected(index) => {
                self.on_favorite_row_selected(index, sender)
            }
            SidebarInput::UnifiedFavoritesSelected => self.on_unified_favorites_selected(sender),
            SidebarInput::UnifiedTagsSelected => self.on_unified_tags_selected(sender),
            SidebarInput::FolderRowSelected { account_id, index } => {
                self.on_folder_row_selected(account_id, index, sender)
            }
            SidebarInput::SelectFolderRow { account_id, path } => {
                self.on_select_folder_row(account_id, path, sender)
            }
            SidebarInput::SetUnread { folders } => self.on_set_unread(folders),
            SidebarInput::SetBusy(busy) => self.on_set_busy(busy),
            SidebarInput::SetOffline(offline) => self.on_set_offline(offline),
            SidebarInput::SetCollapsed(collapsed) => {
                self.on_set_collapsed(collapsed, widgets, sender)
            }
            SidebarInput::ToggleCollapsed => self.on_toggle_collapsed(widgets, sender),
            SidebarInput::ToggleCollapseLocal(id) => self.on_toggle_collapse_local(id, sender),
            SidebarInput::ToggleCustomFoldersLocal(id) => {
                self.on_toggle_custom_folders_local(id, sender)
            }
            SidebarInput::ToggleFolderNode { account_id, path } => {
                self.toggle_folder_node(account_id, path, &sender);
            }
            SidebarInput::ReorderFolder {
                account_id,
                path,
                target,
                after,
            } => self.on_reorder_folder(account_id, path, target, after, sender),
            SidebarInput::ExpandForDrop(id) => self.on_expand_for_drop(id, sender),
            SidebarInput::DropOnFolder {
                account_id: dest_account,
                path: dest,
                payload,
            } => self.on_drop_on_folder(dest_account, dest, payload, sender),
        }
    }
}

#[cfg(test)]
mod tests;
