//! What the dialog's rows and the app ask of it: the kept settings'
//! changes, and leaving an open editor.

use super::choices::{files_large_at, read_mark_at};
use super::*;

impl Preferences {
    pub(super) fn on_change_fetch_interval(&mut self, index: u32, sender: ComponentSender<Self>) {
        let secs = FETCH_INTERVALS
            .get(index as usize)
            .map(|(_, s)| *s)
            .unwrap_or(0);
        let _ = sender.output(PrefOutput::SetFetchInterval(secs));
    }

    pub(super) fn on_change_app_theme(&mut self, index: u32, sender: ComponentSender<Self>) {
        let theme = APP_THEMES
            .get(index as usize)
            .map(|(_, t)| *t)
            .unwrap_or_default();
        let _ = sender.output(PrefOutput::SetAppTheme(theme));
    }

    pub(super) fn on_change_read_mark(&mut self, idx: u32, sender: ComponentSender<Self>) {
        let _ = sender.output(PrefOutput::SetReadMark(read_mark_at(idx)));
    }

    pub(super) fn on_change_keep_downloads(&mut self, idx: u32, sender: ComponentSender<Self>) {
        let days = KEEP_DOWNLOADS_DAYS.get(idx as usize).copied().unwrap_or(0);
        let _ = sender.output(PrefOutput::SetKeepDownloads(days));
    }

    pub(super) fn on_change_files_large(&mut self, idx: u32, sender: ComponentSender<Self>) {
        let large = files_large_at(idx);
        if self.files.large != large {
            self.files.large = large;
            let _ = sender.output(PrefOutput::SetFilesPrefs(self.files));
        }
    }

    /// The app changed the policy (a hand-off dialog's "always do this"):
    /// move the row, only when it differs, as a row set to its own value
    /// is a no-op but one set on every update hung GTK in the unmapped
    /// dialog.
    pub(super) fn on_set_files_prefs(&mut self, prefs: crate::config::FilesPrefs) {
        self.files = prefs;
        if let Some(row) = &self.files_large_row {
            let at = super::choices::files_large_index(prefs.large);
            if row.selected() != at {
                row.set_selected(at);
            }
        }
    }

    /// Accounts, filters, tags and senders come from a fresh panel at each
    /// open: it goes on the Accounts sub-page.
    pub(super) fn on_set_accounts_panel(
        &mut self,
        panel: gtk::Widget,
        accounts: relm4::Sender<crate::ui::accounts::AccountsInput>,
    ) {
        self.accounts_sender = accounts;
        if let Some(slot) = &self.accounts_slot {
            slot.set_child(Some(&panel));
        }
    }

    pub(super) fn on_editor_open(&mut self, page: Option<&'static str>) {
        self.editor_open = page.is_some();
        self.editor_page = page.unwrap_or("accounts");
        if let Some(header) = &self.host_header {
            header.set_visible(page.is_none());
        }
    }

    pub(super) fn on_cloud_editor_open(&mut self, open: bool) {
        self.editor_open = open;
        self.editor_page = "cloud";
        self.show_sub_header("cloud", !open);
    }
}
