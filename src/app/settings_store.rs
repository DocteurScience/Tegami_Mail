//! Saving the app's settings to disk.

use relm4::prelude::*;

use crate::config;
use crate::ui::message_list::MessageListInput;
use crate::ui::message_view::MessageViewInput;

use super::model::AppModel;

impl AppModel {
    /// Persist all app settings together.
    pub(super) fn save_settings(&self) {
        config::save_privacy(config::PrivacyFile {
            allowed_senders: self.allowed_senders.clone(),
            auto_remote_content: self.auto_remote_content,
            gravatar: self.gravatar,
            avatars: self.avatars,
            contact_book: self.contact_book.clone(),
            sender_logos: self.sender_logos,
            date_style: self.date_style,
            clock_style: self.clock_style,
            fetch_interval_secs: self.fetch_interval_secs,
            push: self.push,
            blacklist: self.blacklist.clone(),
            threading: self.threading,
            thread_newest_first: self.thread_newest_first,
            confirm_thread_delete: self.confirm_thread_delete,
            notifications: self.notifications_enabled,
            notification_content: self.notification_content,
            notification_buttons: self.notification_buttons.to_list(),
            swipe_enabled: self.swipe_enabled,
            compose_default_from: self.compose_default_from.clone(),
            paste_plain: self.paste_plain,
            return_paragraph: self.return_paragraph,
            toolbar_expanded: self.toolbar_expanded,
            compose_font: self.compose_font.clone(),
            compose_font_size: self.compose_font_size,
            compose_zoom: self.compose_zoom,
            print: self.print_options,
            // Both are written: the boolean is what an older version reads.
            compose_plain: self.compose_format == config::ComposeFormat::Plain,
            compose_format: Some(self.compose_format),
            follow_plain: self.follow_plain,
            signature_position: self.signature_position,
            signature_dashes: self.signature_dashes,
            fold_quote: self.fold_quote,
            reply_header: self.reply_header.clone(),
            undo_send_secs: self.undo_send_secs,
            spellcheck: self.spellcheck,
            // Every save is after the first load, which applied it.
            spellcheck_langs: self.spellcheck_langs.clone(),
            preview_lines: self.preview_lines,
            list_columns: config::ListColumn::to_keys(&self.list_columns),
            list_headings: self.list_headings,
            list_column_scroll: self.list_column_scroll,
            list_column_widths: config::list_column_widths_to(&self.list_column_widths),
            single_key_shortcuts: self.single_key.get(),
            gmail_keys: self.gmail_keys.get(),
            keep_downloads_days: self.keep_downloads_days,
            run_in_background: self.run_in_background.get(),
            autostart: self.autostart,
            reading_pane: self.reading_pane,
            tray: self.tray_enabled,
            tray_icon: self.tray_icon,
            tray_mail: self.tray_mail,
            launcher_count: self.launcher_badge != config::LauncherBadge::Off,
            launcher_badge: Some(self.launcher_badge),
            show_remote_banner: self.show_remote_banner,
            receipts: self.receipts,
            request_receipts: self.request_receipts,
            returned_receipts: self.returned_receipts,
            notify_receipts: self.notify_receipts,
            app_theme: self.app_theme,
            text_scale: self.text_scale,
            console_mode: self.console_mode,
            status_bar_top: self.status_bar_top,
            read_mark: self.read_mark,
            files_action: self.files_prefs.action,
            files_large: self.files_prefs.large,
            files_limit_mb: self.files_prefs.limit_mb,
            start_view: self.start_view,
            folder_sort: self.folder_sort,
        });
    }

    pub(super) fn push_settings_to_children(&self) {
        self.message_view
            .emit(MessageViewInput::SetReadMark(self.read_mark));
        self.message_list
            .emit(MessageListInput::SetGravatar(self.gravatar));
        self.message_list
            .emit(MessageListInput::SetAvatars(self.avatars));
        // The formatter is a free function, so the preference has to be handed to
        // it before anything draws a date.
        self.message_list
            .emit(MessageListInput::SetSenderLogos(self.sender_logos));
        crate::datefmt::set_style(self.date_style, self.clock_style);
        crate::ui::print_preview::set_options(self.print_options);
        self.message_list.emit(MessageListInput::SetLook {
            avatars: self.avatars,
            preview_lines: self.preview_lines,
        });
        self.message_view.emit(MessageViewInput::SetFoldLook {
            preview_lines: self.preview_lines,
        });
        self.push_single_line();
        self.message_list
            .emit(MessageListInput::SetColumns(self.list_columns.clone()));
        self.message_list
            .emit(MessageListInput::SetHeadings(self.list_headings));
        self.message_list
            .emit(MessageListInput::SetColumnScroll(self.list_column_scroll));
        self.message_list.emit(MessageListInput::SetColumnWidths(
            self.list_column_widths.clone(),
        ));
        self.message_list
            .emit(MessageListInput::SetThreading(self.threading));
        self.message_view.emit(MessageViewInput::SetZoom(self.zoom));
        self.message_view
            .emit(MessageViewInput::SetUnsubscribed(self.unsubscribed_map()));
        self.message_view
            .emit(MessageViewInput::SetIdentities(self.identities_map()));
        self.message_view.emit(MessageViewInput::SetInviteAnswers(
            self.invite_answers_map(),
        ));
    }
}
