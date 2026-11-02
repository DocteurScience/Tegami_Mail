//! The user picking a row: each list's selection handler, keeping one
//! selection across the sidebar and telling the app.

use super::*;

impl Sidebar {
    pub(super) fn on_outbox_row_selected(&mut self, sender: ComponentSender<Self>) {
        if self.selected == Sel::Outbox {
            return;
        }
        self.selected = Sel::Outbox;
        self.clear_other_selections(Sel::Outbox);
        let _ = sender.output(SidebarOutput::OutboxSelected);
    }

    pub(super) fn on_tag_row_selected(
        &mut self,
        slot: Slot,
        index: i32,
        sender: ComponentSender<Self>,
    ) {
        if let Some(t) = self.tags.get(index as usize).cloned() {
            let account = slot.account();
            let key = Sel::Tag(account, t.keyword.clone());
            if self.selected == key {
                return;
            }
            self.selected = key.clone();
            self.clear_other_selections(key);
            let _ = sender.output(SidebarOutput::TagSelected {
                keyword: t.keyword,
                account,
            });
        }
    }

    pub(super) fn on_smart_row_selected(&mut self, index: i32, sender: ComponentSender<Self>) {
        let Some((id, _)) = self.smart_folders.get(index as usize).cloned() else {
            return;
        };
        let key = Sel::Smart(id.clone());
        if self.selected == key {
            return;
        }
        self.selected = key.clone();
        self.clear_other_selections(key);
        let _ = sender.output(SidebarOutput::SmartSelected(id));
    }

    pub(super) fn on_favorite_row_selected(&mut self, index: i32, sender: ComponentSender<Self>) {
        let Some(r) = self.favorite_folders.get(index as usize).cloned() else {
            return;
        };
        let key = Sel::Favorite(r.account_id, r.folder.path.clone());
        if self.selected == key {
            return;
        }
        self.selected = key.clone();
        self.clear_other_selections(key);
        let _ = sender.output(SidebarOutput::FolderSelected {
            account_id: r.account_id,
            folder_id: r.folder.id,
            name: r.folder.name,
            path: r.folder.path,
        });
    }

    pub(super) fn on_unified_favorites_selected(&mut self, sender: ComponentSender<Self>) {
        if self.selected == Sel::UnifiedFavorites {
            return;
        }
        self.selected = Sel::UnifiedFavorites;
        self.clear_other_selections(Sel::UnifiedFavorites);
        let _ = sender.output(SidebarOutput::UnifiedFavoritesSelected);
    }

    pub(super) fn on_unified_tags_selected(&mut self, sender: ComponentSender<Self>) {
        if self.selected == Sel::UnifiedTags {
            return;
        }
        self.selected = Sel::UnifiedTags;
        self.clear_other_selections(Sel::UnifiedTags);
        let _ = sender.output(SidebarOutput::UnifiedTagsSelected);
    }

    pub(super) fn on_folder_row_selected(
        &mut self,
        account_id: u32,
        index: i32,
        sender: ComponentSender<Self>,
    ) {
        let folder = self
            .sections
            .iter()
            .find(|s| s.account.id == account_id)
            .and_then(|s| s.folders.get(index as usize))
            .cloned();
        if let Some(folder) = folder {
            let key = Sel::Folder(account_id, folder.path.clone());
            if self.selected == key {
                return;
            }
            self.selected = key.clone();
            self.clear_other_selections(key);
            let _ = sender.output(SidebarOutput::FolderSelected {
                account_id,
                folder_id: folder.id,
                name: folder.name.clone(),
                path: folder.path.clone(),
            });
        }
    }
}
