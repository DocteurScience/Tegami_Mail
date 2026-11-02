//! Which sections, rows and tree nodes are open, and the unread counts the
//! rows show; the rail's and Focus Mode's open state is temporary.

use super::tree::hidden_by_collapse;
use super::types::{chevron_icon, UnifiedRow};
use super::widgets::slide_tree_row;
use super::*;

impl Sidebar {
    /// Flip one tree node, restyle its caret, and re-apply visibility across
    /// the account's tree — no rebuild, so nothing flickers. Reports the new
    /// state for persistence.
    pub(super) fn toggle_folder_node(
        &mut self,
        account_id: u32,
        path: String,
        sender: &ComponentSender<Self>,
    ) {
        let nodes = self.tree_collapsed.entry(account_id).or_default();
        let collapsed = if nodes.contains(&path) {
            nodes.remove(&path);
            false
        } else {
            nodes.insert(path.clone());
            true
        };
        if let Some(img) = self.tree_chevrons.get(&(account_id, path.clone())) {
            if collapsed {
                img.remove_css_class("open");
            } else {
                img.add_css_class("open");
            }
        }
        self.apply_tree_visibility(account_id);
        let _ = sender.output(SidebarOutput::FolderNodeCollapsed {
            account_id,
            path,
            collapsed,
        });
    }

    /// Re-apply row visibility across one account's custom-folder tree (#51):
    /// a row shows unless some ancestor node is collapsed. Rows are never
    /// removed, so selection indices hold still.
    fn apply_tree_visibility(&self, account_id: u32) {
        // The folders inside the Inbox, under its row (#345).
        if let (Some(list), Some(folders), Some(revealers)) = (
            self.folder_lists.get(&account_id),
            self.main_folders.get(&account_id),
            self.main_row_revealers.get(&account_id),
        ) {
            let collapsed = self
                .tree_collapsed
                .get(&account_id)
                .cloned()
                .unwrap_or_default();
            for (&i, rev) in revealers {
                let (Some(row), Some(folder)) = (list.row_at_index(i as i32), folders.get(i))
                else {
                    continue;
                };
                slide_tree_row(&row, rev, hidden_by_collapse(&folder.path, &collapsed));
            }
        }
        let (Some(list), Some(folders)) = (
            self.custom_folder_lists.get(&account_id),
            self.custom_folders.get(&account_id),
        ) else {
            return;
        };
        let collapsed = self
            .tree_collapsed
            .get(&account_id)
            .cloned()
            .unwrap_or_default();
        let revealers = self.tree_row_revealers.get(&account_id);
        for (i, folder) in folders.iter().enumerate() {
            let Some(row) = list.row_at_index(i as i32) else {
                continue;
            };
            let hidden = hidden_by_collapse(&folder.path, &collapsed);
            match revealers.and_then(|r| r.get(i)) {
                Some(rev) => slide_tree_row(&row, rev, hidden),
                None => row.set_visible(!hidden),
            }
        }
    }

    /// Whether a row's list is open (persisted, folded by the rail).
    fn row_open(&self, row: UnifiedRow) -> bool {
        match row {
            UnifiedRow::Favorites => self.favorites_expanded,
            UnifiedRow::Tags => self.tags_expanded,
            UnifiedRow::Smart => self.smart_expanded,
        }
    }

    /// Whether "Fold up expanded items" starts this row folded in the rail:
    /// the sidebar is the icon rail and the row's switch is on. Such a row
    /// keeps its own rail-only open state (`rail_open`).
    fn locked_row(&self, row: UnifiedRow) -> bool {
        // Focus Mode folds every unified row the same way.
        self.focus_fold_unified
            || self.collapsed
                && match row {
                    UnifiedRow::Favorites | UnifiedRow::Smart => false,
                    UnifiedRow::Tags => self.rail_fold.folds_tags(),
                }
    }

    /// Whether the unified rows' open states are the temporary kind: the
    /// icon rail's, or Focus Mode's. Either keeps the saved state untouched.
    fn rows_temporary(&self) -> bool {
        self.collapsed || self.focus_fold_unified
    }

    /// Whether "Fold up expanded items" holds the accounts folded.
    fn locked_accounts(&self) -> bool {
        self.collapsed && self.rail_fold.folds_accounts()
    }

    /// Whether a unified row's list shows open. In the rail: what it was
    /// last set to there, else folded when "Fold up expanded items" covers
    /// it, else its saved state. In the full sidebar: its saved state.
    pub(super) fn row_shown_open(&self, row: UnifiedRow) -> bool {
        if !self.rows_temporary() {
            return self.row_open(row);
        }
        match self.rail_open.get(&row) {
            Some(open) => *open,
            None => !self.locked_row(row) && self.row_open(row),
        }
    }

    /// Whether an account's section shows folded: the same rule, with the
    /// rail-only state in `rail_open_accounts`.
    pub(super) fn account_shown_folded(&self, section: &SectionData) -> bool {
        if !self.collapsed {
            return section.collapsed;
        }
        match self.rail_open_accounts.get(&section.account.id) {
            Some(open) => !*open,
            None => self.locked_accounts() || section.collapsed,
        }
    }

    fn set_row_open(&mut self, row: UnifiedRow, open: bool) {
        match row {
            UnifiedRow::Favorites => self.favorites_expanded = open,
            UnifiedRow::Tags => self.tags_expanded = open,
            UnifiedRow::Smart => self.smart_expanded = open,
        }
    }

    /// Focus Mode is folding the unified rows: shut every open one on
    /// screen, animated, with its chevron turned. The rebuild that follows
    /// draws them folded for good (see `locked_row`).
    pub(super) fn fold_unified_rows_now(&self) {
        for w in self.kind_widgets.values() {
            w.revealer.set_reveal_child(false);
            if let Some(ch) = &w.chevron {
                ch.set_icon_name(Some(chevron_icon(false)));
            }
        }
    }

    /// The unread total a unified row's folded chip shows.
    pub(super) fn row_unread(&self, row: UnifiedRow) -> u32 {
        match row {
            UnifiedRow::Favorites => self
                .favorite_folders
                .iter()
                .filter(|r| r.folder.kind != FolderKind::Sent)
                .map(|r| r.folder.unread)
                .sum(),
            UnifiedRow::Tags => 0,
            UnifiedRow::Smart => self.smart_unread.values().sum(),
        }
    }

    /// Open or fold a unified row's list in place (a click on its chevron,
    /// a long press in the rail), and report the section states.
    pub(super) fn toggle_row(&mut self, row: UnifiedRow, sender: &ComponentSender<Self>) {
        let open = !self.row_shown_open(row);
        if self.rows_temporary() {
            // Opened or folded for the rail alone: the saved state is
            // untouched, and the full sidebar comes back as it was left.
            self.rail_open.insert(row, open);
        } else {
            self.set_row_open(row, open);
            self.report_sections(sender);
        }
        let unread = self.row_unread(row);
        if let Some(w) = self.kind_widgets.get(&row) {
            w.revealer.set_reveal_child(open);
            // Expanded: the list shows each count, so the total chip bows
            // out; it returns when folded back up.
            if let Some(b) = &w.badge {
                b.set_visible(unread > 0 && !open && self.chip_shown(row));
            }
            if let Some(ch) = &w.chevron {
                ch.set_icon_name(Some(chevron_icon(open)));
            }
        }
    }

    /// Whether `row` is drawn in the tree style: Favorites and Tags
    /// with it chosen, out of the icon rail.
    pub(super) fn as_heading(&self, row: UnifiedRow) -> bool {
        self.section_style == crate::config::SectionStyle::Tree
            && !self.collapsed
            && matches!(row, UnifiedRow::Favorites | UnifiedRow::Tags)
    }

    /// Whether a row's total-unread chip is shown. Tags never carry one.
    pub(super) fn chip_shown(&self, row: UnifiedRow) -> bool {
        match row {
            UnifiedRow::Favorites | UnifiedRow::Smart => true,
            UnifiedRow::Tags => false,
        }
    }

    /// Whether the unified Tags section has anything to show.
    pub(super) fn unified_tags_shown(&self) -> bool {
        !self.tags.is_empty() && self.tags_placement.shown()
    }

    /// The Favorites and Tags rows placed `at`, above or
    /// below the accounts.
    pub(super) fn placed_rows(&self, at: crate::config::SectionPlacement) -> Vec<UnifiedRow> {
        let mut rows = Vec::new();
        if self.favorites_shown() && self.favorites_placement == at {
            rows.push(UnifiedRow::Favorites);
        }
        if self.unified_tags_shown() && self.tags_placement == at {
            rows.push(UnifiedRow::Tags);
        }
        rows
    }

    /// Whether Favorites are in the sidebar at all.
    pub(super) fn favorites_shown(&self) -> bool {
        !self.favorite_folders.is_empty() && self.favorites_placement.shown()
    }

    pub(super) fn tags_open(&self, slot: Slot) -> bool {
        match slot {
            Slot::Unified => self.tags_expanded,
            Slot::Account(id) => self
                .sections
                .iter()
                .find(|s| s.account.id == id)
                .is_some_and(|s| s.tags_expanded),
        }
    }

    fn set_tags_open(&mut self, slot: Slot, open: bool, sender: &ComponentSender<Self>) {
        match slot {
            Slot::Unified => {
                self.tags_expanded = open;
                self.report_sections(sender);
            }
            Slot::Account(id) => {
                if let Some(s) = self.sections.iter_mut().find(|s| s.account.id == id) {
                    s.tags_expanded = open;
                }
                let _ = sender.output(SidebarOutput::ToggleAccountTags(id));
            }
        }
    }

    /// The paths of an account's folders that are favorites.
    pub(super) fn favorite_paths(&self, account_id: u32) -> Vec<String> {
        self.favorite_folders
            .iter()
            .filter(|r| r.account_id == account_id)
            .map(|r| r.folder.path.clone())
            .collect()
    }

    /// Tell the app how the sections stand, for the saved layout.
    fn report_sections(&self, sender: &ComponentSender<Self>) {
        let _ = sender.output(SidebarOutput::SectionsOpen {
            tags: self.tags_expanded,
            favorites: self.favorites_expanded,
            smart: self.smart_expanded,
        });
    }
}

impl Sidebar {
    // The unified slot is a unified row; an account's is a heading.
    pub(super) fn on_toggle_tags_expand(&mut self, slot: Slot, sender: ComponentSender<Self>) {
        if slot == Slot::Unified {
            self.toggle_row(UnifiedRow::Tags, &sender);
        } else {
            let open = !self.tags_open(slot);
            self.set_tags_open(slot, open, &sender);
            if let Some(w) = self.tag_sections.get(&slot) {
                w.revealer.set_reveal_child(open);
                w.chevron.set_icon_name(Some(chevron_icon(open)));
            }
        }
    }

    pub(super) fn on_toggle_smart_expand(&mut self, sender: ComponentSender<Self>) {
        self.toggle_row(UnifiedRow::Smart, &sender);
        // The header is a toggle, not a view: it never stays picked.
        if let Some(w) = self.kind_widgets.get(&UnifiedRow::Smart) {
            let header = w.header.clone();
            let quiet = self.quiet.clone();
            gtk::glib::idle_add_local_once(move || {
                quiet.set(true);
                header.unselect_all();
                quiet.set(false);
            });
        }
    }

    pub(super) fn on_toggle_collapse_local(&mut self, id: u32, sender: ComponentSender<Self>) {
        // In the rail this opens or folds the account for the rail
        // alone — the saved state stays as it is for the full
        // sidebar, which comes back as it was left.
        let rail_only = self.collapsed;
        if let Some(rev) = self.revealers.get(&id) {
            let expanded = !rev.reveals_child();
            rev.set_reveal_child(expanded);
            if let Some(ch) = self.chevrons.get(&id) {
                ch.set_icon_name(Some(chevron_icon(expanded)));
            }
            if rail_only {
                self.rail_open_accounts.insert(id, expanded);
            } else if let Some(s) = self.sections.iter_mut().find(|s| s.account.id == id) {
                s.collapsed = !expanded;
            }
            // The Inbox chip lives inside the folder list we just hid/shown,
            // so mirror it onto the avatar: visible only while collapsed.
            if let Some(label) = self.account_circle_badges.get(&id) {
                let n = self
                    .sections
                    .iter()
                    .find(|s| s.account.id == id)
                    .and_then(|s| s.folders.iter().find(|f| f.kind == FolderKind::Inbox))
                    .map(|f| f.unread)
                    .unwrap_or(0);
                label.set_text(&n.to_string());
                label.set_visible(!expanded && n > 0);
            }
            if !rail_only {
                let _ = sender.output(SidebarOutput::AccountFolded {
                    id,
                    folded: !expanded,
                });
            }
        }
    }

    pub(super) fn on_toggle_custom_folders_local(
        &mut self,
        id: u32,
        sender: ComponentSender<Self>,
    ) {
        if let Some(rev) = self.custom_revealers.get(&id) {
            let expanded = !rev.reveals_child();
            rev.set_reveal_child(expanded);
            if let Some(ch) = self.custom_chevrons.get(&id) {
                ch.set_icon_name(Some(chevron_icon(expanded)));
            }
            if let Some(s) = self.sections.iter_mut().find(|s| s.account.id == id) {
                s.custom_expanded = expanded;
            }
            let _ = sender.output(SidebarOutput::ToggleCustomFolders(id));
        }
    }

    pub(super) fn on_expand_for_drop(&mut self, id: u32, sender: ComponentSender<Self>) {
        // Expand a collapsed account so its folders become drop targets.
        self.reveal_account(id, &sender);
    }
}
