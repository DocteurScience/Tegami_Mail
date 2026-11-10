//! The wizard's messages: its pages' input, what it hands the app, and the
//! results of its connection tests and sign-ins.

use super::*;

#[derive(Debug)]
pub(crate) enum WelcomeInput {
    /// "Get started": on to the accounts.
    Start,
    /// "Email account…": the form.
    OpenForm,
    /// "Continue" on the accounts: the last page, or out when nothing was
    /// added.
    Continue,
    ProviderChanged,
    TestAndAdd,
    ImportGoa(usize),
    /// The window came back to the front: Online Accounts may hold more now.
    RescanGoa,
    OpenOnlineAccounts,
    Finish,
}

#[derive(Debug)]
pub(crate) enum WelcomeOutput {
    /// A manual account passed its connection test, or a custom OAuth one
    /// signed in: save + connect it.
    AddAccount(Box<AccountConfig>),
    /// A GNOME Online Accounts import was chosen.
    ImportGoa(Box<AccountConfig>),
}

#[derive(Debug)]
pub(crate) enum WelcomeCmd {
    Tested {
        account: Box<AccountConfig>,
        result: ConnTest,
        seq: u32,
    },
    /// The browser sign-in for a custom OAuth account ended: its refresh
    /// token, or why not.
    SignedIn {
        account: Box<AccountConfig>,
        /// The refresh token, and who signed in when the provider says.
        result: Result<(String, Option<(String, String)>), String>,
        seq: u32,
    },
}
