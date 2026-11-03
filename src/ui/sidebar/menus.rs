//! The sidebar's right-click menus and the folder sort menu.

use super::*;

/// The right-click menu of an account's Folders heading: how its folders
/// are sorted (its own choice, or Settings'), and Reset Folder Order when
/// it has one of its own.
pub(super) fn folder_sort_menu(
    section: &SectionData,
    sender: &ComponentSender<Sidebar>,
) -> Vec<Vec<MenuEntry>> {
    use crate::config::FolderSort;
    let id = section.account.id;
    let own = section.own_folder_sort;
    let pick = |label: String, sort: Option<FolderSort>| {
        let s = sender.clone();
        MenuEntry::new(label, move || {
            let _ = s.output(SidebarOutput::SetFolderSort {
                account_id: id,
                sort,
            });
        })
        .selected(own == sort)
    };
    let follow = i18n_f(
        "Follow settings: {sort}",
        &[("sort", &section.default_folder_sort.label())],
    );
    let mut sections = vec![
        vec![pick(follow, None)],
        FolderSort::ALL
            .iter()
            .map(|&sort| pick(sort.label(), Some(sort)))
            .collect(),
    ];
    if !section.folder_order.is_empty() {
        let s = sender.clone();
        sections.push(vec![MenuEntry::new(
            i18n("Reset folder order"),
            move || {
                let _ = s.output(SidebarOutput::Context(CtxAction::ResetFolderOrder(id)));
            },
        )]);
    }
    sections
}

/// The context menu of one folder, the same wherever the folder is listed
/// (under its account, or among Favorites), in the design's sections
/// (docs/DESIGN.md, "Menus"; mockup 06): Mark All as Read and Sync This
/// Folder; New Subfolder and Rename; what else this folder offers; Delete
/// Folder alone at the end. `filtered` says a filter rule files into it.
pub(super) fn folder_menu_items(
    id: u32,
    f: &Folder,
    filtered: bool,
    has_filters: bool,
    favorite: bool,
) -> Vec<Vec<(&'static str, CtxAction)>> {
    let custom = f.kind == FolderKind::Custom;
    let first = vec![
        (
            i18n_noop("Mark all as read"),
            CtxAction::MarkFolderRead {
                account_id: id,
                folder_id: f.id,
            },
        ),
        (
            i18n_noop("Sync this folder"),
            CtxAction::RefreshFolder {
                account_id: id,
                folder_id: f.id,
            },
        ),
    ];
    let mut shape = Vec::new();
    // Starred is a search over every folder on Gmail, not a mailbox that
    // can hold others; Snoozed is mail on its way back.
    if !matches!(f.kind, FolderKind::Starred | FolderKind::Snoozed) {
        shape.push((
            i18n_noop("New subfolder…"),
            CtxAction::NewSubfolder {
                account_id: id,
                parent: f.path.clone(),
            },
        ));
    }
    // Only user-created folders can be renamed or deleted.
    if custom {
        shape.push((
            i18n_noop("Rename…"),
            CtxAction::RenameFolder {
                account_id: id,
                name: f.name.clone(),
                path: f.path.clone(),
            },
        ));
    }
    let mut more = Vec::new();
    // Favorites (#367): any folder can be kept at the top of the sidebar.
    more.push(if favorite {
        (
            i18n_noop("Remove from favorites"),
            CtxAction::RemoveFavorite {
                account_id: id,
                path: f.path.clone(),
            },
        )
    } else {
        (
            i18n_noop("Add to favorites"),
            CtxAction::AddFavorite {
                account_id: id,
                path: f.path.clone(),
            },
        )
    });
    // Rules normally only meet mail arriving in the Inbox; from here they
    // can be held up against whatever is already in this folder (#198).
    // Not on Drafts, Templates, Junk or Trash: filing mail *out* of those is
    // never what a rule about incoming mail meant. Nor on Snoozed, whose
    // mail is on its way back to where it came from.
    let filterable = !matches!(
        f.kind,
        FolderKind::Drafts
            | FolderKind::Templates
            | FolderKind::Snoozed
            | FolderKind::Junk
            | FolderKind::Trash
            | FolderKind::Starred
    );
    if has_filters && filterable {
        more.push((
            i18n_noop("Apply filters"),
            CtxAction::ApplyFilters {
                account_id: id,
                folder_id: f.id,
            },
        ));
    }
    // A filter files into this folder: its rule is a click away.
    if filtered {
        more.push((
            i18n_noop("Edit filter…"),
            CtxAction::EditFilter {
                account_id: id,
                path: f.path.clone(),
            },
        ));
    }
    // Hidden (#239): only a plain folder, since a role folder out of sight
    // would still be where mail is filed to.
    if custom {
        more.push((
            i18n_noop("Hide folder"),
            CtxAction::HideFolder {
                account_id: id,
                path: f.path.clone(),
            },
        ));
    }
    // Trash and Junk can be emptied outright (#152).
    let empty_label = match f.kind {
        FolderKind::Trash => Some(i18n_noop("Empty trash…")),
        FolderKind::Junk => Some(i18n_noop("Empty junk…")),
        _ => None,
    };
    if let Some(label) = empty_label {
        more.push((
            label,
            CtxAction::EmptyFolder {
                account_id: id,
                folder_id: f.id,
                name: f.name.clone(),
                path: f.path.clone(),
            },
        ));
    }
    let mut sections = vec![first, shape, more];
    if custom {
        sections.push(vec![(
            i18n_noop("Delete folder"),
            CtxAction::DeleteFolder {
                account_id: id,
                name: f.name.clone(),
                path: f.path.clone(),
            },
        )]);
    }
    sections.retain(|s| !s.is_empty());
    sections
}

/// Right-click on a tag row, wherever tags are listed: edit it in Settings.
pub(super) fn attach_tag_context_menu(
    list: &gtk::ListBox,
    keywords: Vec<String>,
    sender: &ComponentSender<Sidebar>,
) {
    let click = gtk::GestureClick::new();
    click.set_button(gtk::gdk::BUTTON_SECONDARY);
    let cs = sender.clone();
    let list_w = list.clone();
    click.connect_pressed(move |_, _, x, y| {
        if let Some(k) = list_w
            .row_at_y(y as i32)
            .and_then(|row| keywords.get(row.index() as usize))
        {
            show_sidebar_menu(
                &list_w,
                x,
                y,
                vec![(i18n_noop("Edit tag…"), CtxAction::EditTag(k.clone()))],
                &cs,
            );
        }
    });
    list.add_controller(click);
}

/// Wire the folder menu ([`folder_menu_items`]) onto a folder list;
/// `folders` maps the list's row indices to their folders.
pub(super) fn attach_folder_context_menu(
    list: &gtk::ListBox,
    id: u32,
    folders: Vec<Folder>,
    filtered: Vec<String>,
    favorites: Vec<String>,
    has_filters: bool,
    sender: &ComponentSender<Sidebar>,
) {
    let click = gtk::GestureClick::new();
    click.set_button(gtk::gdk::BUTTON_SECONDARY);
    let cs = sender.clone();
    let list_w = list.clone();
    click.connect_pressed(move |_, _, x, y| {
        if let Some(f) = list_w
            .row_at_y(y as i32)
            .and_then(|row| folders.get(row.index() as usize))
        {
            let items = folder_menu_items(
                id,
                f,
                filtered.contains(&f.path),
                has_filters,
                favorites.contains(&f.path),
            );
            show_sidebar_menu_sections(&list_w, x, y, items, &cs);
        }
    });
    list.add_controller(click);
}

/// Pop up a right-click context menu of `items` anchored at (x, y) in
/// `parent`, styled to GNOME HIG (sized to content, no scrollbar).
pub(super) fn show_sidebar_menu(
    parent: &impl IsA<gtk::Widget>,
    x: f64,
    y: f64,
    items: Vec<(&str, CtxAction)>,
    sender: &ComponentSender<Sidebar>,
) {
    show_sidebar_menu_sections(parent, x, y, vec![items], sender);
}

/// [`show_sidebar_menu`] with the entries in sections, a separator between
/// each.
pub(super) fn show_sidebar_menu_sections(
    parent: &impl IsA<gtk::Widget>,
    x: f64,
    y: f64,
    sections: Vec<Vec<(&str, CtxAction)>>,
    sender: &ComponentSender<Sidebar>,
) {
    // The pick goes out once the menu has closed: an action that rebuilds
    // the sidebar would otherwise take the menu's parent list away while
    // the menu is still closing, and GTK, handing focus back to it, gives
    // it to the first list instead (see `rebuild_normal`).
    let chosen: std::rc::Rc<std::cell::RefCell<Option<CtxAction>>> = Default::default();
    let entry_of = |(label, action): (&str, CtxAction)| {
        let chosen = chosen.clone();
        // Deleting a folder takes its mail with it: the one entry in
        // the destructive colour (docs/DESIGN.md, "Menus").
        let destructive = matches!(action, CtxAction::DeleteFolder { .. });
        let entry = MenuEntry::new(i18n(label), move || {
            *chosen.borrow_mut() = Some(action.clone());
        });
        if destructive {
            entry.destructive()
        } else {
            entry
        }
    };
    let sections = sections
        .into_iter()
        .map(|items| items.into_iter().map(entry_of).collect())
        .collect();
    let popover = crate::ui::context_menu::show_context_menu_popover(parent, x, y, None, sections);
    let s = sender.clone();
    popover.connect_closed(move |_| {
        if let Some(action) = chosen.borrow_mut().take() {
            let s = s.clone();
            gtk::glib::idle_add_local_once(move || {
                let _ = s.output(SidebarOutput::Context(action));
            });
        }
    });
}
