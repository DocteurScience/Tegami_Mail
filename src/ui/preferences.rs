//! The preferences dialog (docs/DESIGN.md, "Preferences"): five pages,
//! assembled in `dialog` from rows the builders in `preferences/pages`
//! make; Accounts (`ui/accounts.rs`), OpenPGP, Cloud Storage, Translation
//! and Contacts are components of their own, shown on sub-pages.

use adw::prelude::*;
use relm4::prelude::*;

use crate::config::AppTheme;
use crate::i18n::{i18n, i18n_f, i18n_noop, ni18n_f};
use std::rc::Rc;

mod choices;
mod composing;
mod contacts;
mod dialog;
mod grammar;
mod handlers;
mod init_layout;
mod init_message_rows;
mod init_rows;
mod messages;
mod pages;
mod rows;
mod spelling;

pub(crate) use choices::language_choices;
use choices::{APP_THEMES, FETCH_INTERVALS, KEEP_DOWNLOADS_DAYS};
pub(crate) use messages::{PrefInit, PrefInput, PrefOutput};
use pages::{BackupPage, ComposingPage, KeptRows, PrivacyPage};
use rows::{compose_font_setup, middle_ellipsize};
use spelling::{rebuild_personal_words, SpellLangRows};

// ---- Preferences window -----------------------------------------------------

pub(crate) struct Preferences {
    /// The large-attachment policy and the rest of the Files hand-off
    /// preferences, kept whole to hand the app back on a change.
    files: crate::config::FilesPrefs,
    /// The large-attachment row, for the app to move when a hand-off
    /// dialog's "always do this" changes the policy while the dialog is up.
    files_large_row: Option<adw::ComboRow>,
    /// The dialog and its pages and sub-pages, set once assembled.
    dialog: Option<adw::PreferencesDialog>,
    dialog_pages: Option<Rc<dialog::DialogPages>>,
    /// The accounts panel's slot, for the fresh panel each open brings.
    accounts_slot: Option<adw::Bin>,
    /// The accounts component, told which of its pages to show.
    accounts_sender: relm4::Sender<crate::ui::accounts::AccountsInput>,
    /// The accounts sub-page's header; hidden while an editor of the panel
    /// is open, whose own header takes over.
    host_header: Option<adw::HeaderBar>,
    /// The OpenPGP page (#133), kept alive with the dialog.
    pgp_keys: Option<Controller<crate::ui::pgp_keys::PgpKeys>>,
    /// (name, address) per enabled account and alias, the rows of the
    /// "Send new messages from" combo after its first (#157).
    identities: Vec<(String, String)>,
    /// The Cloud Storage page (#144), likewise.
    cloud: Option<Controller<crate::ui::cloud_accounts::CloudAccounts>>,
    /// The address books page (#408), likewise.
    contacts: Option<Controller<crate::ui::contact_sources::ContactSources>>,
    /// A page about one address book is pushed over the books' list.
    contacts_open: bool,
    /// An editor is up on a sub-page: leaving it asks about unsaved changes.
    editor_open: bool,
    /// Which sub-page's editor is up: "accounts", "cloud" or "contacts".
    editor_page: &'static str,
    /// Composing → Advanced → Languages, kept to follow a change made from
    /// a composer's right-click menu (#365).
    spell_langs: Option<SpellLangRows>,
}

/// The `sender` the view's handlers report through. Setting a row's initial
/// value fires the same signal a change by hand does: a combo row's model
/// going in moves its selection to the first entry, and the saved value then
/// moves it again. Built hidden a moment after startup (`AppMsg::PrewarmSettings`),
/// the window so reported every choice twice, the first time wrong: the tray
/// item was taken down and published again for each, which Cinnamon's status
/// applet answered by crashing (#275). Nothing is reported until `ready` is
/// set, at the end of `init`, once the saved values are in.
#[derive(Clone)]
struct ViewSender {
    inner: ComponentSender<Preferences>,
    ready: Rc<std::cell::Cell<bool>>,
}

impl ViewSender {
    fn input(&self, msg: PrefInput) {
        if self.ready.get() {
            self.inner.input(msg);
        } else {
            tracing::debug!("settings window: {msg:?} while the rows are being set, not reported");
        }
    }

    fn output(&self, msg: PrefOutput) -> Result<(), PrefOutput> {
        self.inner.output(msg)
    }
}

#[relm4::component(pub(crate))]
impl Component for Preferences {
    type Init = PrefInit;
    type Input = PrefInput;
    type Output = PrefOutput;
    type CommandOutput = ();

    // The rows, built by `pages` before the view and set in the dialog's
    // pages by `assemble_dialog`.
    additional_fields! {
        kept: KeptRows,
        composing: ComposingPage,
        privacy: PrivacyPage,
        backup: BackupPage,
    }

    view! {
        // docs/DESIGN.md, "Preferences": the dialog, its search on, its
        // five pages added in `assemble_dialog`. Closing it keeps it for the
        // next open.
        adw::PreferencesDialog {
            set_title: &i18n("Preferences"),
            set_search_enabled: true,
            set_content_width: 760,
            set_content_height: 720,
            connect_closed[sender] => move |_| {
                let _ = sender.output(PrefOutput::Closed);
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let t_init = std::time::Instant::now();
        let mut model = Preferences {
            files: init.files,
            files_large_row: None,
            dialog: None,
            dialog_pages: None,
            accounts_slot: None,
            accounts_sender: init.accounts_sender.clone(),
            host_header: None,
            pgp_keys: None,
            identities: init.identities.clone(),
            cloud: None,
            contacts: None,
            contacts_open: false,
            editor_open: false,
            editor_page: "accounts",
            spell_langs: None,
        };

        // The rows report through `ViewSender`, silent until they hold
        // their saved values.
        let component_sender = sender;
        let sender = ViewSender {
            inner: component_sender.clone(),
            ready: Rc::new(std::cell::Cell::new(false)),
        };
        let kept = KeptRows::build(&init, &sender);
        let composing = ComposingPage::build(&sender);
        let privacy = PrivacyPage::build(&init, &sender);
        let backup = BackupPage::build(&sender);
        let widgets = view_output!();
        let ready = sender.ready.clone();
        let sender = component_sender;
        model.files_large_row = Some(widgets.kept.files_large_row.clone());

        model.init_privacy_rows(&widgets, &init);
        model.init_composing_rows(&widgets, &init, &sender);
        let slots = model.mount_hosted_pages(&init, &sender);
        model.assemble_dialog(&widgets, &init, slots, &root, &sender);
        model.init_first_page(&init, &sender, &root);

        tracing::debug!("preferences: built in {:?}", t_init.elapsed());
        // From here on a row's signal is the user's doing.
        ready.set(true);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            PrefInput::ChangeKeepDownloads(idx) => self.on_change_keep_downloads(idx, sender),
            PrefInput::ChangeFilesLarge(idx) => self.on_change_files_large(idx, sender),
            PrefInput::SetFilesPrefs(p) => self.on_set_files_prefs(p),
            PrefInput::SetSpellcheckLangs(langs) => self.on_set_spellcheck_langs(langs),
            PrefInput::ToggleSwipeEnabled(on) => {
                let _ = sender.output(PrefOutput::SetSwipeEnabled(on));
            }
            PrefInput::ChangeComposeDefaultFrom(index) => {
                self.on_change_compose_default_from(index, sender)
            }
            PrefInput::ChangeFetchInterval(index) => self.on_change_fetch_interval(index, sender),
            PrefInput::ToggleNotifications(on) => {
                let _ = sender.output(PrefOutput::SetNotifications(on));
            }
            PrefInput::ChangeReadMark(idx) => self.on_change_read_mark(idx, sender),
            PrefInput::SetContactBooks(books) => self.on_set_contact_books(books),
            PrefInput::SetLocalContactCount(n) => self.on_set_local_contact_count(n),
            PrefInput::ExportSettings => {
                let _ = sender.output(PrefOutput::ExportSettings);
            }
            PrefInput::ImportSettings => {
                let _ = sender.output(PrefOutput::ImportSettings);
            }
            PrefInput::ChangeAppTheme(index) => self.on_change_app_theme(index, sender),
            PrefInput::ChangeRemoteContent(index) => {
                // The two stored switches the choice stands for (screen 24).
                let (auto, warn) = choices::remote_content_at(index);
                let _ = sender.output(PrefOutput::SetAutoRemoteContent(auto));
                let _ = sender.output(PrefOutput::SetShowRemoteBanner(warn));
            }
            PrefInput::ShowAccounts(accounts) => {
                self.show_by_id(if accounts { "accounts" } else { "messages" }, &sender);
            }
            PrefInput::SetAccountsPanel {
                panel,
                sender: accounts,
            } => self.on_set_accounts_panel(panel, accounts),
            PrefInput::ShowPageById(id) => self.show_by_id(&id, &sender),
            PrefInput::EditorOpen(page) => self.on_editor_open(page),
            PrefInput::CloudEditorOpen(open) => self.on_cloud_editor_open(open),
            PrefInput::Contacts(out) => self.on_contacts(out, sender),
            PrefInput::ChangeComposeFormat(idx) => self.on_change_compose_format(idx, sender),
            PrefInput::ChangeUndoSend(idx) => self.on_change_undo_send(idx, sender),
            PrefInput::ChangeSignaturePosition(idx) => {
                self.on_change_signature_position(idx, sender)
            }
        }
    }
}
