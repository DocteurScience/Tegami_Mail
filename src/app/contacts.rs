//! Contacts: the address books, syncing them, and importing vCards.

use adw::prelude::*;
use relm4::prelude::*;

use crate::i18n::{i18n, i18n_f};
use crate::ui::message_list::MessageListInput;
use crate::ui::message_view::MessageViewInput;
use crate::ui::message_window::MessageWindowInput;
use crate::ui::notifications::NotifyInput;
use crate::ui::preferences::PrefInput;

use super::demo::demo_mode;
use super::model::AppModel;
use super::msg::AppMsg;

/// Sync the address books that sync with a server, after `delay_ms` and
/// off the main thread, and read the contacts again if any did (#408).
/// Unless `force`d, a book synced in the last few minutes is left alone.
pub(super) fn sync_contact_books(sender: &ComponentSender<AppModel>, force: bool, delay_ms: u64) {
    let s = sender.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        if crate::contacts::carddav::sync_books(force) {
            s.input(AppMsg::ContactBooksSynced);
        }
    });
}

/// Tell what depends on the contacts to read them again, and read the books
/// new ones can go to, off the main thread, after `delay_ms` (EDS takes a moment to flush
/// a write to the cache the read goes through).
pub(super) fn reload_contacts(sender: &ComponentSender<AppModel>, delay_ms: u64) {
    let s = sender.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        if demo_mode() {
            s.input(AppMsg::ContactsLoaded);
            s.input(AppMsg::ContactBooksLoaded(crate::contacts::demo_books()));
            return;
        }
        s.input(AppMsg::ContactsLoaded);
        s.input(AppMsg::ContactBooksLoaded(crate::contacts::writable_books()));
    });
}

impl AppModel {
    /// Read the names "Show names from Contacts" puts on mail (#541) off
    /// the main thread, or let them go when the switch is off.
    pub(super) fn read_contact_names(&self, sender: &ComponentSender<Self>) {
        // Saved contacts always go by their names (docs/DESIGN.md,
        // "Organize mail and services"): the switch is gone.
        if demo_mode() {
            sender.input(AppMsg::ContactNamesRead(Default::default()));
            return;
        }
        let s = sender.clone();
        std::thread::spawn(move || {
            s.input(AppMsg::ContactNamesRead(crate::contacts::read_saved_names()));
        });
    }

    /// Dialog to add an email to GNOME Contacts (choosing the address book).
    /// Which books can take it is asked of EDS first, off the UI thread:
    /// that opens every book, and a remote one can go to its server (#315).
    pub(super) fn show_add_contact_dialog(
        &self,
        name: &str,
        email: &str,
        sender: &ComponentSender<Self>,
    ) {
        let (name, email) = (name.to_string(), email.to_string());
        let input = sender.input_sender().clone();
        std::thread::spawn(move || {
            let books = crate::contacts::writable_books();
            let _ = input.send(AppMsg::AddContactBooks { name, email, books });
        });
    }

    #[allow(deprecated)] // adw::MessageDialog: AlertDialog would change how it looks
    fn add_contact_dialog(
        &self,
        name: &str,
        email: &str,
        books: Vec<crate::contacts::Book>,
        sender: &ComponentSender<Self>,
    ) {
        if books.is_empty() || email.trim().is_empty() {
            self.notice_error(i18n("No address book available to add contacts"));
            return;
        }

        let dialog = adw::MessageDialog::new(
            Some(&self.window),
            Some(i18n("Add to contacts").as_str()),
            None,
        );
        dialog.add_response("cancel", &i18n("Cancel"));
        dialog.add_response("add", &i18n("Add"));
        dialog.set_response_appearance("add", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("add"));
        dialog.set_close_response("cancel");

        let form = gtk::ListBox::new();
        form.add_css_class("boxed-list");
        form.set_selection_mode(gtk::SelectionMode::None);
        let name_row = adw::EntryRow::new();
        name_row.set_title(&i18n("Name"));
        name_row.set_text(name);
        let email_row = adw::EntryRow::new();
        email_row.set_title(&i18n("Email"));
        email_row.set_text(email);
        let book_row = adw::ComboRow::new();
        book_row.set_title(&i18n("Address book"));
        let labels: Vec<&str> = books.iter().map(|b| b.name.as_str()).collect();
        book_row.set_model(Some(&gtk::StringList::new(&labels)));
        form.append(&name_row);
        form.append(&email_row);
        form.append(&book_row);
        dialog.set_extra_child(Some(&form));

        let input = sender.input_sender().clone();
        dialog.connect_response(None, move |_, resp| {
            if resp != "add" {
                return;
            }
            let name = name_row.text().trim().to_string();
            let email = email_row.text().trim().to_string();
            let idx = book_row.selected() as usize;
            let Some(book) = books.get(idx).cloned() else {
                return;
            };
            if email.is_empty() {
                return;
            }
            // Writing talks to EDS over D-Bus (blocking) — do it off the UI thread.
            let input = input.clone();
            std::thread::spawn(move || {
                let result = crate::contacts::add_or_merge(&book.uid, &name, &email);
                let _ = input.send(AppMsg::ContactAdded(result));
            });
        });
        dialog.present();
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_contacts(&mut self, msg: AppMsg, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::AddContactAddr(addr) => {
                // From an address's right-click menu: only the address is
                // known; the dialog's name field starts blank for the user.
                self.show_add_contact_dialog("", &addr, &sender);
            }

            AppMsg::AddContactBooks { name, email, books } => {
                self.add_contact_dialog(&name, &email, books, &sender);
            }
            AppMsg::ContactAdded(result) => {
                use crate::contacts::AddOutcome;
                let (text, error) = match result {
                    Ok(AddOutcome::Created) => (i18n("Added to contacts"), false),
                    Ok(AddOutcome::Merged(name)) => {
                        (i18n_f("Added email to {name}", &[("name", &name)]), false)
                    }
                    Ok(AddOutcome::AlreadyPresent(name)) => (
                        i18n_f("Already in contacts ({name})", &[("name", &name)]),
                        false,
                    ),
                    Err(e) => (
                        i18n_f("Could not add contact: {e}", &[("e", &e.to_string())]),
                        true,
                    ),
                };
                self.notifications.emit(NotifyInput::Push {
                    text,
                    error,
                    connectivity: false,
                });
            }
            AppMsg::ContactPhotosChanged => {
                // The list skips the work when avatars are off; the
                // reader's cards draw initials only, so it has nothing to do.
                self.message_list
                    .emit(MessageListInput::ContactPhotosChanged);
                self.read_contact_names(&sender);
            }

            AppMsg::ContactBooksSynced => {
                // EDS writes what it fetched just after the sync returns:
                // the books Settings lists and the names the list shows.
                reload_contacts(&sender, 1500);
                if self.filters.iter().any(|r| r.needs_contacts()) {
                    self.saved_addresses.refresh();
                }
            }

            AppMsg::ContactBooksLoaded(books) => {
                if let Some(p) = &self.prefs {
                    p.emit(PrefInput::SetContactBooks(books));
                }
            }

            AppMsg::ContactsLoaded => {
                // A contact was saved, deleted or imported: the rules on
                // whether a sender is in Contacts should know.
                if self.filters.iter().any(|r| r.needs_contacts()) {
                    self.saved_addresses.refresh();
                }
                self.read_contact_names(&sender);
            }

            AppMsg::ContactNamesRead(names) => {
                if crate::contacts::set_contact_names(names) {
                    self.message_list
                        .emit(MessageListInput::ContactNamesChanged);
                    self.message_view.emit(MessageViewInput::FacesChanged);
                    self.popouts_emit(|| MessageWindowInput::FacesChanged);
                }
            }

            _ => unreachable!("not routed to on_contacts"),
        }
    }
}

/// Nothing keeps a CardDAV or Online Accounts address book in step
/// unless an app holds it open, and off GNOME there may be no other
/// (#408). A moment after the window, not in its way.
pub(super) fn start_contacts_sync(sender: &ComponentSender<AppModel>) {
    if !demo_mode() {
        sync_contact_books(sender, false, 5000);
    }
    // Contacts is read and listed in the background, so the first visit
    // shows it at once (#477); a visit re-reads it to catch up.
    reload_contacts(sender, 4000);
    // Refresh visible avatars when the GNOME Contacts photo index
    // changes (first load finishing, or an EDS/CardDAV sync).
    crate::contacts::watch_photo_changes({
        let input = sender.input_sender().clone();
        move || {
            let _ = input.send(AppMsg::ContactPhotosChanged);
        }
    });
}
