//! The child components: their launch, and their outputs turned into the
//! messages they mean to the root component.

use relm4::prelude::*;

use crate::ui::message_list::{MessageList, MessageListOutput};
use crate::ui::message_view::{MessageView, MessageViewOutput};
use crate::ui::notifications::{NotificationCenter, NotifyOutput};
use crate::ui::preferences::PrefOutput;
use crate::ui::sidebar::{Sidebar, SidebarOutput};

use super::boot::{sidebar_init, Boot};
use super::model::AppModel;
use super::msg::AppMsg;
use super::types::UnifiedView;

/// What the message list reports, as the message it means here.
fn message_list_output_msg(out: MessageListOutput) -> AppMsg {
    match out {
        MessageListOutput::Selected {
            message,
            thread,
            solo,
        } => AppMsg::MessageSelected {
            message,
            thread,
            solo,
        },
        MessageListOutput::SelectionKeys(keys) => AppMsg::SelectionKeys(keys),
        MessageListOutput::Checked(count) => AppMsg::ListChecked(count),
        MessageListOutput::ViewMissed => AppMsg::EmitUnifiedNow,
        MessageListOutput::MoveMessages {
            account_id,
            dest,
            messages,
        } => AppMsg::MoveMessagesTo {
            account_id,
            dest,
            messages,
        },
        MessageListOutput::DeleteThread { messages } => AppMsg::DeleteThread(messages),
        MessageListOutput::ThreadGrew { message, thread } => AppMsg::ThreadGrew {
            message: Box::new(message),
            thread,
        },
        MessageListOutput::ThreadsListed { groups } => AppMsg::ThreadsListed { groups },
        MessageListOutput::CountChanged(text) => AppMsg::ListCount(text),
        MessageListOutput::Activated { message, thread } => {
            AppMsg::OpenMessageWindow { message, thread }
        }
        MessageListOutput::Action {
            action,
            message,
            conversation,
        } => AppMsg::RowAction {
            action,
            message,
            conversation,
        },
        MessageListOutput::SetTag {
            message,
            keyword,
            add,
        } => AppMsg::SetTag {
            message,
            keyword,
            add,
        },
        MessageListOutput::SetTagMany {
            messages,
            keyword,
            add,
        } => AppMsg::SetTagMany {
            messages,
            keyword,
            add,
        },
        MessageListOutput::Bulk { action, messages } => AppMsg::Bulk { action, messages },
        MessageListOutput::Snooze { messages, choice } => {
            AppMsg::SnoozeMessages { messages, choice }
        }
        MessageListOutput::MoveTo {
            messages,
            offer_whole,
            x,
            y,
        } => AppMsg::ListMoveTo {
            messages,
            offer_whole,
            x,
            y,
        },
        MessageListOutput::SelectionCleared => AppMsg::ClearReader,
        MessageListOutput::OpenedByKey { already_open } => AppMsg::OpenedByKey { already_open },
        MessageListOutput::SearchActive(active) => AppMsg::SearchActive(active),
        MessageListOutput::FindInBodies { terms } => AppMsg::FindInBodies { terms },
        MessageListOutput::FindOnServers { terms, scope } => AppMsg::FindOnServers { terms, scope },
        MessageListOutput::SortChanged(key) => AppMsg::ListSortChanged(key),
        MessageListOutput::ColumnWidths(widths) => AppMsg::ListColumnWidths(widths),
    }
}

/// What the message view reports, as the message it means here.
fn message_view_output_msg(out: MessageViewOutput) -> AppMsg {
    match out {
        MessageViewOutput::AllowSender(addr) => AppMsg::AllowSender(addr),
        MessageViewOutput::SetRemote {
            account_id,
            id,
            show,
        } => AppMsg::SetRemoteContent {
            account_id,
            id,
            show,
        },
        MessageViewOutput::OpenWindow(m) => AppMsg::OpenMessageWindow {
            message: *m,
            thread: Vec::new(),
        },
        MessageViewOutput::CardAction { action, message } => AppMsg::CardAction { action, message },
        MessageViewOutput::AttachmentAction {
            account_id,
            id,
            index,
            verb,
        } => AppMsg::CardAttachment {
            account_id,
            id,
            index,
            verb,
        },
        MessageViewOutput::CardMenu { message, x, y, hit } => AppMsg::CardMenu {
            message,
            x,
            y,
            hit,
            popout: None,
        },
        MessageViewOutput::MarkSeen { account_id, id } => {
            AppMsg::ThreadMessageSeen { account_id, id }
        }
        MessageViewOutput::SelectCards(keys) => AppMsg::SelectCards(keys),
        MessageViewOutput::ComposeTo(addr) => AppMsg::ComposeTo(addr),
        MessageViewOutput::AddContactAddr(addr) => AppMsg::AddContactAddr(addr),
        MessageViewOutput::ReloadBody(m) => AppMsg::ReloadBody(m),
        MessageViewOutput::Notice(text) => AppMsg::Notice(text),
        MessageViewOutput::Zoom(step) => AppMsg::ZoomMessage(step),
        MessageViewOutput::Unsubscribe { message, info } => AppMsg::Unsubscribe { message, info },
        MessageViewOutput::ReceiptAnswer { message, send } => {
            AppMsg::ReceiptAnswer { message, send }
        }
        MessageViewOutput::InviteAction {
            message,
            invite,
            action,
        } => AppMsg::InviteAction {
            message,
            invite,
            action,
        },
    }
}

/// What the notifications reports, as the message it means here.
fn notifications_output_msg(out: NotifyOutput) -> AppMsg {
    match out {
        NotifyOutput::CountChanged(n) => AppMsg::NotifyCount(n),
        NotifyOutput::ExportLog => AppMsg::Pref(PrefOutput::ExportLog),
        NotifyOutput::Shown(shown) => AppMsg::StatusBarShown(shown),
    }
}

/// Route a sidebar instance's output to the app — the docked rail and the
/// floating peek panel are two instances, driven alike.
pub(super) fn sidebar_output_msg(out: SidebarOutput) -> AppMsg {
    match out {
        SidebarOutput::UnifiedFavoritesSelected => AppMsg::UnifiedSelected(UnifiedView::Favorites),
        SidebarOutput::UnifiedTagsSelected => AppMsg::TagSelected {
            keyword: None,
            account: None,
        },
        SidebarOutput::SmartSelected(id) => AppMsg::SmartSelected(id),
        SidebarOutput::TagSelected { keyword, account } => AppMsg::TagSelected {
            keyword: Some(keyword),
            account,
        },
        SidebarOutput::ToggleAccountTags(id) => AppMsg::ToggleAccountTags(id),
        SidebarOutput::RefreshRequested => AppMsg::Refresh,
        SidebarOutput::StatusBarRequested => AppMsg::ToggleNotifications,
        SidebarOutput::NewTagRequested => AppMsg::NewTagDialog { apply: false },
        SidebarOutput::OutboxSelected => AppMsg::ShowOutbox,
        SidebarOutput::FolderSelected {
            account_id,
            folder_id,
            name,
            path,
        } => AppMsg::FolderSelected {
            account_id,
            folder_id,
            name,
            path,
        },
        SidebarOutput::AccountFolded { id, folded } => AppMsg::AccountFolded { id, folded },
        SidebarOutput::ToggleCustomFolders(id) => AppMsg::ToggleCustomFolders(id),
        SidebarOutput::CollapsedChanged(collapsed) => AppMsg::SidebarCollapsed(collapsed),
        SidebarOutput::SectionsOpen {
            tags,
            favorites,
            smart,
        } => AppMsg::SidebarSectionsOpen {
            tags,
            favorites,
            smart,
        },
        SidebarOutput::FolderNodeCollapsed {
            account_id,
            path,
            collapsed,
        } => AppMsg::FolderNodeCollapsed {
            account_id,
            path,
            collapsed,
        },
        SidebarOutput::AddAccount => AppMsg::AddFirstAccount,
        SidebarOutput::Context(action) => AppMsg::SidebarContext(action),
        SidebarOutput::MoveMessages {
            dest_account,
            dest,
            items,
        } => AppMsg::DropMoveMessages {
            dest_account,
            dest,
            items,
        },
        SidebarOutput::MoveFolder {
            account_id,
            path,
            dest,
        } => AppMsg::MoveFolder {
            account_id,
            path,
            dest,
        },
        SidebarOutput::ReorderFolders { account_id, paths } => {
            AppMsg::ReorderFolders { account_id, paths }
        }
        SidebarOutput::PlaceFolder {
            account_id,
            path,
            dest,
            target,
            after,
        } => AppMsg::PlaceFolder {
            account_id,
            path,
            dest,
            target,
            after,
        },
        SidebarOutput::SetFolderSort { account_id, sort } => {
            AppMsg::SetAccountFolderSort { account_id, sort }
        }
    }
}

/// The child components, launched before the model that owns them.
pub(super) struct Children {
    pub(super) sidebar: Controller<Sidebar>,
    pub(super) peek_sidebar: Controller<Sidebar>,
    pub(super) message_list: Controller<MessageList>,
    pub(super) message_view: Controller<MessageView>,
    pub(super) lightbox: Controller<crate::ui::lightbox::Lightbox>,
    pub(super) notifications: Controller<NotificationCenter>,
}

impl Children {
    /// Launches every child with its output forwarded to the root.
    pub(super) fn launch(boot: &Boot, sender: &ComponentSender<AppModel>) -> Self {
        let sidebar = Sidebar::builder()
            .launch(sidebar_init(&boot.sidebar_state, boot.rail_now, false))
            .forward(sender.input_sender(), sidebar_output_msg);
        // The peek panel's rows: a second, always-expanded instance.
        let peek_sidebar = Sidebar::builder()
            .launch(sidebar_init(&boot.sidebar_state, false, true))
            .forward(sender.input_sender(), sidebar_output_msg);

        let message_list = MessageList::builder()
            .launch(())
            .forward(sender.input_sender(), message_list_output_msg);

        let message_view = MessageView::builder()
            .launch(())
            .forward(sender.input_sender(), message_view_output_msg);

        let lightbox = crate::ui::lightbox::Lightbox::builder().launch(()).detach();

        let notifications = NotificationCenter::builder()
            .launch(())
            .forward(sender.input_sender(), notifications_output_msg);
        Children {
            sidebar,
            peek_sidebar,
            message_list,
            message_view,
            lightbox,
            notifications,
        }
    }
}
