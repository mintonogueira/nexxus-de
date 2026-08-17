//! Live filesystem monitoring and generation-safe snapshot publication.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use thiserror::Error;

use crate::{ApplicationIndexConfig, IndexDelta, IndexSnapshot, ScanError, scan};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationIndexEvent {
    Changed(IndexDelta),
    WatchError(String),
    RescanError(String),
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("initial XDG scan failed: {0}")]
    InitialScan(#[from] ScanError),
    #[error("filesystem watcher initialization failed: {0}")]
    Watcher(#[from] notify::Error),
    #[error("worker thread could not be started: {0}")]
    Thread(#[from] std::io::Error),
    #[error("no XDG application root or safe parent can be watched")]
    NoWatchableRoot,
}

pub struct ApplicationIndexService {
    config: ApplicationIndexConfig,
    snapshot: Arc<RwLock<IndexSnapshot>>,
    subscribers: Arc<Mutex<Vec<mpsc::Sender<ApplicationIndexEvent>>>>,
    watcher: Arc<Mutex<RecommendedWatcher>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl ApplicationIndexService {
    /// Performs an authoritative initial scan and starts the filesystem event worker.
    pub fn start(config: ApplicationIndexConfig) -> Result<Self, ServiceError> {
        let initial = scan(&config)?;
        let snapshot = Arc::new(RwLock::new(initial));
        let subscribers = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (event_tx, event_rx) = mpsc::channel();
        let watcher = Arc::new(Mutex::new(notify::recommended_watcher(event_tx)?));
        let mut watched = BTreeMap::new();
        refresh_watch_paths(&config, &watcher, &mut watched)?;
        if watched.is_empty() {
            return Err(ServiceError::NoWatchableRoot);
        }

        let worker_config = config.clone();
        let worker_snapshot = Arc::clone(&snapshot);
        let worker_subscribers = Arc::clone(&subscribers);
        let worker_watcher = Arc::clone(&watcher);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("nexxus-xdg-index".to_owned())
            .spawn(move || {
                worker_loop(
                    worker_config,
                    worker_snapshot,
                    worker_subscribers,
                    worker_watcher,
                    worker_stop,
                    event_rx,
                    watched,
                )
            })?;

        Ok(Self {
            config,
            snapshot,
            subscribers,
            watcher,
            stop,
            worker: Some(worker),
        })
    }

    /// Returns a clone of the latest immutable generation for lock-free consumer use.
    pub fn snapshot(&self) -> IndexSnapshot {
        read_snapshot(&self.snapshot).clone()
    }

    /// Adds a consumer event stream. Dead receivers are pruned during broadcast.
    pub fn subscribe(&self) -> mpsc::Receiver<ApplicationIndexEvent> {
        let (sender, receiver) = mpsc::channel();
        lock_mutex(&self.subscribers).push(sender);
        receiver
    }

    pub fn config(&self) -> &ApplicationIndexConfig {
        &self.config
    }

    pub fn watcher_kind(&self) -> notify::WatcherKind {
        let _guard = lock_mutex(&self.watcher);
        <RecommendedWatcher as Watcher>::kind()
    }
}

impl Drop for ApplicationIndexService {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn worker_loop(
    config: ApplicationIndexConfig,
    snapshot: Arc<RwLock<IndexSnapshot>>,
    subscribers: Arc<Mutex<Vec<mpsc::Sender<ApplicationIndexEvent>>>>,
    watcher: Arc<Mutex<RecommendedWatcher>>,
    stop: Arc<AtomicBool>,
    event_rx: mpsc::Receiver<notify::Result<notify::Event>>,
    mut watched: BTreeMap<PathBuf, RecursiveMode>,
) {
    while !stop.load(Ordering::Acquire) {
        let first = match event_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        if let Err(error) = first {
            broadcast(
                &subscribers,
                ApplicationIndexEvent::WatchError(error.to_string()),
            );
            continue;
        }

        // Package installs commonly emit bursts of create/write/rename events.
        // A short debounce collapses them into one deterministic authoritative rescan.
        thread::sleep(Duration::from_millis(120));
        while event_rx.try_recv().is_ok() {}

        if let Err(error) = refresh_watch_paths(&config, &watcher, &mut watched) {
            broadcast(
                &subscribers,
                ApplicationIndexEvent::WatchError(error.to_string()),
            );
        }

        let scanned = match scan(&config) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                broadcast(
                    &subscribers,
                    ApplicationIndexEvent::RescanError(error.to_string()),
                );
                continue;
            }
        };

        let previous = read_snapshot(&snapshot).clone();
        let comparable = scanned.clone().with_generation(previous.generation);
        if comparable == previous {
            continue;
        }
        let next = scanned.with_generation(previous.generation.saturating_add(1));
        let delta = IndexDelta::between(&previous, &next);
        *write_snapshot(&snapshot) = next;
        broadcast(&subscribers, ApplicationIndexEvent::Changed(delta));
    }
}

/// Reconciles watcher paths as optional XDG/Flatpak directories appear or disappear.
fn refresh_watch_paths(
    config: &ApplicationIndexConfig,
    watcher: &Arc<Mutex<RecommendedWatcher>>,
    watched: &mut BTreeMap<PathBuf, RecursiveMode>,
) -> notify::Result<()> {
    let mut desired = BTreeMap::new();
    for root in &config.roots {
        if root.path.is_dir() {
            desired.insert(root.path.clone(), RecursiveMode::Recursive);
        } else if let Some(parent) = nearest_existing_parent(&root.path) {
            desired
                .entry(parent)
                .or_insert(RecursiveMode::NonRecursive);
        }
    }

    let mut watcher = lock_mutex(watcher);
    for path in watched
        .keys()
        .filter(|path| !desired.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>()
    {
        let _ = watcher.unwatch(&path);
        watched.remove(&path);
    }
    for (path, mode) in &desired {
        if watched.get(path) == Some(mode) {
            continue;
        }
        if watched.contains_key(path) {
            let _ = watcher.unwatch(path);
        }
        watcher.watch(path, *mode)?;
        watched.insert(path.clone(), *mode);
    }
    Ok(())
}

/// Finds the closest existing directory while refusing `/` itself as a watch
/// target; watching the filesystem root would be unnecessarily broad and noisy.
fn nearest_existing_parent(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(candidate) = current {
        candidate.parent()?;
        if candidate.is_dir() {
            return Some(candidate.to_path_buf());
        }
        current = candidate.parent();
    }
    None
}

fn broadcast(
    subscribers: &Arc<Mutex<Vec<mpsc::Sender<ApplicationIndexEvent>>>>,
    event: ApplicationIndexEvent,
) {
    lock_mutex(subscribers).retain(|subscriber| subscriber.send(event.clone()).is_ok());
}

fn lock_mutex<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn read_snapshot(lock: &RwLock<IndexSnapshot>) -> std::sync::RwLockReadGuard<'_, IndexSnapshot> {
    lock.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn write_snapshot(lock: &RwLock<IndexSnapshot>) -> std::sync::RwLockWriteGuard<'_, IndexSnapshot> {
    lock.write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
