//! What the app hands the sidebar: its contents (a rebuild), unread counts
//! and the small states of the outbox, attachments, busy and offline rows.

use super::*;

impl Sidebar {
    /// `msg` is the `SetContents` the router hands over whole: its nineteen
    /// fields are the sidebar's contents and configuration.
    pub(super) fn on_set_contents(
        &mut self,
        msg: SidebarInput,
        widgets: &SidebarWidgets,
        sender: ComponentSender<Self>,
    ) {
        let SidebarInput::SetContents {
            mut sections,
            chevrons_left,
            account_click_folds,
            rail_dots,
            rail_fold,
            favorite_folders,
            tags,
            section_style,
            tags_placement,
            favorites_placement,
            smart_folders,
        } = msg
        else {
            // Only SetContents is routed here.
            return;
        };
        // Order each account's folders essential-first, then custom, so
        // the essential/custom split lines up with row indices (the main
        // list holds indices 0..E, the custom list E..), each part in
        // the user's own order where they set one.
        for s in &mut sections {
            s.folders = order_folders(
                std::mem::take(&mut s.folders),
                &s.folder_order,
                s.folder_sort,
            );
        }
        // Seed the tree's collapsed nodes from the persisted state;
        // later chevron clicks flip the local copy.
        self.tree_collapsed = sections
            .iter()
            .map(|s| (s.account.id, s.tree_collapsed.iter().cloned().collect()))
            .collect();
        self.sections = sections;
        self.chevrons_left = chevrons_left;
        self.account_click_folds = account_click_folds;
        self.rail_dots = rail_dots;
        // "Fold up expanded items" is a view of the rail, not a
        // change to what is saved: the rebuild below draws the
        // ticked items folded while the sidebar is collapsed and
        // as they are once it expands (see `locked_row`).
        self.rail_fold = rail_fold;
        self.favorite_folders = favorite_folders;
        self.tags = tags;
        self.section_style = section_style;
        self.tags_placement = tags_placement;
        self.favorites_placement = favorites_placement;
        self.smart_folders = smart_folders;
        self.settle_selection();
        self.rebuild(widgets, &sender);
    }

    /// The selection after new contents: a row that is gone falls back to
    /// the default view, and a folder that left a section stays the open
    /// folder, highlighted on its account's own row.
    fn settle_selection(&mut self) {
        // The open smart folder was deleted: fall back to the
        // default view.
        if let Sel::Smart(id) = &self.selected {
            if !self.smart_folders.iter().any(|(i, _)| i == id) {
                self.selected = Sel::None;
            }
        }
        // The open tag was removed (or its account): fall back to
        // the default view.
        if let Sel::Tag(acc, kw) = &self.selected {
            let gone = !self.tags.iter().any(|t| t.keyword.eq_ignore_ascii_case(kw))
                || acc.is_some_and(|a| !self.sections.iter().any(|s| s.account.id == a));
            if gone {
                self.selected = Sel::None;
            }
        }
        if self.selected == Sel::UnifiedFavorites && !self.favorites_shown() {
            self.selected = Sel::None;
        }
        if self.selected == Sel::UnifiedTags && !self.unified_tags_shown() {
            self.selected = Sel::None;
        }
        // A folder taken out of Favorites is still the open folder.
        if let Sel::Favorite(acc, path) = &self.selected {
            if !self
                .favorite_folders
                .iter()
                .any(|r| r.account_id == *acc && r.folder.path == *path)
            {
                self.selected = Sel::Folder(*acc, path.clone());
            }
        }
    }

    pub(super) fn on_set_outbox_count(
        &mut self,
        count: u32,
        widgets: &SidebarWidgets,
        sender: ComponentSender<Self>,
    ) {
        if self.outbox_count == count {
            return;
        }
        let appearing = (self.outbox_count == 0) != (count == 0);
        self.outbox_count = count;
        // The row itself comes and goes with the count, so the sidebar
        // only needs rebuilding when it crosses zero; otherwise just
        // refresh the badge in place.
        if appearing {
            // Leaving the Outbox selected when its row disappears would
            // strand the view on an empty list.
            if count == 0 && self.selected == Sel::Outbox {
                self.selected = Sel::None;
            }
            self.rebuild(widgets, &sender);
        } else if let Some(list) = &self.outbox_list {
            if let Some(row) = list.row_at_index(0) {
                if let Some(badge) = row.child().and_downcast::<gtk::Box>() {
                    if let Some(label) = badge.last_child().and_downcast::<gtk::Label>() {
                        label.set_label(&count.to_string());
                    }
                }
            }
        }
    }

    pub(super) fn on_set_smart_unread(&mut self, counts: HashMap<String, u32>) {
        for (id, badge) in &self.smart_badges {
            let n = counts.get(id).copied().unwrap_or(0);
            badge.set_label(&n.to_string());
            badge.set_visible(n > 0);
        }
        self.smart_unread = counts;
        if let Some(w) = self.kind_widgets.get(&UnifiedRow::Smart) {
            if let Some(b) = &w.badge {
                let total = self.row_unread(UnifiedRow::Smart);
                b.set_label(&total.to_string());
                b.set_visible(total > 0 && !self.row_shown_open(UnifiedRow::Smart));
            }
        }
    }

    /// The tags' unread counts arrived: their badges change in place, and
    /// the Tags row's folded total with them.
    pub(super) fn on_set_tag_unread(&mut self, counts: HashMap<String, u32>) {
        for (keyword, badge) in &self.tag_badges {
            let n = counts.get(keyword).copied().unwrap_or(0);
            badge.set_label(&n.to_string());
            badge.set_visible(n > 0);
        }
        self.tag_unread = counts;
        if let Some(w) = self.kind_widgets.get(&UnifiedRow::Tags) {
            if let Some(b) = &w.badge {
                let total = self.row_unread(UnifiedRow::Tags);
                b.set_label(&total.to_string());
                b.set_visible(total > 0 && !self.row_shown_open(UnifiedRow::Tags));
            }
        }
    }

    pub(super) fn on_set_unread(
        &mut self,
        folders: HashMap<(u32, u32), u32>,
        fresh: Option<std::collections::HashSet<(u32, u32)>>,
    ) {
        self.fresh = fresh;
        // Mirror the fresh counts into every row list too, so a
        // rebuild draws them right (the account folders are done
        // below).
        let fresh = |aid: u32, fid: u32, cur: u32| folders.get(&(aid, fid)).copied().unwrap_or(cur);
        for r in self.favorite_folders.iter_mut() {
            r.folder.unread = fresh(r.account_id, r.folder.id, r.folder.unread);
        }
        for s in &mut self.sections {
            let aid = s.account.id;
            for f in &mut s.filtered {
                f.unread = fresh(aid, f.id, f.unread);
            }
        }
        for ((aid, fid), label) in self
            .folder_badges
            .iter()
            .chain(self.kind_widgets.values().flat_map(|w| &w.row_badges))
        {
            let n = folders.get(&(*aid, *fid)).copied().unwrap_or(0);
            label.set_text(&n.to_string());
            label.set_visible(n > 0);
        }
        for (row, w) in &self.kind_widgets {
            let total: u32 = w
                .row_badges
                .keys()
                .map(|k| folders.get(k).copied().unwrap_or(0))
                .sum();
            if let Some(b) = &w.badge {
                b.set_text(&total.to_string());
                b.set_visible(total > 0 && !self.row_shown_open(*row) && self.chip_shown(*row));
            }
        }
        // Keep the avatar-circle badges in sync too. They only show while
        // the account is collapsed (toggled live in ToggleCollapseLocal),
        // so we just refresh the number and re-apply that visibility rule.
        for section in &self.sections {
            if let Some(label) = self.account_circle_badges.get(&section.account.id) {
                let n = section
                    .folders
                    .iter()
                    .find(|f| f.kind == FolderKind::Inbox)
                    .and_then(|f| folders.get(&(section.account.id, f.id)))
                    .copied()
                    .unwrap_or(0);
                label.set_text(&n.to_string());
                label.set_visible(self.account_shown_folded(section) && n > 0);
            }
        }
        self.restyle_seen();
        // Persist the fresh counts into `sections` as well. Otherwise the
        // next rebuild_normal (e.g. toggling the sidebar collapse) recreates
        // every badge from the folder unread values captured at the last
        // SetContents, reverting in-place updates — so a read inbox's chip
        // reappears on collapse. Keep `sections` a faithful mirror.
        for section in &mut self.sections {
            for folder in &mut section.folders {
                folder.unread = folders
                    .get(&(section.account.id, folder.id))
                    .copied()
                    .unwrap_or(0);
            }
        }
    }

    pub(super) fn on_set_busy(&mut self, busy: bool) {
        self.busy = busy;
        if let Some(sp) = &self.sync_spinner {
            sp.set_spinning(busy);
        }
        if let Some(stack) = &self.sync_stack {
            stack.set_visible_child_name(if busy { "spinner" } else { "icon" });
        }
    }

    pub(super) fn on_set_offline(&mut self, offline: bool) {
        self.offline = offline;
        if let Some(button) = &self.sync_button {
            crate::ui::show_refresh_offline(button, offline);
        }
    }
}
