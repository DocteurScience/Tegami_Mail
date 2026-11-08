//! Preferences → Privacy (screen 24): three settings where there were
//! seven switches. Remote content is one choice — never, ask, always —
//! that stands for the former "Always load remote content" and "Warn when
//! remote content is blocked"; sender photos and logos are one switch for
//! Gravatar and website logos. Pre-loading remote content ahead and the
//! addressing warning keep their stored values without a row (owner,
//! 2026-11-08: the page merges into three).

use adw::prelude::*;

use super::super::choices::{photos_on, receipts_at, remote_content_index};
use super::super::rows::no_truncate;
use super::super::{PrefInit, PrefInput, PrefOutput, ViewSender};
use crate::i18n::i18n;

/// The page, and the rows on it that the dialog reaches later.
#[derive(Debug)]
pub(in crate::ui::preferences) struct PrivacyPage {
    pub(in crate::ui::preferences) page: adw::PreferencesPage,
}

impl PrivacyPage {
    pub(in crate::ui::preferences) fn build(init: &PrefInit, sender: &ViewSender) -> Self {
        let page = adw::PreferencesPage::new();

        let remote = adw::PreferencesGroup::new();
        remote.set_title(&i18n("Remote content"));
        let content = adw::ComboRow::new();
        content.set_title(&i18n("Images and remote content"));
        content.set_subtitle(&i18n(
            "Remote content can tell the sender when and from where you read.",
        ));
        content.set_model(Some(&gtk::StringList::new(&[
            &i18n("Never"),
            &i18n("Ask"),
            &i18n("Always"),
        ])));
        no_truncate(&content);
        content.set_selected(remote_content_index(
            init.auto_remote_content,
            init.show_remote_banner,
        ));
        let s = sender.clone();
        content.connect_selected_notify(move |r| {
            s.input(PrefInput::ChangeRemoteContent(r.selected()));
        });
        remote.add(&content);

        let photos = adw::SwitchRow::new();
        photos.set_title(&i18n("Sender photos and logos"));
        photos.set_subtitle(&i18n(
            "Gravatar and website logos: the sender’s address is sent to these services.",
        ));
        photos.set_active(photos_on(init.gravatar, init.sender_logos));
        let s = sender.clone();
        photos.connect_active_notify(move |r| {
            let _ = s.output(PrefOutput::SetGravatar(r.is_active()));
            let _ = s.output(PrefOutput::SetSenderLogos(r.is_active()));
        });
        remote.add(&photos);
        page.add(&remote);

        let receipts = adw::PreferencesGroup::new();
        receipts.set_title(&i18n("Read receipts"));
        let answer = adw::ComboRow::new();
        answer.set_title(&i18n("Answer read-receipt requests"));
        answer.set_subtitle(&i18n("A receipt tells the sender you opened the message."));
        answer.set_model(Some(&gtk::StringList::new(&[
            &i18n("Ask"),
            &i18n("Always"),
            &i18n("Never"),
        ])));
        no_truncate(&answer);
        answer.set_selected(super::super::choices::receipts_index(init.receipts));
        let s = sender.clone();
        answer.connect_selected_notify(move |r| {
            let _ = s.output(PrefOutput::SetReceipts(receipts_at(r.selected())));
        });
        receipts.add(&answer);
        page.add(&receipts);

        Self { page }
    }
}
