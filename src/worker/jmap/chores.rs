//! Housekeeping: body search hits for filters (#191), auto-empty (#140)
//! and list clean-up (#407).

use std::collections::{HashMap, HashSet};

use crate::cache::Cache;
use crate::config::AccountConfig;
use crate::models::Message;
use crate::worker::chores::{
    auto_empty_due, auto_empty_roles, graph_before_date, role_folder_path,
};
use crate::worker::ids::hash_uid;
use crate::worker::jmap::folders::jmap_refresh_unread;
use crate::worker::jmap::index::JmapState;
use crate::worker::jmap::messages::jmap_query_ids;
use crate::worker::jmap::ops::{jmap_all_ids_in, jmap_destroy_all};
use crate::worker::jmap::wire::JmapSession;
use crate::worker::{blocking, WorkerEvent};

/// Body-search hits (#191) for the inbox listing: one `Email/query` with a
/// `body` filter per needle.
pub(super) async fn emit_jmap_body_hits(
    s: &JmapSession,
    account: &AccountConfig,
    folder_id: u32,
    path: &str,
    messages: &[Message],
    state: &JmapState,
    emit: &impl Fn(WorkerEvent),
) {
    if messages.is_empty() || state.inbox.as_ref().map(|(_, p)| p.as_str()) != Some(path) {
        return;
    }
    let needles = crate::config::filter_body_needles(&account.email);
    if needles.is_empty() {
        return;
    }
    let Some((_, mailbox_id)) = state.folders.get(path).cloned() else {
        return;
    };
    let listed: HashSet<u32> = messages.iter().map(|m| m.uid).collect();
    let mut hits: HashMap<u32, Vec<String>> = HashMap::new();
    for needle in needles {
        let sess = s.clone();
        let mid = mailbox_id.clone();
        let n = needle.clone();
        let found = blocking(move || {
            jmap_query_ids(
                &sess,
                serde_json::json!({ "inMailbox": mid, "body": n }),
                250,
            )
        })
        .await;
        match found {
            Ok(ids) => {
                for uid in ids
                    .iter()
                    .map(|id| hash_uid(id))
                    .filter(|u| listed.contains(u))
                {
                    hits.entry(uid).or_default().push(needle.clone());
                }
            }
            Err(e) => tracing::warn!("filter: JMAP body search for {needle:?} failed: {e}"),
        }
    }
    tracing::info!(
        "filter: JMAP body search over {} messages hit {}",
        listed.len(),
        hits.len()
    );
    emit(WorkerEvent::BodyHits { folder_id, hits });
}

/// Auto-empty (#140): destroy whatever in Junk / Trash is older than the
/// account's chosen age.
pub(super) async fn auto_empty_jmap(
    s: &JmapSession,
    account_id: u32,
    account: &AccountConfig,
    cache: Option<&Cache>,
    state: &mut JmapState,
    last: &mut Option<std::time::Instant>,
    emit: &impl Fn(WorkerEvent),
) {
    if !auto_empty_due(account, last) {
        return;
    }
    let folders = cache
        .map(|c| c.load_folders(account_id))
        .unwrap_or_default();
    let mut purged_any = false;
    for (kind, role, days) in auto_empty_roles(account) {
        let Some(path) = role_folder_path(account, &folders, kind, role) else {
            continue;
        };
        let Some((_, mailbox_id)) = state.folders.get(&path).cloned() else {
            continue;
        };
        let before = graph_before_date(days);
        let sess = s.clone();
        let ids = match blocking(move || {
            let ids = jmap_all_ids_in(&sess, &mailbox_id, serde_json::json!({ "before": before }))?;
            jmap_destroy_all(&sess, &ids)?;
            Ok(ids)
        })
        .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::warn!("auto-empty: could not empty {path} on account {account_id}: {e}");
                continue;
            }
        };
        for id in &ids {
            let uid = hash_uid(id);
            state.uids.remove(&uid);
            if let Some(c) = cache {
                c.delete_message(account_id, &path, uid);
            }
        }
        tracing::info!(
            "auto-empty: deleted {} message(s) older than {days} days from {path} on account {account_id}",
            ids.len()
        );
        purged_any |= !ids.is_empty();
    }
    if purged_any {
        jmap_refresh_unread(s, account_id, cache, state, emit).await;
    }
}
