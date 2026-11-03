//! Seeding a cache with the demo mailbox, for the showcase and the
//! screenshots.

use crate::backend::{MailBackend, MockBackend};
use crate::models::Message;

/// Seed a cache with the demo's sample mail, so the offline demo exercises
/// the queries a real cache answers (search, smart folders, tags). Only
/// ever called with the in-memory cache demo mode opens, so none of this can
/// reach a real `cache.db`.
pub(crate) fn seed_demo_cache(cache: &crate::cache::Cache) {
    let mock = MockBackend::new();
    seed_sample_mail(cache, &mock);
}

/// Every sample folder and its messages.
fn seed_sample_mail(cache: &crate::cache::Cache, mock: &MockBackend) {
    for account in mock.accounts() {
        let folders = MailBackend::folders(mock, account.id);
        cache.save_folders(account.id, &folders);
        for folder in &folders {
            let messages: Vec<Message> = mock
                .messages(folder.id)
                .into_iter()
                .map(|mut m| {
                    m.folder_id = folder.id;
                    m
                })
                .collect();
            if !messages.is_empty() {
                cache.save_messages(account.id, &folder.path, &messages);
            }
        }
    }
}
