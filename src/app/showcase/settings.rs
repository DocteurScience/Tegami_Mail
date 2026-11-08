//! Showcase hooks for the windows over the main one: shortcuts, Settings
//! and the account editor, Add to Contacts, message dialogs and Contacts.

use adw::prelude::*;
use relm4::prelude::*;

use crate::app::model::AppModel;
use crate::app::msg::AppMsg;
use crate::i18n::i18n;
use crate::ui::showcase::showcase_capture;
use crate::ui::sidebar::CtxAction;

use super::capture::input_after;

/// TEGAMI_SHOWCASE_SHORTCUTS=<height> opens the shortcut list at
/// 4 s, that tall, so all of it fits a capture (with
/// TEGAMI_SHOWCASE_TOP).
pub(super) fn stage_shortcuts(sender: &ComponentSender<AppModel>) {
    if let Some(h) = std::env::var("TEGAMI_SHOWCASE_SHORTCUTS")
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
    {
        input_after(sender, 4, AppMsg::ShowShortcuts);
        gtk::glib::timeout_add_seconds_local_once(5, move || {
            let tops = gtk::Window::toplevels();
            if let Some(w) = (0..tops.n_items())
                .filter_map(|i| tops.item(i))
                .filter_map(|o| o.downcast::<adw::Window>().ok())
                .find(|w| {
                    w.is_visible() && w.title().is_some_and(|t| t == i18n("Keyboard shortcuts"))
                })
            {
                w.set_default_size(w.default_width(), h);
            }
        });
    }
}

/// TEGAMI_SHOWCASE_ACCOUNT=new[:<provider label>] opens a blank account
/// editor a beat after the Settings window (with
/// TEGAMI_SHOWCASE_SETTINGS), so the editor itself can be captured.
pub(super) fn stage_new_account(sender: &ComponentSender<AppModel>) {
    if let Some(rest) = std::env::var("TEGAMI_SHOWCASE_ACCOUNT")
        .ok()
        .and_then(|v| v.strip_prefix("new").map(str::to_string))
    {
        let provider = rest.strip_prefix(':').map(str::to_string);
        input_after(sender, 5, AppMsg::ShowcaseNewAccount(provider));
    }
}

/// TEGAMI_SHOWCASE_ACCOUNT=N opens account N's editor a beat
/// after the Settings window (with TEGAMI_SHOWCASE_SETTINGS),
/// so the editor itself can be captured.
pub(super) fn stage_account_settings(sender: &ComponentSender<AppModel>) {
    if let Some(Ok(n)) = std::env::var("TEGAMI_SHOWCASE_ACCOUNT")
        .ok()
        .map(|v| v.parse::<u32>())
    {
        input_after(
            sender,
            5,
            AppMsg::SidebarContext(CtxAction::OpenAccountSettings(n + 1)),
        );
    }
}

/// TEGAMI_SHOWCASE_ACCOUNT_SUB=<servers|sync|folders|aliases|signatures|
/// openpgp> opens that sub-page of the account page at 7s (with
/// TEGAMI_SHOWCASE_ACCOUNT); TEGAMI_SHOWCASE_ADD_MENU=1 opens the Add
/// account menu at 6s (screens 27 to 34).
pub(super) fn stage_account_sub(sender: &ComponentSender<AppModel>) {
    if let Ok(sub) = std::env::var("TEGAMI_SHOWCASE_ACCOUNT_SUB") {
        input_after(sender, 7, AppMsg::ShowcaseAccountSub(sub));
    }
    // TEGAMI_SHOWCASE_TROUBLESHOOTING=1 opens Troubleshooting at 5s.
    if std::env::var("TEGAMI_SHOWCASE_TROUBLESHOOTING").is_ok() {
        input_after(sender, 5, AppMsg::OpenTroubleshooting);
    }
    if std::env::var("TEGAMI_SHOWCASE_ADD_MENU").is_ok() {
        input_after(sender, 6, AppMsg::ShowcaseAddMenu);
    }
}

/// TEGAMI_SHOWCASE_EDITOR_DIRTY=1 types into the open account
/// editor's Label field at 7s, so leaving an edited editor can
/// be exercised without a keyboard.
pub(super) fn stage_editor_dirty(sender: &ComponentSender<AppModel>) {
    if std::env::var("TEGAMI_SHOWCASE_EDITOR_DIRTY").is_ok() {
        input_after(sender, 7, AppMsg::ShowcaseDirtyEditor);
    }
}

/// TEGAMI_SHOWCASE_SETTINGS_GO=<category> picks that sidebar
/// category at 8s, exactly as a click on its row does.
pub(super) fn stage_settings_go(sender: &ComponentSender<AppModel>) {
    if let Ok(page) = std::env::var("TEGAMI_SHOWCASE_SETTINGS_GO") {
        input_after(sender, 8, AppMsg::ShowSettingsPage(page));
    }
}

/// TEGAMI_SHOWCASE_ADD_CONTACT=<address> opens Add to Contacts
/// for it at 6s and captures the dialog at 10s, beside the
/// TEGAMI_SHOWCASE file, so the books it offers can be checked.
#[allow(deprecated)] // adw::MessageDialog: AlertDialog would change how it looks
pub(super) fn stage_add_contact(sender: &ComponentSender<AppModel>) {
    if let Ok(addr) = std::env::var("TEGAMI_SHOWCASE_ADD_CONTACT") {
        input_after(sender, 6, AppMsg::AddContactAddr(addr));
        gtk::glib::timeout_add_seconds_local_once(10, move || {
            let tops = gtk::Window::toplevels();
            let dialog = (0..tops.n_items())
                .filter_map(|i| tops.item(i))
                .filter_map(|o| o.downcast::<adw::MessageDialog>().ok())
                .find(|d| d.is_visible());
            match (dialog, std::env::var("TEGAMI_SHOWCASE")) {
                (Some(d), Ok(path)) => {
                    showcase_capture(d.upcast_ref(), &format!("{path}.contact.png"))
                }
                _ => tracing::warn!("showcase: no Add to Contacts dialog"),
            }
        });
    }
}

/// TEGAMI_SHOWCASE_DIALOG=save|discard|cancel answers whatever
/// message dialog is on screen at 10s, captured first beside
/// the TEGAMI_SHOWCASE file.
#[allow(deprecated)] // adw::MessageDialog: AlertDialog would change how it looks
pub(super) fn stage_dialog() {
    if let Ok(answer) = std::env::var("TEGAMI_SHOWCASE_DIALOG") {
        gtk::glib::timeout_add_seconds_local_once(10, move || {
            let tops = gtk::Window::toplevels();
            let dialog = (0..tops.n_items())
                .filter_map(|i| tops.item(i))
                .filter_map(|o| o.downcast::<adw::MessageDialog>().ok())
                .find(|d| d.is_visible());
            match dialog {
                Some(d) => {
                    if let Ok(path) = std::env::var("TEGAMI_SHOWCASE") {
                        showcase_capture(d.upcast_ref(), &format!("{path}.dialog.png"));
                    }
                    // Answering is what a button click emits; the
                    // click also takes the dialog off screen, so
                    // hide it (closing would answer a second time).
                    d.response(&answer);
                    d.set_visible(false);
                }
                None => tracing::warn!("showcase: no dialog to answer"),
            }
        });
    }
}
