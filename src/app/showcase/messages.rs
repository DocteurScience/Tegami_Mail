//! The showcase's own messages, which stage what a timer cannot reach from
//! outside the model.

use adw::prelude::*;
use relm4::prelude::*;

use crate::app::model::AppModel;
use crate::app::msg::AppMsg;
use crate::models::FolderKind;
use crate::ui::compose::ComposeInput;

impl AppModel {
    /// The messages the router sends here (see `AppModel::route`).
    pub(in crate::app) fn on_showcase(&mut self, msg: AppMsg, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::ShowcasePopout => {
                if let Some(m) = self.current.clone() {
                    let thread = self.current_thread.clone();
                    sender.input(AppMsg::OpenMessageWindow { message: m, thread });
                }
            }

            AppMsg::ShowcaseFolder { kind, account } => {
                let account = account.unwrap_or_else(|| self.active_account());
                let found = self
                    .folders
                    .get(&account)
                    .and_then(|fs| fs.iter().find(|f| f.kind == kind))
                    .map(|f| (f.id, f.name.clone(), f.path.clone()));
                if let Some((id, name, path)) = found {
                    self.select_folder(account, id, name, path);
                }
            }

            AppMsg::ShowcaseNotify => {
                let account = self.active_account();
                let inbox = self
                    .folders
                    .get(&account)
                    .and_then(|fs| fs.iter().find(|f| f.kind == FolderKind::Inbox))
                    .map(|f| f.id);
                let newest = inbox.and_then(|f| {
                    self.message_cache
                        .get(&(account, f))
                        .and_then(|ms| ms.iter().max_by_key(|m| m.timestamp))
                        .map(|m| (f, m.id, m.from_name.clone(), m.subject.clone()))
                });
                match newest {
                    Some((folder, id, from, subject)) => {
                        self.announce_new_mail(account, folder, id, &from, &subject, 0, true);
                    }
                    None => tracing::warn!(
                        "showcase notify: no Inbox message cached for account {account}"
                    ),
                }
            }
            AppMsg::ShowcaseDeleteAttachment { name, confirmed } => {
                match self.attachment_target(&name, None) {
                    Some(target) if confirmed => sender.input(AppMsg::DeleteAttachment(target)),
                    Some(target) => sender.input(AppMsg::AskDeleteAttachment(target)),
                    None => tracing::warn!("showcase: no attachment called {name:?} on screen"),
                }
            }
            AppMsg::ShowcaseComposeMenu { from } => {
                if let Some(r) = self.composers.last() {
                    r.controller.emit(if from {
                        ComposeInput::FromMenu
                    } else {
                        ComposeInput::OverflowMenu
                    });
                }
            }
            AppMsg::ShowcaseComposeFrom(n) => {
                if let Some(r) = self.composers.last() {
                    r.controller.emit(ComposeInput::ShowcaseFrom(n));
                }
            }
            AppMsg::ShowcaseForward { account_id, id } => {
                let found = self
                    .current_thread
                    .iter()
                    .find(|m| m.account_id == account_id && m.id == id)
                    .cloned()
                    .or_else(|| {
                        self.message_cache
                            .values()
                            .flatten()
                            .find(|m| m.account_id == account_id && m.id == id)
                            .cloned()
                    });
                match found {
                    Some(m) => self.forward(m, &sender),
                    None => tracing::warn!("showcase forward: {account_id}:{id} is not loaded"),
                }
            }
            AppMsg::ShowcaseComposeUndo => {
                if let Some(r) = self.composers.last() {
                    r.controller.emit(ComposeInput::ShowcaseHistory(0));
                }
            }
            AppMsg::ShowcaseBurger => {
                if let Some(m) = self.sidebar_menu.as_ref() {
                    // Focus first: a click on a MenuButton focuses it before
                    // the popover opens, and popup() alone skips that — the
                    // difference between the two is a bug's hiding place.
                    m.grab_focus();
                    m.popup();
                    let mut chain = Vec::new();
                    let mut node = gtk::prelude::GtkWindowExt::focus(&self.window);
                    while let Some(w) = node {
                        chain.push(format!("{}", w.type_()));
                        node = w.parent();
                    }
                    tracing::info!(
                        target: "tegami::compose::undo",
                        "burger: opened, focus now [{}]",
                        chain.join(" < ")
                    );
                }
            }
            AppMsg::ShowcaseComposeFormat(format) => {
                if let Some(r) = self.composers.last() {
                    r.controller.emit(ComposeInput::SetFormat(format));
                }
            }

            AppMsg::ShowcaseComposeClose => {
                if let Some(r) = self.composers.last() {
                    r.controller.emit(ComposeInput::Cancel);
                }
            }

            AppMsg::ShowcaseNewAccount(provider) => {
                if let Some(acc) = &self.accounts_win {
                    acc.emit(crate::ui::accounts::AccountsInput::AddAccount);
                    if let Some(label) = provider {
                        acc.emit(crate::ui::accounts::AccountsInput::DebugProvider(label));
                    }
                }
            }
            AppMsg::ShowcaseDirtyEditor => {
                if let Some(acc) = &self.accounts_win {
                    acc.emit(crate::ui::accounts::AccountsInput::DebugEditLabel(
                        "Edited in the capture".into(),
                    ));
                }
            }

            AppMsg::ShowcaseEditFilter(i) => {
                if let Some(a) = &self.accounts_win {
                    a.emit(crate::ui::accounts::AccountsInput::EditFilter(i));
                }
            }
            _ => unreachable!("not routed to on_showcase"),
        }
    }
}
