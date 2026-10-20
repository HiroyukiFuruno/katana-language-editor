use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
#[path = "consumer_join_publication_tests.rs"]
mod consumer_join_publication_tests;

pub(super) struct ConsumerJoinPublication;

impl ConsumerJoinPublication {
    pub(super) fn publish(
        destination: &Path,
        build: impl FnOnce(&Path) -> Result<(), String>,
    ) -> Result<(), String> {
        reject_existing_destination(destination)?;
        let parent = destination
            .parent()
            .ok_or_else(|| "publication destination has no parent directory".to_string())?;
        let parent = validate_parent(parent)?;
        let destination = parent.join(
            destination
                .file_name()
                .ok_or_else(|| "publication destination has no file name".to_string())?,
        );
        reject_existing_destination(&destination)?;
        let temporary = reserve_temporary(&parent)?;

        let result = build(&temporary).and_then(|()| {
            reject_existing_destination(&destination)?;
            renamore::rename_exclusive(&temporary, &destination)
                .map_err(|error| format!("failed to publish consumer join: {error}"))
        });
        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(with_cleanup_error(error, &temporary)),
        }
    }
}

fn reject_existing_destination(destination: &Path) -> Result<(), String> {
    match fs::symlink_metadata(destination) {
        Ok(_) => Err(format!(
            "publication destination already exists: {}",
            destination.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to inspect publication destination {}: {error}",
            destination.display()
        )),
    }
}

fn validate_parent(parent: &Path) -> Result<PathBuf, String> {
    let metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("failed to inspect publication parent: {error}"))?;
    if !metadata.is_dir() {
        return Err("publication parent is not a directory".into());
    }
    parent
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize publication parent: {error}"))
}

fn reserve_temporary(parent: &Path) -> Result<PathBuf, String> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to create temporary name: {error}"))?
        .as_nanos();
    for attempt in 0..32u64 {
        let suffix = format!(
            ".consumer-join-{seed}-{}-{attempt}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let temporary = parent.join(suffix);
        match fs::create_dir(&temporary) {
            Ok(()) => return Ok(temporary),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("failed to reserve publication temporary: {error}")),
        }
    }
    Err("failed to reserve a unique publication temporary".into())
}

fn with_cleanup_error(error: String, temporary: &Path) -> String {
    match fs::remove_dir_all(temporary) {
        Ok(()) => error,
        Err(cleanup) => format!("{error}; failed to clean publication temporary: {cleanup}"),
    }
}
