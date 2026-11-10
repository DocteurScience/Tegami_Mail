//! The first-run wizard (docs/DESIGN.md, "First run"; screens 49 to 52),
//! shown when Tegami Mail starts with no account: welcome, the accounts to
//! add, an email account by hand, and the last page. A sequence in an
//! `AdwNavigationView`, in the system's colours, with back and close in
//! the header bar.
//!
//! Accounts apply through the app's own pipelines ([`WelcomeOutput`]), so
//! the wizard keeps nothing of its own. It sets no preference either: the
//! privacy defaults are those of Preferences > Privacy.

use adw::prelude::*;
use relm4::prelude::*;

use crate::config::Protocol;
use crate::config::{blank_account, AccountConfig};
use crate::i18n::{i18n, i18n_f};
use crate::ui::accounts::{Provider, PROVIDERS};
use crate::worker::{self, ConnTest};

mod goa;
mod manual;
mod messages;
mod pages;
mod steps;

use goa::{build_accounts_page, rebuild_goa_rows, AccountsPage};
use manual::{build_form_page, FormPage};
pub(crate) use messages::{WelcomeCmd, WelcomeInput, WelcomeOutput};
use pages::{build_done_page, build_welcome_page, column, heading, nav_page, pill, sentence};

pub(crate) struct Welcome {
    goa: Vec<crate::services::goa::GoaMailAccount>,
    /// Emails added so far, either way, in order: the last page names the
    /// first.
    added: Vec<String>,
    /// Generation counter for connection tests and sign-ins: a newer one
    /// supersedes a running one, whose late result is ignored.
    test_seq: u32,
}

pub(crate) struct WelcomeWidgets {
    nav: adw::NavigationView,
    accounts: AccountsPage,
    form: FormPage,
    done: adw::NavigationPage,
    /// The last page's sentence, naming the account.
    done_text: gtk::Label,
}

impl Component for Welcome {
    type Init = ();
    type Input = WelcomeInput;
    type Output = WelcomeOutput;
    type CommandOutput = WelcomeCmd;
    type Root = adw::Window;
    type Widgets = WelcomeWidgets;

    fn init_root() -> Self::Root {
        let win = adw::Window::new();
        win.set_default_size(520, 560);
        win.set_title(Some(i18n("Welcome to Tegami Mail").as_str()));
        win
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let existing: Vec<String> = crate::config::load()
            .unwrap_or_default()
            .into_iter()
            .map(|a| a.email.to_ascii_lowercase())
            .collect();
        let mut goa = crate::services::goa::list_mail_accounts();
        goa.retain(|g| g.can_connect() && !existing.contains(&g.email.to_ascii_lowercase()));
        let model = Welcome {
            goa,
            added: Vec::new(),
            test_seq: 0,
        };

        let nav = adw::NavigationView::new();
        nav.add(&build_welcome_page(&sender));
        let accounts = build_accounts_page(&model, &sender);
        let form = build_form_page(&sender);
        let (done, done_text) = build_done_page(&sender);
        root.set_content(Some(&nav));
        // Back from Online Accounts in GNOME Settings: what was added there
        // shows up without a button to look again.
        {
            let s = sender.clone();
            root.connect_is_active_notify(move |w| {
                if w.is_active() {
                    s.input(WelcomeInput::RescanGoa);
                }
            });
        }

        let widgets = WelcomeWidgets {
            nav,
            accounts,
            form,
            done,
            done_text,
        };
        rebuild_goa_rows(&widgets.accounts.list, &model.goa, &sender);
        showcase(&root, &sender);
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            WelcomeInput::Start => self.on_start(widgets),
            WelcomeInput::OpenForm => self.on_open_form(widgets),
            WelcomeInput::Continue => self.on_continue(widgets, root),
            WelcomeInput::ProviderChanged => self.on_provider_changed(widgets),
            WelcomeInput::TestAndAdd => self.on_test_and_add(widgets, sender),
            WelcomeInput::ImportGoa(index) => self.on_import_goa(index, widgets, sender),
            WelcomeInput::RescanGoa => self.on_rescan_goa(widgets, sender),
            WelcomeInput::OpenOnlineAccounts => crate::ui::accounts::open_online_accounts(),
            WelcomeInput::Finish => root.close(),
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            WelcomeCmd::Tested {
                account,
                result,
                seq,
            } => self.on_tested(account, result, seq, widgets, sender),
            WelcomeCmd::SignedIn {
                account,
                result,
                seq,
            } => self.on_signed_in(account, result, seq, widgets, sender),
        }
    }
}

/// Screenshot hook: TEGAMI_SHOWCASE_WIZARD=accounts or email walks to that
/// page (with TEGAMI_WELCOME=1 to open the wizard over the demo), and
/// TEGAMI_SHOWCASE_WIZARD_SHOT=path captures the wizard alone after a beat
/// (TEGAMI_SHOWCASE is the main window's).
fn showcase(root: &adw::Window, sender: &ComponentSender<Welcome>) {
    if let Some(page) = std::env::var("TEGAMI_SHOWCASE_WIZARD")
        .ok()
        .filter(|p| p == "accounts" || p == "email")
    {
        let s = sender.clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
            s.input(WelcomeInput::Start);
            if page == "email" {
                s.input(WelcomeInput::OpenForm);
            }
        });
    }
    if let Ok(path) = std::env::var("TEGAMI_SHOWCASE_WIZARD_SHOT") {
        let win = root.clone();
        gtk::glib::timeout_add_seconds_local_once(4, move || {
            crate::ui::showcase::showcase_capture(win.upcast_ref(), &path);
        });
    }
}
