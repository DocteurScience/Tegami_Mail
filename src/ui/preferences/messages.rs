//! The settings window's messages: what it starts with, what the app and
//! its own rows send it, and the preference changes it reports.

use super::*;

/// Initial data for the preferences dialog: the kept settings' values.
#[derive(Debug)]
pub(crate) struct PrefInit {
    pub(crate) auto_remote_content: bool,
    pub(crate) show_remote_banner: bool,
    /// Answering requests for read receipts, and asking for them (#520).
    pub(crate) receipts: crate::config::ReceiptPolicy,
    pub(crate) request_receipts: bool,
    pub(crate) returned_receipts: crate::config::ReturnedReceipts,
    pub(crate) gravatar: bool,
    pub(crate) sender_logos: bool,
    pub(crate) fetch_interval_secs: u64,
    pub(crate) push: bool,
    /// Reading pane shows conversations newest-message-first.
    pub(crate) thread_newest_first: bool,
    /// Deleting a whole selected conversation asks for confirmation.
    pub(crate) confirm_thread_delete: bool,
    /// Message rows take a sideways swipe to archive / delete (#92).
    pub(crate) swipe_enabled: bool,
    /// What handed-in files open into, and the large-attachment policy.
    pub(crate) files: crate::config::FilesPrefs,
    /// The identity new messages are sent from (#157); empty = the open
    /// folder's account. One of `identities`' addresses.
    pub(crate) compose_default_from: String,
    pub(crate) paste_plain: bool,
    pub(crate) return_paragraph: bool,
    pub(crate) toolbar_expanded: bool,
    /// The font rich-text messages are written in, as a CSS family list,
    /// and its size in points; empty and 0 are the defaults.
    pub(crate) compose_font: String,
    pub(crate) compose_font_size: u32,
    /// Composing → Default zoom; 0 = the reader's.
    pub(crate) compose_zoom: u32,
    pub(crate) spellcheck: bool,
    pub(crate) spellcheck_langs: String,
    /// New messages start as plain text (#180).
    pub(crate) compose_format: crate::config::ComposeFormat,
    /// Replies and forwards to plain-text mail are written in plain text (#436).
    pub(crate) follow_plain: bool,
    /// Where the signature sits in a reply or forward (#237).
    pub(crate) signature_position: crate::config::SignaturePosition,
    pub(crate) signature_dashes: bool,
    pub(crate) fold_quote: bool,
    /// The line above a reply's quote, "" for the default (#528).
    pub(crate) reply_header: String,
    /// Undo Send delay in seconds, 0 for off (#434).
    pub(crate) undo_send_secs: u32,
    pub(crate) app_theme: AppTheme,
    pub(crate) notifications: bool,
    pub(crate) notification_content: bool,
    /// A notification when a read receipt comes back (#520).
    pub(crate) notify_receipts: bool,
    pub(crate) read_mark: crate::config::ReadMark,
    pub(crate) contact_names: bool,
    pub(crate) single_key_shortcuts: bool,
    pub(crate) gmail_keys: bool,
    pub(crate) keep_downloads_days: u32,
    pub(crate) run_in_background: bool,
    pub(crate) autostart: bool,
    /// The accounts panel (built by the AccountsWindow component), shown on
    /// the Accounts page's sub-pages.
    pub(crate) accounts_panel: gtk::Widget,
    /// Open on the Accounts page.
    pub(crate) start_on_accounts: bool,
    /// The page to open on, when the app remembers one from earlier this
    /// session; overrides `start_on_accounts`.
    pub(crate) start_page: Option<String>,
    /// The accounts component's inbox, to show its pages.
    pub(crate) accounts_sender: relm4::Sender<crate::ui::accounts::AccountsInput>,
    /// (name, address) per account and alias, for the OpenPGP page's
    /// key generator and the "Send new messages from" row.
    pub(crate) identities: Vec<(String, String)>,
}

#[derive(Debug)]
pub(crate) enum PrefInput {
    /// The Contacts page: the books new contacts can go to, freshly read.
    SetContactBooks(Vec<crate::contacts::Book>),
    /// The Tegami Mail book's size changed (an import, a deletion).
    SetLocalContactCount(usize),
    /// The large-attachment policy row moved.
    ChangeFilesLarge(u32),
    /// The app changed the Files preferences (a dialog's "always do this").
    SetFilesPrefs(crate::config::FilesPrefs),
    /// The spelling languages changed, possibly from a composer (#365).
    SetSpellcheckLangs(String),
    ChangeKeepDownloads(u32),
    ToggleSwipeEnabled(bool),
    /// The "Send new messages from" combo: 0 = the open folder's account,
    /// then `identities` in order.
    ChangeComposeDefaultFrom(u32),
    ChangeFetchInterval(u32),
    ToggleNotifications(bool),
    ChangeReadMark(u32),
    ExportSettings,
    ImportSettings,
    ChangeComposeFormat(u32),
    ChangeSignaturePosition(u32),
    ChangeUndoSend(u32),
    ChangeAppTheme(u32),
    /// Privacy → Images and remote content: Never, Ask or Always.
    ChangeRemoteContent(u32),
    /// Show the Accounts page (true) or Messages (false).
    ShowAccounts(bool),
    /// A fresh accounts panel for this open (the dialog is kept between
    /// opens; the panel is rebuilt over the current accounts).
    SetAccountsPanel {
        panel: gtk::Widget,
        sender: relm4::Sender<crate::ui::accounts::AccountsInput>,
    },
    /// Show a page or a sub-page by id (`dialog::place_of`).
    ShowPageById(String),
    /// Show that page, then hand the accounts panel this input (an account
    /// page to open, a filter to add), so what it pushes stays up.
    ShowPageThen(String, crate::ui::accounts::AccountsInput),
    /// Push a service's sub-page over whatever is shown, without leaving
    /// it (an account's OpenPGP page opens the keys).
    PushPage(&'static str),
    /// How many tags, filters and smart folders there are (screen 26).
    SetCounts {
        tags: usize,
        filters: usize,
        smart: usize,
    },
    /// An accounts-panel editor subpage (account, filter or tag) opened on
    /// the named page, or closed — hide/show the sub-page's header so the
    /// editor's own header takes over.
    EditorOpen(Option<&'static str>),
    /// The Cloud Storage page's account editor is up (or gone).
    CloudEditorOpen(bool),
    /// What the Contacts page asks of the dialog and the app.
    Contacts(crate::ui::contact_sources::ContactSourcesOutput),
}

#[derive(Debug)]
pub(crate) enum PrefOutput {
    /// The Contacts page's buttons: the app opens the file dialogs and the
    /// confirmation, and does the work off the main thread.
    ImportLocalContacts,
    ExportLocalContacts,
    DeleteAllLocalContacts,
    SetContactBook(String),
    /// The Contacts page is showing: read the books new contacts can go to
    /// (D-Bus, so off the main thread) and send them back.
    LoadContactBooks,
    /// An address book was added, changed, removed or switched on or off
    /// for Tegami Mail (#408): sync the books and read the contacts again.
    ContactSourcesChanged,
    /// A category was shown, so the app can reopen the window on it.
    PageShown(String),
    SetAutoRemoteContent(bool),
    SetShowRemoteBanner(bool),
    SetReceipts(crate::config::ReceiptPolicy),
    SetRequestReceipts(bool),
    SetReturnedReceipts(crate::config::ReturnedReceipts),
    SetGravatar(bool),
    SetAvatars(bool),
    SetSenderLogos(bool),
    SetThreading(bool),
    SetThreadNewestFirst(bool),
    SetConfirmThreadDelete(bool),
    SetSwipeEnabled(bool),
    SetFilesPrefs(crate::config::FilesPrefs),
    SetComposeDefaultFrom(String),
    SetPastePlain(bool),
    SetReturnParagraph(bool),
    SetToolbarExpanded(bool),
    SetComposeFont(String),
    SetComposeFontSize(u32),
    SetComposeZoom(u32),
    SetSpellcheck(bool),
    SetSpellcheckLangs(String),
    SetFetchInterval(u64),
    SetPush(bool),
    SetNotifications(bool),
    SetNotificationContent(bool),
    SetNotifyReceipts(bool),
    /// Console mode and the status bar's place, which the main menu's
    /// Troubleshooting entry sets (docs/DESIGN.md, "Removed or moved").
    #[allow(dead_code)] // Constructed by the Troubleshooting menu, next.
    SetConsoleMode(bool),
    #[allow(dead_code)]
    SetStatusBarTop(bool),
    SetReadMark(crate::config::ReadMark),
    ExportSettings,
    ExportLog,
    ImportSettings,
    SetAppTheme(AppTheme),
    /// A theme was picked in the gallery (its id, or "system").
    SetTheme(String),
    SetPreviewLines(u32),
    /// Keep Contacts' search field open.
    /// Saved contacts go by their address-book names (#541).
    SetContactNames(bool),
    SetSingleKey(bool),
    SetGmailKeys(bool),
    /// How many days downloaded mail is kept; 0 keeps it (#394).
    SetKeepDownloads(u32),
    SetRunInBackground(bool),
    SetAutostart(bool),
    SetComposeFormat(crate::config::ComposeFormat),
    SetFollowPlain(bool),
    SetSignaturePosition(crate::config::SignaturePosition),
    SetSignatureDashes(bool),
    SetFoldQuote(bool),
    SetReplyHeader(String),
    SetUndoSend(u32),
    Closed,
}
