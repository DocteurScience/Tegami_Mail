//! Building the list: its first state, and the scroll and click handlers
//! `init` hangs on the widgets the view made.

use super::rows::LIST_WINDOW;
use super::{MessageList, MessageListInput, QuickFilters, SearchScope, SortOrder};
use crate::ui::message_row::RowShared;
use adw::prelude::*;
use relm4::prelude::*;

impl MessageList {
    /// The list before any folder is shown.
    pub(super) fn initial(
        list_view: gtk::ListView,
        shared: std::rc::Rc<RowShared>,
        input: relm4::Sender<MessageListInput>,
        color_provider: crate::ui::DisplayCss,
    ) -> Self {
        MessageList {
            list_view,
            shared,
            input,
            rebuild_queued: None,
            view_key: None,
            view_fp: 0,
            views: std::collections::VecDeque::new(),
            pending_restore: None,
            switched: false,
            look_stale: Default::default(),
            pending_select: None,
            late_select: None,
            all: Vec::new(),
            search_pool: Vec::new(),
            pool_folders: Default::default(),
            search_text: Default::default(),
            search_gen: Default::default(),
            search_hits: None,
            parsed: Default::default(),
            text_hits: None,
            server_round: 0,
            server_searching: false,
            scope: SearchScope::AllFolders,
            search_entry: None,
            scope_picker: None,
            search_open: false,
            search_closed_at: None,
            bars: None,
            shown: Vec::new(),
            total_matches: 0,
            index_complete: true,
            loaded: false,
            at_bottom: false,
            window: LIST_WINDOW,
            tail: Vec::new(),
            threads_gen: std::cell::Cell::new(0),
            threads_job: std::cell::Cell::new(None),
            query: String::new(),
            gravatar: false,
            avatars: true,
            sender_logos: false,
            preview_lines: 1,
            colorize: false,
            account_colors: std::collections::HashMap::new(),
            color_provider,
            face_gen: 0,
            tags_gen: 0,
            thread_links: Vec::new(),
            source_threads: std::cell::RefCell::new(None),
            thread_summaries: std::collections::HashMap::new(),
            own_addresses: Default::default(),
            gmail_keys: crate::config::load_privacy().gmail_keys,
            groups: std::collections::HashMap::new(),
            listed_folders: std::collections::HashSet::new(),
            asked_threads: std::collections::HashSet::new(),
            recheck_accounts: std::collections::HashSet::new(),
            selected_id: None,
            selected_ids: Vec::new(),
            emitted_thread: Vec::new(),
            selection_count: 0,
            checking: false,
            nav_direction: 1,
            from_reader: 0,
            reader_keys: Vec::new(),
            show_recipient: false,
            restorable: false,
            snooze_offer: None,
            in_junk: false,
            in_drafts: false,
            in_templates: false,
            msg_thread: std::collections::HashMap::new(),
            thread_members: std::collections::HashMap::new(),
            scroller: None,
            sort: SortOrder::DateNewest,
            quick: QuickFilters::default(),
            contact_addresses: None,
            last_count: String::new(),
            threading: true,
            single_line: false,
            columns: crate::config::ListColumn::DEFAULT.to_vec(),
            graph_accounts: std::collections::HashSet::new(),
            graph_in_view: false,
            day_headings: false,
            pane_width: 0,
            bulk_tag_btn: gtk::Button::new(),
        }
    }
}

/// Report the rows' width as the view is allocated.
pub(super) fn watch_pane_width(
    scroller: &gtk::ScrolledWindow,
    sender: &ComponentSender<MessageList>,
) {
    // The rows' width, which fitted columns fit into: the view's
    // page across, set as it is allocated.
    let input = sender.input_sender().clone();
    scroller
        .hadjustment()
        .connect_notify_local(Some("page-size"), move |adj, _| {
            let _ = input.send(MessageListInput::PaneWidth(adj.page_size() as i32));
        });
}

/// Report reaching the bottom, and nearing the end, of the list.
pub(super) fn watch_bottom(scroller: &gtk::ScrolledWindow, sender: &ComponentSender<MessageList>) {
    // Whether the list sits at its bottom, for the spinner that says
    // more of the folder is on its way.
    let adj = scroller.vadjustment();
    let input = sender.input_sender().clone();
    let at_bottom = std::rc::Rc::new(std::cell::Cell::new(false));
    let near = std::rc::Rc::new(std::cell::Cell::new(false));
    let check = move |adj: &gtk::Adjustment| {
        let bottom =
            adj.upper() > adj.page_size() && adj.value() + adj.page_size() >= adj.upper() - 1.0;
        if at_bottom.replace(bottom) != bottom {
            let _ = input.send(MessageListInput::AtBottom(bottom));
        }
        // Two screens from the end, the next rows are added, so
        // they are there before the scroll reaches them.
        let close = adj.upper() > adj.page_size()
            && adj.value() + adj.page_size() >= adj.upper() - 2.0 * adj.page_size();
        if near.replace(close) != close && close {
            let _ = input.send(MessageListInput::NearEnd);
        }
    };
    let c = check.clone();
    adj.connect_value_changed(move |a| c(a));
    adj.connect_changed(move |a| check(a));
}
