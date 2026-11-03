//! Where composers live: each in a window of its own (docs/DESIGN.md,
//! "Replying and composing").

use adw::prelude::*;
use relm4::prelude::*;

use crate::ui::compose::{Compose, ComposeInput, ComposePrefill};
use crate::ui::message_list::MessageListInput;

use super::model::AppModel;
use super::msg::AppMsg;

/// A compose window (New Message, compose-to, a reply, a forward, an
/// edited draft) and its component. Both refs must stay alive: the window
/// holds the content, the controller holds the component root.
pub(super) struct ComposeHost {
    pub(super) id: u32,
    pub(super) controller: Controller<Compose>,
    pub(super) window: adw::Window,
}

impl AppModel {
    /// A new message with these recipients, from the account being looked at.
    pub(super) fn compose_addressed(
        &mut self,
        prefill: ComposePrefill,
        sender: &ComponentSender<Self>,
    ) {
        let account = self
            .current
            .as_ref()
            .map(|m| m.account_id)
            .unwrap_or_else(|| self.active_account());
        let (account, prefill) = self.new_message_from(account, prefill);
        self.open_compose(account, prefill, sender);
    }

    /// Open a compose window.
    pub(super) fn open_compose(
        &mut self,
        account_id: u32,
        prefill: ComposePrefill,
        sender: &ComponentSender<Self>,
    ) {
        let (id, init) = self.build_compose_init(account_id, prefill, true);
        let controller = self.spawn_compose(init, sender);
        let window = self.compose_window_host(controller.widget(), id, sender);
        controller.emit(ComposeInput::FocusInitial);
        self.composers.push(ComposeHost {
            id,
            controller,
            window,
        });
    }

    /// Tear down a composer by id (from a Close output or a window's close-request).
    pub(super) fn close_compose(&mut self, id: u32) {
        tracing::debug!(target: "tegami::compose", "composer {id} closed");
        if let Some(pos) = self.composers.iter().position(|h| h.id == id) {
            let host = self.composers.remove(pos);
            host.window.set_content(None::<&gtk::Widget>);
            host.window.destroy();
        }
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_compose_host(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::ComposeClosed(id) => {
                self.close_compose(id);
                self.message_list.emit(MessageListInput::ReclaimFocus);
            }

            AppMsg::ComposeCloseRequested(id) => match self.composers.iter().find(|h| h.id == id) {
                Some(h) => h.controller.emit(ComposeInput::Cancel),
                None => self.close_compose(id),
            },

            _ => unreachable!("not routed to on_compose_host"),
        }
    }
}
