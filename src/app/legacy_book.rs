//! The former Tegami Mail address book, gone with docs/DESIGN.md
//! ("Organize mail and services"): when it still holds contacts, the app
//! offers to export them as one vCard file, to open in GNOME Contacts. The
//! book's file is never deleted; once exported it is renamed, so the offer
//! stops. "Later" asks again at the next start.

use adw::prelude::*;
use relm4::prelude::*;

use crate::i18n::{i18n, i18n_f, ni18n_f};
use crate::ui::notifications::NotifyInput;

use super::demo::demo_mode;
use super::model::AppModel;
use super::msg::AppMsg;

/// A moment after the window, not in its way: count the former book's
/// contacts off the main thread, and offer the export when there are any.
pub(super) fn check_legacy_book(sender: &ComponentSender<AppModel>) {
    if demo_mode() {
        return;
    }
    let s = sender.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(3));
        let n = crate::contacts::local::count();
        if n > 0 {
            s.input(AppMsg::LegacyBook(n));
        }
    });
}

impl AppModel {
    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_legacy_book(&mut self, msg: AppMsg, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::LegacyBook(n) => self.offer_legacy_export(n, &sender),
            AppMsg::LegacyBookExported(result) => {
                let (text, error) = match result {
                    Ok((n, file)) => (
                        ni18n_f(
                            "{n} contact exported to {file}. Open the file in GNOME Contacts to keep it.",
                            "{n} contacts exported to {file}. Open the file in GNOME Contacts to keep them.",
                            n as u32,
                            &[("n", &n.to_string()), ("file", &file)],
                        ),
                        false,
                    ),
                    Err(e) => (
                        i18n_f("Could not export the contacts: {e}", &[("e", &e)]),
                        true,
                    ),
                };
                self.notifications.emit(NotifyInput::Push {
                    text,
                    error,
                    connectivity: false,
                });
            }
            _ => unreachable!("not routed to on_legacy_book"),
        }
    }

    fn offer_legacy_export(&self, n: usize, sender: &ComponentSender<Self>) {
        let dialog = adw::AlertDialog::new(
            Some(&i18n("Export your Tegami Mail contacts")),
            Some(&ni18n_f(
                "The Tegami Mail address book holds {n} contact. Tegami Mail now uses the address books GNOME provides: export it as a vCard file, then open the file in GNOME Contacts.",
                "The Tegami Mail address book holds {n} contacts. Tegami Mail now uses the address books GNOME provides: export them as a vCard file, then open the file in GNOME Contacts.",
                n as u32,
                &[("n", &n.to_string())],
            )),
        );
        dialog.add_response("later", &i18n("Later"));
        dialog.add_response("export", &i18n("Export…"));
        dialog.set_response_appearance("export", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("export"));
        dialog.set_close_response("later");
        let window = self.window.clone();
        let s = sender.clone();
        dialog.connect_response(Some("export"), move |_, _| {
            choose_and_export(&window, &s);
        });
        dialog.present(Some(&self.window));
    }
}

/// Ask where to save the file, then export there off the main thread.
fn choose_and_export(window: &adw::ApplicationWindow, sender: &ComponentSender<AppModel>) {
    let dialog = gtk::FileDialog::builder()
        .title(i18n("Export contacts"))
        .initial_name("tegami-contacts.vcf")
        .build();
    let s = sender.clone();
    dialog.save(Some(window), gtk::gio::Cancellable::NONE, move |res| {
        let Some(path) = res.ok().and_then(|f| f.path()) else {
            return;
        };
        let s = s.clone();
        std::thread::spawn(move || {
            let result = crate::contacts::local::export_to(&path).map(|n| {
                crate::contacts::local::set_aside();
                (n, path.display().to_string())
            });
            s.input(AppMsg::LegacyBookExported(result));
        });
    });
}
