//! Favorites (#367): the folders picked from every account, merged into
//! one list, rebuilt only when what it holds changed. The only merged view
//! left once All Inboxes and the other combined views went (owner's
//! decision, 2026-11-02).

use std::collections::HashSet;

use relm4::prelude::*;

use crate::models::{FolderKind, Message};
use crate::ui::message_list::MessageListInput;

use super::model::AppModel;
use super::msg::AppMsg;

/// What a list holds, as one number: every message's place, state and
/// tags, whatever order they come in. A few milliseconds for tens of
/// thousands of messages, against a merge and a regrouping many times that.
fn content_fp<'a>(msgs: impl Iterator<Item = &'a Message>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut sum = 0u64;
    let mut n = 0u64;
    for m in msgs {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (m.account_id, m.folder_id, m.uid, m.id, m.timestamp).hash(&mut h);
        (m.unread, m.starred, m.answered, m.forwarded).hash(&mut h);
        (
            &m.keywords,
            &m.labels,
            m.subject.len(),
            m.preview.len(),
            m.from_addr.len(),
        )
            .hash(&mut h);
        (m.message_id.len(), m.references.len()).hash(&mut h);
        sum = sum.wrapping_add(h.finish());
        n += 1;
    }
    sum ^ n.rotate_left(32)
}

/// How many of a folder's newest messages a list not in memory shows at
/// once, read from the index while the rest of the folder is.
pub(super) const LIST_NEWEST: usize = 300;

impl AppModel {
    /// Merge Favorites' slices for the list shortly (and hand it the
    /// new thread links), once for every answer that lands meanwhile. Each
    /// merge regroups the whole view, which across accounts can be tens of
    /// thousands of messages.
    pub(super) fn queue_favorites_emit(&self, sender: &ComponentSender<Self>) {
        if self.favorites_emit_queued.replace(true) {
            return;
        }
        // Shortly after the first answer, so the rest of a run joins it, and
        // never sooner after the last merge than the view's size allows: an
        // account that keeps sending batches (a large folder catching up)
        // must not keep the window busy regrouping. A second for a small
        // view, up to five for one of tens of thousands of messages.
        let size = self.favorites_content().count();
        let gap = std::time::Duration::from_millis((size as u64 / 16).clamp(1000, 5000));
        let since = self.favorites_emitted_at.get().elapsed();
        let wait = std::time::Duration::from_millis(150).max(gap.saturating_sub(since));
        let s = sender.input_sender().clone();
        gtk::glib::timeout_add_local_once(wait, move || {
            let _ = s.send(AppMsg::EmitFavorites);
        });
    }

    /// The key the message list keeps Favorites under, apart from every
    /// folder's.
    pub(super) fn favorites_view_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        "favorites".hash(&mut h);
        h.finish()
    }

    /// What Favorites holds, as one number: every message's place,
    /// state and tags, whatever order the slices are in. A few milliseconds
    /// for tens of thousands of messages, against a merge and a regrouping
    /// many times that.
    pub(super) fn favorites_fp(&self) -> u64 {
        content_fp(self.favorites_content())
    }

    /// Every message of Favorites: its slices, and for a folder
    /// not copied yet, the folder's cache.
    fn favorites_content(&self) -> impl Iterator<Item = &Message> {
        let filled = self
            .favorites_slices
            .iter()
            .filter(|(k, _)| !self.favorites_unfilled.contains(k))
            .flat_map(|(_, v)| v.iter());
        let unfilled = self
            .favorites_unfilled
            .iter()
            .filter_map(|k| self.message_cache.get(k))
            .flat_map(|v| v.iter());
        filled.chain(unfilled)
    }

    /// The folders on show: the favorite ones, or the open folder.
    pub(super) fn folders_on_show(&self) -> HashSet<(u32, u32)> {
        if self.favorites_open {
            self.favorites_targets()
                .into_iter()
                .map(|(a, f, _)| (a, f))
                .collect()
        } else {
            self.selected
                .iter()
                .map(|s| (s.account_id, s.folder_id))
                .collect()
        }
    }

    /// Whether a folder of `len` messages is too large to keep in memory
    /// off screen: alone, or with the other favorite folders,
    /// more than [`crate::ui::message_list::KEEP_AT_MOST`]. Inboxes and the
    /// folders whose unread mail is counted are always kept: what arrives
    /// next is compared with them.
    pub(super) fn too_large_off_screen(
        &self,
        key: (u32, u32),
        len: usize,
        on_show: &HashSet<(u32, u32)>,
    ) -> bool {
        use crate::ui::message_list::KEEP_AT_MOST;
        let kind_of = |k: &(u32, u32)| self.folder_kind(k.0, k.1).unwrap_or(FolderKind::Custom);
        let kind = kind_of(&key);
        if kind == FolderKind::Inbox || on_show.contains(&key) {
            return false;
        }
        let row = match kind {
            FolderKind::Custom => len,
            _ => {
                len + self
                    .message_cache
                    .iter()
                    .filter(|(k, _)| **k != key && kind_of(k) == kind)
                    .map(|(_, l)| l.len())
                    .sum::<usize>()
            }
        };
        row > KEEP_AT_MOST && !self.counted_folders(key.0).iter().any(|c| c.id == key.1)
    }

    /// Let go of the folders [`Self::too_large_off_screen`] picks out, once
    /// their view is left. Read for one visit, an Archive of 81,000 messages
    /// stayed for the rest of the session, with the heap it was copied
    /// through (#504). Its next visit reads the index, as its first did.
    pub(super) fn shed_large_folders(&mut self) {
        let on_show = self.folders_on_show();
        let shed: Vec<(u32, u32)> = self
            .message_cache
            .iter()
            .filter(|(k, l)| self.too_large_off_screen(**k, l.len(), &on_show))
            .map(|(k, _)| *k)
            .collect();
        if shed.is_empty() {
            return;
        }
        let n: usize = shed
            .iter()
            .filter_map(|k| self.message_cache.remove(k))
            .map(|l| l.len())
            .sum();
        tracing::debug!("let go of {} folders off screen, {n} messages", shed.len());
        // Handed back once the list has let go of its rows too, a frame or
        // so on, rather than at the next half-minute trim: a system monitor
        // watched over the switch should see the memory go.
        gtk::glib::timeout_add_seconds_local_once(3, || {
            if let Some(held) = crate::desktop::memory_report::trim_if_worthwhile() {
                tracing::debug!(target: "tegami::memory", "trimmed the heap: {} was freed but held", crate::desktop::memory_report::human_bytes(held as u64));
            }
        });
    }

    /// Copy the folders `open_favorites` left to the folder caches into
    /// the Favorites slices.
    pub(super) fn fill_favorites_slices(&mut self) {
        let t = std::time::Instant::now();
        for key in std::mem::take(&mut self.favorites_unfilled) {
            match self.message_cache.get(&key) {
                Some(c) if !c.is_empty() => {
                    self.favorites_slices.insert(key, c.to_vec());
                }
                _ => {
                    self.favorites_slices.remove(&key);
                }
            }
        }
        tracing::debug!("favorites: slices filled in {:?}", t.elapsed());
    }

    /// Push every account's inbox slice to the list as one date-sorted run. The
    /// slices arrive independently (cache seed, then each account's load), so the
    /// whole merged list is re-emitted each time one of them changes.
    pub(super) fn emit_favorites(&self) {
        let t = std::time::Instant::now();
        let fp = self.favorites_fp();
        if self.list_fp.get() == Some(fp)
            && self.message_list.model().view_key() == Some(self.favorites_view_key())
        {
            tracing::debug!("favorites: unchanged, not merged ({:?})", t.elapsed());
            return;
        }
        self.list_fp.set(Some(fp));
        self.message_list.emit(MessageListInput::SetViewFp(fp));
        self.favorites_emitted_at.set(t);
        let mut merged: Vec<Message> = self.favorites_content().cloned().collect();
        merged.sort_by_key(|m| std::cmp::Reverse(m.timestamp));
        tracing::debug!(
            "favorites: merged {} messages in {:?}",
            merged.len(),
            t.elapsed()
        );
        self.message_list
            .emit(MessageListInput::SetMessages { messages: merged });
    }

    /// The folders Favorites merges: (account, folder id, path).
    pub(super) fn favorites_targets(&self) -> Vec<(u32, u32, String)> {
        self.favorite_folder_refs()
            .into_iter()
            .map(|r| (r.account_id, r.folder.id, r.folder.path))
            .collect()
    }

    /// Whether a folder is one Favorites merges.
    pub(super) fn is_favorites_target(&self, account_id: u32, folder_id: u32) -> bool {
        self.favorite_folder_refs()
            .iter()
            .any(|r| r.account_id == account_id && r.folder.id == folder_id)
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_favorites_view(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::FillFavoritesSlices => self.fill_favorites_slices(),

            AppMsg::FavoritesFoldersLoaded { lists } => self.on_favorites_folders_loaded(lists),

            AppMsg::EmitFavoritesNow => {
                self.list_fp.set(None);
                if self.favorites_open {
                    self.emit_favorites();
                }
            }

            AppMsg::EmitFavorites => {
                self.favorites_emit_queued.set(false);
                if self.thread_links_due.replace(false) {
                    self.push_thread_links();
                }
                if self.favorites_open && self.favorites_dirty.replace(false) {
                    self.emit_favorites();
                }
            }
            _ => unreachable!("not routed to on_favorites_view"),
        }
    }

    /// Folders not in memory yet, read whole from the index off the main
    /// thread: for Favorites being opened.
    fn on_favorites_folders_loaded(&mut self, lists: Vec<((u32, u32), Vec<Message>)>) {
        let open = self.favorites_open;
        let mut landed = false;
        for ((account_id, folder_id), messages) in lists {
            // The worker's own answer may have come first. An empty
            // folder is recorded as read too: building Favorites
            // ahead waits on all of its folders.
            if self
                .message_cache
                .get(&(account_id, folder_id))
                .is_some_and(|c| !c.is_empty())
            {
                continue;
            }
            if messages.is_empty() {
                self.message_cache
                    .entry((account_id, folder_id))
                    .or_default();
                continue;
            }
            let messages = self.overlay_local_state(account_id, folder_id, messages);
            if open && self.is_favorites_target(account_id, folder_id) {
                self.favorites_unfilled.remove(&(account_id, folder_id));
                self.favorites_slices
                    .insert((account_id, folder_id), messages.clone());
                landed = true;
            } else if !open
                && self
                    .selected
                    .as_ref()
                    .is_some_and(|s| (s.account_id, s.folder_id) == (account_id, folder_id))
            {
                // The folder on show had only its newest so far.
                self.message_list.emit(MessageListInput::SetMessages {
                    messages: messages.clone(),
                });
            }
            self.message_cache
                .insert((account_id, folder_id), messages.into());
        }
        // A view left before its folders landed.
        self.shed_large_folders();
        if landed {
            self.emit_favorites();
            self.push_index_complete();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::msg;

    /// The fingerprint ignores order, and (B29) only hashes the lengths of
    /// the text fields: an equal-length edit leaves it unchanged.
    #[test]
    fn content_fingerprint_ignores_order_and_same_length_text() {
        let a = msg(1, 1, 1);
        let mut b = msg(1, 1, 2);
        b.subject = "abc".into();
        let fp = content_fp([&a, &b].into_iter());
        assert_eq!(fp, content_fp([&b, &a].into_iter()));
        let mut b2 = b.clone();
        b2.subject = "xyz".into();
        assert_eq!(fp, content_fp([&a, &b2].into_iter()));
        let mut b3 = b.clone();
        b3.unread = true;
        assert_ne!(fp, content_fp([&a, &b3].into_iter()));
        assert_ne!(content_fp([&a].into_iter()), content_fp(std::iter::empty()));
    }
}
