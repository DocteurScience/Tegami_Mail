//! Settings → Address books (#408), on a sub-page of Accounts.

use super::*;

impl Preferences {
    pub(super) fn on_set_contact_books(&mut self, books: Vec<crate::contacts::Book>) {
        if let Some(c) = &self.contacts {
            c.emit(crate::ui::contact_sources::ContactSourcesInput::SetBooks(
                books,
            ));
        }
    }

    pub(super) fn on_set_local_contact_count(&mut self, n: usize) {
        if let Some(c) = &self.contacts {
            c.emit(crate::ui::contact_sources::ContactSourcesInput::SetLocalCount(n));
        }
    }

    pub(super) fn on_contacts(
        &mut self,
        out: crate::ui::contact_sources::ContactSourcesOutput,
        sender: ComponentSender<Self>,
    ) {
        use crate::ui::contact_sources::ContactSourcesOutput as Out;
        match out {
            Out::PageOpen { open, form } => {
                self.contacts_open = open;
                self.editor_open = form;
                self.editor_page = "contacts";
                self.show_sub_header("contacts", !open);
            }
            Out::Changed => {
                let _ = sender.output(PrefOutput::ContactSourcesChanged);
            }
            Out::ImportLocal => {
                let _ = sender.output(PrefOutput::ImportLocalContacts);
            }
            Out::ExportLocal => {
                let _ = sender.output(PrefOutput::ExportLocalContacts);
            }
            Out::DeleteAllLocal => {
                let _ = sender.output(PrefOutput::DeleteAllLocalContacts);
            }
            Out::SetContactNames(on) => {
                let _ = sender.output(PrefOutput::SetContactNames(on));
            }
            Out::SetContactBook(uid) => {
                let _ = sender.output(PrefOutput::SetContactBook(uid));
            }
            Out::LoadBooks => {
                let _ = sender.output(PrefOutput::LoadContactBooks);
            }
        }
    }
}
