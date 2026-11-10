//! The threads that fetch remote content: a few of the app's own, so a slow
//! server never holds up GIO's thread pool, which the file dialogs and the
//! disk reads share. A URL is fetched once however many wait for it.

use std::collections::VecDeque;
use std::io::Read;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use futures::channel::oneshot::Sender;

use super::store::{self, Entry};

const THREADS: usize = 6;
/// Larger than any picture worth showing in mail; a response past it is
/// treated as a failed fetch rather than held in memory.
const MAX_BYTES: u64 = 50 * 1024 * 1024;

type Reply = Sender<Option<Arc<Entry>>>;

struct Job {
    url: String,
    waiting: Vec<Reply>,
}

#[derive(Default)]
struct Queue {
    now: VecDeque<Job>,
    /// Fetches under way: their URL and who waits for them.
    running: Vec<(String, Vec<Reply>)>,
    threads: usize,
}

fn pool() -> &'static (Mutex<Queue>, Condvar) {
    static POOL: OnceLock<(Mutex<Queue>, Condvar)> = OnceLock::new();
    POOL.get_or_init(Default::default)
}

static USER_AGENT: OnceLock<String> = OnceLock::new();

/// The reader's own user agent, so a server answers these fetches as it
/// answered WebKit's.
pub(super) fn set_user_agent(ua: String) {
    let _ = USER_AGENT.set(ua);
}

/// Queue `url` for the reader, or join the fetch already queued or under
/// way.
pub(super) fn submit(url: String, reply: Option<Reply>) {
    let (lock, cv) = pool();
    let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, waiting)) = q.running.iter_mut().find(|(u, _)| *u == url) {
        waiting.extend(reply);
        return;
    }
    if let Some(job) = q.now.iter_mut().find(|j| j.url == url) {
        job.waiting.extend(reply);
        return;
    }
    q.now.push_back(Job {
        url,
        waiting: reply.into_iter().collect(),
    });
    if q.threads < THREADS {
        q.threads += 1;
        let spawned = std::thread::Builder::new()
            .name("remote-content".into())
            .spawn(work);
        if spawned.is_err() {
            q.threads -= 1;
        }
    }
    cv.notify_one();
}

fn work() {
    let (lock, cv) = pool();
    loop {
        let job = {
            let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
            let job = loop {
                if let Some(j) = q.now.pop_front() {
                    break j;
                }
                q = cv.wait(q).unwrap_or_else(|e| e.into_inner());
            };
            q.running.push((job.url.clone(), job.waiting));
            job.url
        };

        let entry = store::read(&job).or_else(|| {
            let fetched = get(&job)?;
            store::write(&job, &fetched);
            Some(fetched)
        });

        let waiting = {
            let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
            q.running
                .iter()
                .position(|(u, _)| *u == job)
                .map(|at| q.running.swap_remove(at).1)
                .unwrap_or_default()
        };
        let entry = entry.map(Arc::new);
        for reply in waiting {
            let _ = reply.send(entry.clone());
        }
    }
}

fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
    })
}

/// One GET, with no cookies and no Referer. Blocking.
fn get(url: &str) -> Option<Entry> {
    let mut request = agent().get(url).set("Accept", "*/*");
    if let Some(ua) = USER_AGENT.get().filter(|ua| !ua.is_empty()) {
        request = request.set("User-Agent", ua);
    }
    let response = match request.call() {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("remote content fetch failed: {e}");
            return None;
        }
    };
    let content_type = response.header("Content-Type").unwrap_or("").to_string();
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= MAX_BYTES).then_some(Entry {
        content_type,
        bytes,
    })
}
