//! The dialog's structure at build time: the hosted pages, the chip
//! editors, and the first page shown.

use super::*;

/// The components shown on sub-pages, each in a slot of its own.
pub(super) struct HostedSlots {
    pub(super) pgp: adw::Bin,
    pub(super) cloud: adw::Bin,
    pub(super) translation: adw::Bin,
    pub(super) contacts: adw::Bin,
}

impl Preferences {
    /// The pages that are components of their own: OpenPGP, Cloud Storage,
    /// Translation and Contacts (Accounts comes in `PrefInit`).
    pub(super) fn mount_hosted_pages(
        &mut self,
        init: &PrefInit,
        sender: &ComponentSender<Self>,
    ) -> HostedSlots {
        let slots = HostedSlots {
            pgp: adw::Bin::new(),
            cloud: adw::Bin::new(),
            translation: adw::Bin::new(),
            contacts: adw::Bin::new(),
        };
        let pgp = crate::ui::pgp_keys::PgpKeys::builder()
            .launch(crate::ui::pgp_keys::PgpKeysInit {
                identities: init.identities.clone(),
            })
            .detach();
        slots.pgp.set_child(Some(pgp.widget()));
        self.pgp_keys = Some(pgp);
        let cloud = crate::ui::cloud_accounts::CloudAccounts::builder()
            .launch(())
            .forward(sender.input_sender(), |o| match o {
                crate::ui::cloud_accounts::CloudAccountsOutput::EditorOpen(open) => {
                    PrefInput::CloudEditorOpen(open)
                }
            });
        slots.cloud.set_child(Some(cloud.widget()));
        slots
            .translation
            .set_child(Some(&crate::ui::translation_page::build()));
        self.cloud = Some(cloud);
        let contacts = crate::ui::contact_sources::ContactSources::builder()
            .launch(crate::ui::contact_sources::ContactSourcesInit)
            .forward(sender.input_sender(), PrefInput::Contacts);
        slots.contacts.set_child(Some(contacts.widget()));
        self.contacts = Some(contacts);
        slots
    }

    /// The close key and the first page shown.
    pub(super) fn init_first_page(
        &mut self,
        init: &PrefInit,
        sender: &ComponentSender<Self>,
        root: &adw::PreferencesDialog,
    ) {
        root.add_controller(crate::ui::close_key());
        let first = init.start_page.clone().unwrap_or_else(|| {
            if init.start_on_accounts {
                "accounts".to_string()
            } else {
                "messages".to_string()
            }
        });
        self.show_by_id(&first, sender);
    }
}
