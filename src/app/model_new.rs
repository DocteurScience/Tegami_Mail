//! The root component's model as it starts: every field's first value,
//! from what the boot steps read and the children they launched.

use std::collections::{HashMap, HashSet};

use adw::prelude::*;
use relm4::prelude::*;

use crate::config;
use crate::i18n::i18n;
use crate::ui::message_list::QuickFilters;

use super::boot::{Boot, MainMenu};
use super::children::Children;
use super::consts::{ATTACHMENT_CACHE_BUDGET, BODY_CACHE_BUDGET};
use super::demo::{demo_account_configs_saved, demo_filters, demo_mode, demo_tags};
use super::model::AppModel;
use super::transfer::TransferTally;
use super::types::UnifiedView;

impl AppModel {
    /// Builds the model. Several fields read their own settings file here,
    /// and a few have side effects (the link browser handed to the
    /// launcher, the app icon set up, the demo cache seeded): the literal
    /// keeps its field order, which is the order those happen in.
    pub(super) fn new(
        root: &adw::ApplicationWindow,
        sender: &ComponentSender<Self>,
        boot: Boot,
        children: Children,
        main_menu: MainMenu,
    ) -> Self {
        let Boot {
            prefs,
            sidebar_state,
            icon_only,
            rail_now,
            config,
            demo_data,
        } = boot;
        let config::SidebarState {
            order,
            collapsed,
            folders_expanded,
            icon_only: _,
            tree_collapsed,
            folder_order,
            tags_expanded,
            favorites,
            favorites_expanded,
            smart_expanded,
            tags_expanded_accounts,
        } = sidebar_state;
        let Children {
            sidebar,
            peek_sidebar,
            message_list,
            message_view,
            lightbox,
            notifications,
        } = children;
        let MainMenu {
            menu,
            help: help_menu,
            undo: model_undo_menu,
            reading_pane_action,
        } = main_menu;
        crate::ui::keymap::set_key_mode(prefs.single_key_shortcuts, prefs.gmail_keys);
        // Clear what an icon picked from the former gallery left on the
        // desktop, and put the shipped icon back.
        crate::desktop::app_icon::init_on_startup();
        AppModel {
            workers: HashMap::new(),
            mid_searches: HashMap::new(),
            tag_scan: None,
            tag_scan_gen: 0,
            demo_config: if demo_mode() && config.is_empty() {
                demo_account_configs_saved()
            } else {
                Vec::new()
            },
            config,
            window: root.clone(),
            prefs: None,
            accounts_win: None,
            accounts_seed: None,
            composers: Vec::new(),
            self_sender: sender.input_sender().clone(),
            reader_header: std::cell::OnceCell::new(),
            toasts: std::cell::OnceCell::new(),
            outgoing: Default::default(),
            send_ticket: 0,
            window_drop_zones: None,
            next_compose_id: 1,
            menu,
            help_menu,
            status_bar_shown: false,
            undo_menu: model_undo_menu.clone(),
            accounts: Vec::new(),
            folders: HashMap::new(),
            account_order: order,
            collapsed,
            folders_expanded,
            tree_collapsed,
            folder_order,
            selected: None,
            attachments: Vec::new(),
            lightbox,
            attachments_loading: false,
            reader_actions_collapsed: false,
            reader_compact: false,
            list_actions_collapsed: false,
            list_compact: false,
            list_fold: Default::default(),
            pending_print: None,
            mail_split: std::cell::OnceCell::new(),
            reader_toolbar: config::load_reader_toolbar(),
            reader_toolbar_widgets: std::cell::OnceCell::new(),
            reader_overflow_btn: {
                let b = gtk::Button::from_icon_name("view-more-horizontal-symbolic");
                b.set_tooltip_text(Some(i18n("Actions").as_str()));
                b.add_css_class("flat");
                b.set_visible(false);
                b
            },
            reader_tag_btn: {
                let b = gtk::Button::from_icon_name("tag-outline-symbolic");
                b.set_tooltip_text(Some(i18n("Tags").as_str()));
                b.add_css_class("flat");
                b
            },
            reader_snooze_btn: {
                let b = gtk::Button::from_icon_name("alarm-symbolic");
                b.set_tooltip_text(Some(i18n("Snooze").as_str()));
                b.add_css_class("flat");
                b
            },
            reader_move_btn: {
                let b = gtk::Button::from_icon_name("folder-symbolic");
                b.set_tooltip_text(Some(i18n("Move to…").as_str()));
                b.add_css_class("flat");
                b
            },
            attachment_cache: crate::ram_cache::RamCache::new(ATTACHMENT_CACHE_BUDGET),
            unified: false,
            unified_view: UnifiedView::Favorites,
            unified_account: None,
            unified_slices: HashMap::new(),
            unified_boot_requested: HashSet::new(),
            sent_index_requested: HashSet::new(),
            message_cache: HashMap::new(),
            search_gen: 0,
            server_search: Default::default(),
            indexed_folders: HashSet::new(),
            body_cache: crate::ram_cache::RamCache::new(BODY_CACHE_BUDGET),
            sender_cache: HashMap::new(),
            pending_draft: None,
            pending_draft_token: None,
            pending_reply: None,
            pending_draft_pick: None,
            pending_template_pick: None,
            pending_edit_as_new: None,
            pending_forward: None,
            files_prefs: config::load_files_prefs(),
            popouts: HashMap::new(),
            current_thread: Vec::new(),
            listed_thread: Vec::new(),
            reader_focus_pending: false,
            list_selection: Vec::new(),
            list_slots: Vec::new(),
            checked: None,
            compose_waiting: false,
            compose_now: false,
            list_heard: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            carried_bodies: HashMap::new(),
            remote_override: HashMap::new(),
            carried_threads: HashMap::new(),
            undo_action: None,
            redo_action: None,
            bulk_pending: 0,
            related_id_seq: u32::MAX,
            related_ids: HashMap::new(),
            folder_unread: HashMap::new(),
            pending_seen: HashMap::new(),
            pending_tags: HashMap::new(),
            pending_answers: HashMap::new(),
            dropped_unread: Default::default(),
            pending_moves: std::cell::RefCell::new(HashMap::new()),
            transfers: HashMap::new(),
            next_transfer: 0,
            next_local_id: u32::MAX - 1,
            pending_saves: HashMap::new(),
            pending_forward_attach: HashMap::new(),
            transfer_tally: TransferTally::default(),
            sidebar_split: None,
            app_title: None,
            sidebar_menu: None,
            sidebar_header: None,
            sidebar_refresh: {
                let b = gtk::Button::new();
                b.set_tooltip_text(Some(i18n("Refresh or long-press for Status Bar").as_str()));
                b.add_css_class("flat");
                b.set_valign(gtk::Align::Center);
                b
            },
            sidebar_refresh_stack: {
                let s = gtk::Stack::new();
                s.set_transition_type(gtk::StackTransitionType::Crossfade);
                s
            },
            sidebar_refresh_spinner: gtk::Spinner::new(),
            sidebar_collapsed: icon_only,
            sidebar_anim: None,
            auto_rail: false,
            rail_active: rail_now,
            sidebar_peek: false,
            peek_transition: std::rc::Rc::new(std::cell::Cell::new(false)),
            peek_split: None,
            peek_refresh_stack: {
                let s = gtk::Stack::new();
                s.set_transition_type(gtk::StackTransitionType::Crossfade);
                s
            },
            peek_refresh_spinner: gtk::Spinner::new(),
            tags_expanded,
            favorites,
            favorites_expanded,
            smart_expanded,
            tags_expanded_accounts,
            app_theme: prefs.app_theme,
            current: None,
            allowed_senders: prefs.allowed_senders,
            unsubscribed: config::load_unsubscribed(),
            invite_answers: config::load_invite_answers(),
            auto_remote_content: prefs.auto_remote_content,
            show_remote_banner: prefs.show_remote_banner,
            receipts: prefs.receipts,
            receipts_answered: config::load_receipts(),
            request_receipts: prefs.request_receipts,
            read_by: config::load_read_by(),
            returned_receipts: prefs.returned_receipts,
            notify_receipts: prefs.notify_receipts,
            receipts_filed: Default::default(),
            blacklist: prefs.blacklist,
            gravatar: prefs.gravatar,
            avatars: prefs.avatars,
            contact_book: prefs.contact_book.clone(),
            sender_logos: prefs.sender_logos,
            fetch_interval_secs: prefs.fetch_interval_secs,
            push: prefs.push,
            notifications_enabled: prefs.notifications,
            notification_content: prefs.notification_content,
            last_settings_page: None,
            list_count: String::new(),
            preview_lines: config::load_preview_lines(),
            list_columns: config::load_list_columns(),
            list_headings: prefs.list_headings,
            list_column_scroll: prefs.list_column_scroll,
            search_active: false,
            list_column_widths: config::load_list_column_widths(),
            shortcuts_win: None,
            run_in_background: std::rc::Rc::new(std::cell::Cell::new(prefs.run_in_background)),
            autostart: config::load_autostart(),
            reading_pane: prefs.reading_pane,
            single_key: std::rc::Rc::new(std::cell::Cell::new(prefs.single_key_shortcuts)),
            gmail_keys: std::rc::Rc::new(std::cell::Cell::new(prefs.gmail_keys)),
            keep_downloads_days: prefs.keep_downloads_days,
            threading: prefs.threading,
            thread_render_queued: false,
            thread_opened_at: None,
            thread_related_pending: false,
            thread_painted: false,
            thread_cache: HashMap::new(),
            deleting_attachments: Vec::new(),
            showcase_confirm_delete: false,
            thread_cache_order: Vec::new(),
            thread_key: None,
            thread_newest_first: prefs.thread_newest_first,
            reading_pane_action,
            list_header_widgets: std::cell::OnceCell::new(),
            folder_sort: prefs.folder_sort,
            console_mode: prefs.console_mode,
            status_bar_top: prefs.status_bar_top,
            read_mark: prefs.read_mark,
            // The demo (no accounts of its own) ships with tags and filter
            // rules, so its sidebar shows the Tags and Filtered Folders
            // sections; a staged tags.toml / filters.toml still wins.
            saved_addresses: crate::contacts::SavedAddresses::default(),
            quick_filters: QuickFilters::default(),
            filters: {
                let filters = config::load_filters();
                if filters.is_empty() && demo_data {
                    demo_filters()
                } else {
                    filters
                }
            },
            tags: {
                let tags = config::load_tags();
                if tags.is_empty() && demo_data {
                    demo_tags()
                } else {
                    tags
                }
            },
            tag_view: None,
            tag_view_cache: HashMap::new(),
            tag_view_loading: false,
            tag_view_dirty: false,
            smart_folders: config::load_smart_folders(),
            smart_view: None,
            smart_view_cache: HashMap::new(),
            smart_view_loading: false,
            smart_view_dirty: false,
            smart_counts_due: false,
            smart_counts_running: false,
            tag_totals: Default::default(),
            keyword_sync_at: HashMap::new(),
            flag_sync_at: HashMap::new(),
            tag_provider: crate::ui::DisplayCss::new(),
            // Demo mode gets a cache that never touches disk, seeded with the
            // sample mail: the gallery's paging, search and sort are database
            // queries, so the demo has to run them against a real database or
            // it would be exercising nothing. Not with real accounts: their
            // workers keep the real one, and the lists read it.
            cache: if demo_data {
                let c = crate::cache::Cache::in_memory().ok();
                if let Some(c) = c.as_ref() {
                    crate::backend::seed_demo_cache(c);
                }
                c
            } else {
                crate::cache::Cache::open().ok()
            },
            filter_run: None,
            filter_moved: Default::default(),
            forwarded_log: config::ForwardedLog::load(),
            auto_forwards: Vec::new(),
            auto_forward_inflight: Default::default(),
            auto_forward_tickets: Default::default(),
            filter_followups: Default::default(),
            body_hits: Default::default(),
            zoom: 100,
            confirm_thread_delete: prefs.confirm_thread_delete,
            selection_from_cards: false,
            swipe_enabled: prefs.swipe_enabled,
            compose_default_from: prefs.compose_default_from,
            paste_plain: prefs.paste_plain,
            return_paragraph: prefs.return_paragraph,
            toolbar_expanded: prefs.toolbar_expanded,
            compose_font: prefs.compose_font,
            compose_font_size: prefs.compose_font_size,
            compose_zoom: prefs.compose_zoom,
            print_options: prefs.print,
            compose_format: config::load_compose_format(),
            follow_plain: prefs.follow_plain,
            signature_position: prefs.signature_position,
            signature_dashes: prefs.signature_dashes,
            fold_quote: prefs.fold_quote,
            reply_header: prefs.reply_header,
            undo_send_secs: prefs.undo_send_secs,
            spellcheck: prefs.spellcheck,
            spellcheck_langs: prefs.spellcheck_langs,
            auto_fetch_source: None,
            last_wake: None,
            notifications,
            welcome: None,
            notify_count: 0,
            busy: HashSet::new(),
            copies_left: 0,
            sidebar,
            peek_sidebar,
            message_list,
            message_view,
            showing_outbox: false,
            outbox_by_account: HashMap::new(),
            snoozed: super::snooze::load_snoozed(),
            snoozed_held: HashMap::new(),
            snooze_waking: HashMap::new(),
            snooze_undone: HashMap::new(),
            snooze_until: None,
            server_identities: HashMap::new(),
            unified_emit_queued: std::cell::Cell::new(false),
            unified_dirty: std::cell::Cell::new(false),
            thread_links_due: std::cell::Cell::new(false),
            unified_emitted_at: std::cell::Cell::new(std::time::Instant::now()),
            list_fp: std::cell::Cell::new(None),
            unified_unfilled: std::collections::HashSet::new(),
        }
    }
}
