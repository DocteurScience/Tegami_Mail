//! Settings that act on data: importing and exporting settings, logs and
//! local contacts.

use adw::prelude::*;
use relm4::prelude::*;

use crate::i18n::{i18n, i18n_f};
use crate::ui::notifications::NotifyInput;
use crate::ui::preferences::PrefOutput;

use super::contacts::{reload_contacts, sync_contact_books};
use super::demo::demo_mode;
use super::model::AppModel;
use super::msg::AppMsg;

impl AppModel {
    /// Asks where to save, writes the file there with `write` and says in
    /// the status bar how it went: `saved` names the place, a failure is an
    /// error card.
    fn export_to_chosen_file(
        &self,
        title: &str,
        initial_name: &str,
        write: impl FnOnce(&std::path::Path) -> Result<(), String> + 'static,
        saved: impl FnOnce(&std::path::Path) -> String + 'static,
    ) {
        let dialog = gtk::FileDialog::builder()
            .title(title)
            .initial_name(initial_name)
            .build();
        let win = self.window.clone();
        let notif = self.notifications.sender().clone();
        dialog.save(Some(&win), gtk::gio::Cancellable::NONE, move |res| {
            let Ok(file) = res else { return };
            let Some(path) = file.path() else { return };
            let outcome = write(&path);
            let _ = notif.send(match outcome {
                Ok(()) => NotifyInput::Push {
                    text: saved(&path),
                    error: false,
                    connectivity: false,
                },
                Err(e) => NotifyInput::Push {
                    text: i18n_f("Export failed: {e}", &[("e", &(e).to_string())]),
                    error: true,
                    connectivity: false,
                },
            });
        });
    }

    /// The settings changes the router sends here (see `AppModel::route`).
    pub(super) fn on_prefs_data_pref(&mut self, pref: PrefOutput, sender: ComponentSender<Self>) {
        match pref {
            PrefOutput::SetContactBook(uid) => {
                pref!(self.contact_book = uid);
            }
            PrefOutput::LoadContactBooks => {
                if demo_mode() {
                    sender.input(AppMsg::ContactBooksLoaded(crate::contacts::demo_books()));
                    return;
                }
                let s = sender.clone();
                std::thread::spawn(move || {
                    s.input(AppMsg::ContactBooksLoaded(crate::contacts::writable_books()));
                });
            }

            PrefOutput::ExportSettings => self.export_to_chosen_file(
                &i18n("Export settings"),
                "tegami-settings.toml",
                |path| {
                    crate::config::export_bundle()
                        .and_then(|t| std::fs::write(path, t).map_err(|e| e.to_string()))
                },
                |path| {
                    i18n_f(
                        "Settings exported to {display}",
                        &[("display", &(path.display()).to_string())],
                    )
                },
            ),

            PrefOutput::ExportLog => {
                let title = i18n("Export log");
                let name = format!(
                    "tegami-log-{}.txt",
                    chrono::Local::now().format("%Y%m%d-%H%M")
                );
                // Measured now, while the model is at hand: the chooser's
                // callback runs later, without it.
                let memory = self.memory_report();
                self.export_to_chosen_file(
                    &title,
                    &name,
                    move |path| {
                        std::fs::write(path, crate::desktop::console_log::export_text(&memory))
                            .map_err(|e| e.to_string())
                    },
                    |path| {
                        i18n_f(
                            "Log exported to {display}",
                            &[("display", &(path.display()).to_string())],
                        )
                    },
                );
            }

            PrefOutput::ImportSettings => {
                let dialog = gtk::FileDialog::builder()
                    .title(i18n("Import settings"))
                    .build();
                let win = self.window.clone();
                let s = sender.clone();
                dialog.open(Some(&win), gtk::gio::Cancellable::NONE, move |res| {
                    // Only carry the choice out of the chooser's callback —
                    // the work (and any dialog) runs on a clean loop turn.
                    if let Ok(file) = res {
                        if let Some(path) = file.path() {
                            s.input(AppMsg::ImportSettingsFrom(path));
                        }
                    }
                });
            }

            PrefOutput::ContactSourcesChanged => {
                // A book switched on or off, or renamed, counts at once
                // everywhere, the sidebar's Contacts list among them.
                reload_contacts(&sender, 0);
                if self.filters.iter().any(|r| r.needs_contacts()) {
                    self.saved_addresses.refresh();
                }
                // Settings synced a saved account's books already; any it
                // did not are opened and synced now.
                sync_contact_books(&sender, false, 0);
            }
            _ => unreachable!("not routed to on_prefs_data_pref"),
        }
    }

    /// The messages the router sends here (see `AppModel::route`).
    pub(super) fn on_prefs_data(&self, msg: AppMsg) {
        match msg {
            AppMsg::ImportSettingsFrom(path) => self.import_settings_from(path),
            _ => unreachable!("not routed to on_prefs_data"),
        }
    }

    /// Second stage of ImportSettings: the chosen file, on a clean main-loop
    /// turn (working inside the chooser's completion callback froze the app
    /// when a dialog presented there). Nothing is replaced before the user
    /// says so (docs/DESIGN.md, "Organize mail and services"; screen 48).
    fn import_settings_from(&self, path: std::path::PathBuf) {
        let parent = relm4::main_application()
            .active_window()
            .unwrap_or_else(|| self.window.clone().upcast());
        let file = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let confirm = adw::AlertDialog::new(
            Some(&i18n("Replace your settings?")),
            Some(&i18n_f(
                "The settings in {file} replace yours. Keep your accounts in place: removing them first deletes their passwords.",
                &[("file", &file)],
            )),
        );
        confirm.add_response("cancel", &i18n("Cancel"));
        confirm.add_response("replace", &i18n("Replace"));
        confirm.set_response_appearance("replace", adw::ResponseAppearance::Destructive);
        confirm.set_default_response(Some("cancel"));
        confirm.set_close_response("cancel");
        let window = self.window.clone();
        confirm.connect_response(Some("replace"), move |_, _| {
            import_settings_confirmed(&window, &path);
        });
        confirm.present(Some(&parent));
    }
}

/// The import itself, once confirmed: the file's settings on disk, then
/// the offer to restart, which applies them.
#[allow(deprecated)] // adw::MessageDialog: AlertDialog would change how it looks
fn import_settings_confirmed(window: &adw::ApplicationWindow, path: &std::path::Path) {
    {
        let outcome = std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|t| crate::config::import_bundle(&t));
        match outcome {
            Ok(n) => {
                // Imported files are on disk; the running model still
                // holds the old world, so a clean restart is the
                // honest way to apply everything at once.
                // Parent on whichever window is focused (Settings,
                // normally) — a modal behind the focused window reads
                // as a freeze.
                let parent = relm4::main_application()
                    .active_window()
                    .unwrap_or_else(|| window.clone().upcast());
                let alert = adw::MessageDialog::new(
                    Some(&parent),
                    Some(i18n("Settings imported").as_str()),
                    Some(&i18n_f(
                        "{n} mail account(s), the cloud storage accounts and all \
                                 preferences were imported. Restart Tegami Mail to apply them. \
                                 Passwords and sign-ins are not part of a backup; re-enter \
                                 them on first use if this is a new machine.",
                        &[("n", &n.to_string())],
                    )),
                );
                alert.add_response("later", &i18n("Later"));
                alert.add_response("restart", &i18n("Restart Tegami Mail"));
                alert.set_response_appearance("restart", adw::ResponseAppearance::Suggested);
                alert.connect_response(None, |_, resp| {
                    if resp == "restart" {
                        // A detached shell starts the next instance
                        // once this one has quit and released its
                        // D-Bus name (started immediately, the new
                        // instance would just relay to the dying
                        // primary and exit).
                        if let Ok(exe) = std::env::current_exe() {
                            let _ = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(format!("sleep 1.5; exec '{}'", exe.display()))
                                .spawn();
                        }
                        relm4::main_application().quit();
                    }
                });
                alert.present();
            }
            Err(e) => {
                // Said where the user is looking: the confirmation that
                // led here has just closed.
                let alert =
                    adw::AlertDialog::new(Some(&i18n("Import failed")), Some(&e.to_string()));
                alert.add_response("close", &i18n("Close"));
                alert.present(Some(window));
            }
        }
    }
}
