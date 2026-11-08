//! The rows kept from the former General, Reading, Conversations, Message
//! list, Appearance and System pages (docs/DESIGN.md, "Preferences"):
//! `dialog` sets them in the Messages, Notifications and Application pages.

use adw::prelude::*;

use super::super::choices::{files_large_index, read_mark_index};
use super::super::rows::no_truncate;
use super::super::{
    PrefInit, PrefInput, PrefOutput, ViewSender, APP_THEMES, FETCH_INTERVALS, KEEP_DOWNLOADS_DAYS,
};
use crate::i18n::i18n;

/// The rows, each wired to the setting it changes.
#[derive(Debug)]
pub(in crate::ui::preferences) struct KeptRows {
    // Messages → Reading.
    pub(in crate::ui::preferences) read_mark_row: adw::ComboRow,
    pub(in crate::ui::preferences) thread_newest_first_row: adw::SwitchRow,
    pub(in crate::ui::preferences) confirm_thread_delete_row: adw::SwitchRow,
    pub(in crate::ui::preferences) swipe_enabled_row: adw::SwitchRow,
    // Notifications → New messages.
    pub(in crate::ui::preferences) notifications_row: adw::SwitchRow,
    pub(in crate::ui::preferences) notification_content_row: adw::SwitchRow,
    pub(in crate::ui::preferences) notify_receipts_row: adw::SwitchRow,
    pub(in crate::ui::preferences) fetch_row: adw::ComboRow,
    pub(in crate::ui::preferences) push_row: adw::SwitchRow,
    // Application.
    pub(in crate::ui::preferences) app_theme_row: adw::ComboRow,
    pub(in crate::ui::preferences) background_row: adw::SwitchRow,
    pub(in crate::ui::preferences) autostart_row: adw::SwitchRow,
    pub(in crate::ui::preferences) single_key_row: adw::SwitchRow,
    pub(in crate::ui::preferences) gmail_keys_row: adw::SwitchRow,
    pub(in crate::ui::preferences) keep_downloads_row: adw::ComboRow,
    pub(in crate::ui::preferences) files_large_row: adw::ComboRow,
}

fn switch(title: &str, subtitle: &str) -> adw::SwitchRow {
    let row = adw::SwitchRow::new();
    row.set_title(title);
    if !subtitle.is_empty() {
        row.set_subtitle(subtitle);
    }
    row
}

fn combo(title: &str, subtitle: &str, labels: &[String]) -> adw::ComboRow {
    let row = adw::ComboRow::new();
    row.set_title(title);
    if !subtitle.is_empty() {
        row.set_subtitle(subtitle);
    }
    let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
    row.set_model(Some(&gtk::StringList::new(&refs)));
    no_truncate(&row);
    row
}

impl KeptRows {
    /// The rows, holding `init`'s values. They are wired after the values
    /// are in; `ViewSender` holds anything back until the window is ready
    /// in any case.
    pub(in crate::ui::preferences) fn build(init: &PrefInit, sender: &ViewSender) -> Self {
        let read_mark_row = combo(
            &i18n("Mark as read"),
            &i18n("When a message is opened."),
            &[
                i18n("When displayed"),
                i18n("After two seconds"),
                i18n("Manually"),
            ],
        );
        read_mark_row.set_selected(read_mark_index(init.read_mark));
        let thread_newest_first_row =
            switch(&i18n("Newest message first"), &i18n("In a conversation."));
        thread_newest_first_row.set_active(init.thread_newest_first);
        let confirm_thread_delete_row = switch(&i18n("Confirm deleting a conversation"), "");
        confirm_thread_delete_row.set_active(init.confirm_thread_delete);
        let swipe_enabled_row = switch(
            &i18n("Swipe to archive or delete"),
            &i18n("For touch screens and touchpads."),
        );
        swipe_enabled_row.set_active(init.swipe_enabled);

        let notifications_row = switch(
            &i18n("Desktop notifications"),
            &i18n("While the window is in the background."),
        );
        notifications_row.set_active(init.notifications);
        let notification_content_row = switch(
            &i18n("Show sender and subject"),
            &i18n("Off, the notification only says “New message”."),
        );
        notification_content_row.set_active(init.notification_content);
        // Greyed while nothing is posted at all.
        notifications_row
            .bind_property("active", &notification_content_row, "sensitive")
            .sync_create()
            .build();
        // When a receipt comes back (#520): not on screen 23, not removed by
        // the design, so it stays beside the other notifications.
        let notify_receipts_row = switch(
            &i18n("Read receipts"),
            &i18n("When someone opens mail you asked a receipt for."),
        );
        notify_receipts_row.set_active(init.notify_receipts);
        notifications_row
            .bind_property("active", &notify_receipts_row, "sensitive")
            .sync_create()
            .build();
        let fetch_labels: Vec<String> = FETCH_INTERVALS.iter().map(|(l, _)| i18n(l)).collect();
        let fetch_row = combo(&i18n("Check for new messages"), "", &fetch_labels);
        fetch_row.set_selected(
            FETCH_INTERVALS
                .iter()
                .position(|(_, secs)| *secs == init.fetch_interval_secs)
                .unwrap_or(0) as u32,
        );
        // One switch for every account (owner, 2026-11-08).
        let push_row = switch(
            &i18n("Instant new mail (push)"),
            &i18n("Microsoft 365 accounts check at the interval above."),
        );
        push_row.set_active(init.push);

        let theme_labels: Vec<String> = APP_THEMES.iter().map(|(l, _)| i18n(l)).collect();
        let app_theme_row = combo(
            &i18n("Style"),
            &i18n("Follows the system by default. The accent colour also comes from the system."),
            &theme_labels,
        );
        app_theme_row.set_selected(
            APP_THEMES
                .iter()
                .position(|(_, t)| *t == init.app_theme)
                .unwrap_or(0) as u32,
        );
        let background_row = switch(
            &i18n("Keep running in the background"),
            &i18n("Closing the window keeps Tegami Mail running for notifications."),
        );
        background_row.set_active(init.run_in_background);
        let autostart_row = switch(&i18n("Start at login"), "");
        autostart_row.set_active(init.autostart);
        // Starting at login only means anything if Tegami Mail stays running.
        background_row
            .bind_property("active", &autostart_row, "sensitive")
            .sync_create()
            .build();
        let single_key_row = switch(
            &i18n("Single-key shortcuts"),
            &i18n("Ctrl+? shows them all."),
        );
        single_key_row.set_active(init.single_key_shortcuts);
        // The Gmail variant, an option of the row above: it means nothing
        // with single keys off (#392), so it greys out with them.
        let gmail_keys_row = switch(
            &i18n("Gmail keys"),
            &i18n("Swaps the single-key shortcuts for Gmail equivalents."),
        );
        gmail_keys_row.set_active(init.gmail_keys);
        single_key_row
            .bind_property("active", &gmail_keys_row, "sensitive")
            .sync_create()
            .build();
        let keep_downloads_row = combo(
            &i18n("Keep downloaded mail"),
            &i18n("On this computer, for reading offline."),
            &[
                i18n("Always"),
                i18n("A year"),
                i18n("90 days"),
                i18n("30 days"),
            ],
        );
        keep_downloads_row.set_selected(
            KEEP_DOWNLOADS_DAYS
                .iter()
                .position(|d| *d == init.keep_downloads_days)
                .unwrap_or(0) as u32,
        );
        let files_large_row = combo(
            &i18n("Large attachments"),
            "",
            &[
                i18n("Ask each time"),
                i18n("Attach them anyway"),
                i18n("Upload to cloud storage and link them"),
            ],
        );
        files_large_row.set_selected(files_large_index(init.files.large));

        let s = sender.clone();
        read_mark_row
            .connect_selected_notify(move |r| s.input(PrefInput::ChangeReadMark(r.selected())));
        let s = sender.clone();
        thread_newest_first_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetThreadNewestFirst(r.is_active()));
        });
        let s = sender.clone();
        confirm_thread_delete_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetConfirmThreadDelete(r.is_active()));
        });
        let s = sender.clone();
        swipe_enabled_row
            .connect_active_notify(move |r| s.input(PrefInput::ToggleSwipeEnabled(r.is_active())));
        let s = sender.clone();
        notifications_row
            .connect_active_notify(move |r| s.input(PrefInput::ToggleNotifications(r.is_active())));
        let s = sender.clone();
        notification_content_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetNotificationContent(r.is_active()));
        });
        let s = sender.clone();
        notify_receipts_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetNotifyReceipts(r.is_active()));
        });
        let s = sender.clone();
        fetch_row.connect_selected_notify(move |r| {
            s.input(PrefInput::ChangeFetchInterval(r.selected()))
        });
        let s = sender.clone();
        push_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetPush(r.is_active()));
        });
        let s = sender.clone();
        app_theme_row
            .connect_selected_notify(move |r| s.input(PrefInput::ChangeAppTheme(r.selected())));
        let s = sender.clone();
        background_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetRunInBackground(r.is_active()));
        });
        let s = sender.clone();
        autostart_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetAutostart(r.is_active()));
        });
        let s = sender.clone();
        single_key_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetSingleKey(r.is_active()));
        });
        let s = sender.clone();
        gmail_keys_row.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetGmailKeys(r.is_active()));
        });
        let s = sender.clone();
        keep_downloads_row.connect_selected_notify(move |r| {
            s.input(PrefInput::ChangeKeepDownloads(r.selected()))
        });
        let s = sender.clone();
        files_large_row
            .connect_selected_notify(move |r| s.input(PrefInput::ChangeFilesLarge(r.selected())));

        Self {
            read_mark_row,
            thread_newest_first_row,
            confirm_thread_delete_row,
            swipe_enabled_row,
            notifications_row,
            notification_content_row,
            notify_receipts_row,
            fetch_row,
            push_row,
            app_theme_row,
            background_row,
            autostart_row,
            single_key_row,
            gmail_keys_row,
            keep_downloads_row,
            files_large_row,
        }
    }
}
