//! What the composer is told, and what it tells its host.

use super::types::{Field, OwnMail, SigPlace};
use crate::config::ComposeFormat;
use crate::contacts::Suggestion;
use crate::models::DraftOrigin;
use crate::ui::drop_zones::DropChoice;
use crate::worker::OutgoingMessage;

#[derive(Debug)]
pub(crate) enum ComposeInput {
    Send,
    /// The OpenPGP send check answered, for these inputs.
    PgpChecked {
        checked: Vec<String>,
        result: Result<(), String>,
    },
    /// The pane crossed its width breakpoint: fold the toolbar into the ⋯
    /// menu, or spread it out again.
    SetNarrow(bool),
    /// The folded toolbar's ⋯ button: the actions as a menu.
    OverflowMenu,
    /// Files picked; ask which account and how, then upload.
    CloudPicked(Vec<std::path::PathBuf>),
    /// Files let go on one of the drop surfaces.
    DroppedFiles(DropChoice, Vec<std::path::PathBuf>),
    /// Upload these files to `account`, whose `expire_days` and `password`
    /// carry the user's choices for this upload; `link_password` is a
    /// password of their own for every link, else one is generated per
    /// file.
    CloudUpload {
        paths: Vec<std::path::PathBuf>,
        account: crate::services::cloud::CloudAccount,
        link_password: Option<String>,
    },
    /// [`ComposeInput::CloudUpload`] with the account's sign-in read from the
    /// keyring (`None`: not there).
    CloudUploadSignedIn {
        paths: Vec<std::path::PathBuf>,
        account: crate::services::cloud::CloudAccount,
        link_password: Option<String>,
        password: Option<String>,
    },
    /// One upload finished (in a thread): the link, or why not.
    CloudUploaded {
        name: String,
        result: Result<crate::services::cloud::ShareResult, String>,
    },
    /// Take a link back out of the body.
    RemoveCloudLink(usize),
    CopyCloudPasswords,
    /// Move the draft being edited to Trash and close without saving.
    DeleteDraft,
    /// The OpenPGP Sign toggle (#133).
    ToggleSign(bool),
    /// Pick what this message is written in.
    SetFormat(ComposeFormat),
    /// Rich text to plain text would take formatting, pictures or links
    /// out of the body: ask first (#424).
    AskPlain,
    /// Put the message in this format, now that nothing stands in the way.
    ApplyFormat(ComposeFormat),
    /// The Translate button (#327): read the quoted message's language
    /// first, so the menu can offer it.
    TranslateMenu,
    /// The quoted message's text came back: show the languages.
    ShowTranslateMenu(String),
    /// The From button was pressed: the Send from menu.
    FromMenu,
    /// An identity was picked in the Send from menu, by index.
    PickFrom(usize),
    /// Show Original or Show Translation: the translation was the last
    /// edit, so it is undone or done again, the subject with it (#327).
    TranslateToggle {
        original: bool,
    },
    /// Translate what was written (or the selection) into this language.
    TranslateTo(String),
    /// The editor handed over what is to be translated.
    TranslateBegun {
        found: String,
        target: String,
    },
    /// The translation came back, or failed.
    Translated {
        result: Result<Vec<String>, String>,
        subject: Option<String>,
    },
    /// The body came back for a format change; put it in the new one.
    LoadAs {
        to: ComposeFormat,
        body: String,
    },
    /// The OpenPGP Encrypt toggle; encrypting turns signing on too.
    ToggleEncrypt(bool),
    /// The Send menu's read receipt check (#520).
    ToggleReceipt(bool),
    /// The editor's HTML + plain text came back asynchronously — finish sending.
    SendBody(OutgoingFields),
    /// Save the current message to Drafts.
    SaveDraft,
    /// The From account's signature, put in by hand (#385): at the caret,
    /// or before or after the text from the button's right-click menu.
    InsertSignature(SigPlace),
    /// The Insert Signature button was right-clicked.
    SignatureMenu,
    /// The Insert Signature button was clicked: one signature goes in at
    /// the caret, several are listed (#445).
    SignatureClicked,
    /// One of the From account's signatures (#445), by index, in place of
    /// the one in the message; `None` takes it out.
    UseSignature(Option<usize>),
    /// The editor content came back — finish saving the draft.
    SaveDraftBody {
        fields: OutgoingFields,
    },
    /// The automatic save's tick (#340), then, once anything was edited,
    /// the fields and body read for it.
    AutoSave,
    AutoSaveRead,
    AutoSaveBody(OutgoingFields),
    /// An automatic save came back: whether it went through, and where the
    /// copy is now, to be replaced by the next save.
    Autosaved {
        saved: bool,
        origin: Option<DraftOrigin>,
    },
    /// Cancel, Escape or the window's close button: closes at once when
    /// nothing was written, else asks first (#290).
    Cancel,
    /// Ask whether to save the edited message to Drafts or discard it.
    ConfirmDiscard,
    /// The question was answered with Keep Editing.
    KeepEditing,
    /// Close without saving.
    Discard,
    /// Put keyboard focus where writing starts, once the composer is on
    /// screen (opened, or moved between inline and window): see
    /// [`Compose::focus_initial`].
    FocusInitial,
    /// A recipient/subject field changed — mark dirty.
    MarkFieldsDirty,
    AttachFiles,
    /// The writer's own text, read for the forgotten-attachment check.
    AttachCheck(String),
    /// "Send Anyway" in the forgotten-attachment dialog.
    SendWithoutAttachment,
    /// Show a recipient row, with the field rows if a compact reply has
    /// them folded away, and put the cursor in it (#491).
    FocusField(Field),
    AddAttachments(Vec<std::path::PathBuf>),
    RemoveAttachment(usize),
    /// A large attachment's "Send a Link": upload it to the cloud, and
    /// once the link is in the message, take the file off.
    SendAsLink(usize),
    /// The given recipient field changed — refresh autocomplete.
    Suggest(Field),
    /// The accounts whose organization directory is searched (#410), and
    /// the addresses in the mail of those that suggest only their own.
    DirectoriesKnown {
        graph: Vec<crate::config::AccountConfig>,
        own_mail: OwnMail,
    },
    /// Contacts and mail history loaded off the UI thread.
    SuggestionsLoaded(Vec<Suggestion>),
    /// The typing paused: ask the directories, unless it went on since.
    DirectoryLookup {
        field: Field,
        token: String,
        generation: u64,
    },
    /// What the directories answered, and the failures not shown before.
    DirectoryResults {
        field: Field,
        token: String,
        generation: u64,
        matches: Vec<Suggestion>,
        failed: Vec<crate::contacts::directory::DirectoryFailure>,
    },
    /// Addresses just sent to from another composer: into this one's
    /// suggestions at once, without waiting for a reopen.
    AddSuggestions {
        account_id: u32,
        new: Vec<Suggestion>,
    },
    /// Arrow-key move of the autocomplete highlight (+1 down, -1 up).
    CompletionMove(i32),
    /// Accept the highlighted suggestion into the active field.
    CompletionAccept,
    /// Dismiss the autocomplete popover.
    CompletionClose,
    /// Ctrl+Z / Ctrl+Shift+Z (#200): step the composer's history back or
    /// forward — the body's text and the attachments, in one order.
    History {
        redo: bool,
    },
    /// Zoom the body one step in (1) or out (-1), or back to 100% (0).
    Zoom(i8),
    /// The body's text history changed: WebKit gained or lost a step. The
    /// flag is whether it now has anything to take back.
    BodyHistory(bool),
    /// Showcase only (TEGAMI_SHOWCASE_COMPOSE_UNDO): one step of the scripted
    /// history check. See [`Compose::showcase_history`].
    ShowcaseHistory(u8),
    /// Showcase only (TEGAMI_SHOWCASE_COMPOSE_FROM): pick the From row's
    /// entry `n`, as a click would, so the signature swap can be captured.
    ShowcaseFrom(u32),
}

/// A message as it stood when Send, Save or the automatic save read it:
/// the body from the editor, the rest from the fields.
#[derive(Debug, Clone)]
pub(crate) struct OutgoingFields {
    pub(super) html: String,
    pub(super) text: String,
    pub(super) to: String,
    pub(super) cc: String,
    pub(super) bcc: String,
    pub(super) reply_to: String,
    pub(super) subject: String,
    pub(super) from_account_id: u32,
    pub(super) from_alias: Option<String>,
}

#[derive(Debug)]
pub(crate) enum ComposeOutput {
    /// Send the message.
    Send {
        message: Box<OutgoingMessage>,
    },
    /// Save the message to the Drafts folder (no send).
    SaveDraft(Box<OutgoingMessage>),
    /// Save the message to the Templates folder (#360), replacing the
    /// template at its `draft_origin` if it has one.
    SaveTemplate(Box<OutgoingMessage>),
    /// Delete the draft this composer was opened from, and close it. The app
    /// moves the draft to Trash (undoable, like deleting it from the list).
    DeleteDraft {
        id: u32,
        origin: DraftOrigin,
    },
    /// This pane is done (cancelled / sent / draft-saved / superseded). Carries
    /// the id so the app tears down the right host.
    Close(u32),
    /// Save the message to Drafts on the composer's own (#340), leaving it
    /// open.
    AutoSave {
        id: u32,
        message: Box<OutgoingMessage>,
    },
    /// Discarded, a new message that only automatic saves put in Drafts:
    /// take that copy out, and close.
    DiscardAutosave {
        id: u32,
        origin: DraftOrigin,
    },
    /// The spelling languages were changed from the body's right-click
    /// menu (#365): the whole new setting, saved app-wide like the
    /// Settings switches.
    /// Manage Accounts… in the Send from menu: the app's account settings.
    ManageAccounts,
    SetSpellcheckLangs(String),
    /// Check spelling was switched in More actions: the app-wide setting,
    /// as Settings → Composing has it.
    SetSpellcheck(bool),
    /// A directory could not be asked (#307): for the status bar, once per
    /// composer.
    DirectoryError(String),
}
