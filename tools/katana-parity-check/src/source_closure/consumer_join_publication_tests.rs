use super::ConsumerJoinPublication;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn exclusive_rename_preserves_an_existing_empty_directory() -> Result<(), String> {
    let root = fixture("exclusive-race")?;
    let source = root.join("source");
    let destination = root.join("destination");
    fs::create_dir(&source).map_err(|error| error.to_string())?;
    fs::create_dir(&destination).map_err(|error| error.to_string())?;
    fs::write(source.join("payload"), b"original").map_err(|error| error.to_string())?;
    assert!(renamore::rename_exclusive(&source, &destination).is_err());
    assert_eq!(
        fs::read(source.join("payload")).map_err(|error| error.to_string())?,
        b"original"
    );
    assert_eq!(
        fs::read_dir(&destination)
            .map_err(|error| error.to_string())?
            .count(),
        0
    );
    cleanup(&root)
}

fn fixture(name: &str) -> Result<std::path::PathBuf, String> {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let root = std::env::temp_dir().join(format!("consumer-join-{name}-{suffix}"));
    fs::create_dir(&root).map_err(|error| error.to_string())?;
    Ok(root)
}

fn cleanup(root: &Path) -> Result<(), String> {
    fs::remove_dir_all(root).map_err(|error| error.to_string())
}

#[test]
fn existing_destination_is_preserved_and_build_is_not_called() -> Result<(), String> {
    let root = fixture("existing")?;
    let destination = root.join("destination");
    fs::write(&destination, b"existing").map_err(|error| error.to_string())?;
    let called = std::cell::Cell::new(false);
    let result = ConsumerJoinPublication::publish(&destination, |_| {
        called.set(true);
        Ok(())
    });
    assert!(result.is_err());
    assert!(!called.get());
    assert_eq!(
        fs::read(&destination).map_err(|error| error.to_string())?,
        b"existing"
    );
    cleanup(&root)
}

#[cfg(unix)]
#[test]
fn existing_destination_symlink_is_preserved_and_build_is_not_called() -> Result<(), String> {
    use std::os::unix::fs::symlink;

    let root = fixture("symlink")?;
    let target = root.join("target");
    let destination = root.join("destination");
    fs::write(&target, b"target").map_err(|error| error.to_string())?;
    symlink(&target, &destination).map_err(|error| error.to_string())?;
    let called = std::cell::Cell::new(false);
    let result = ConsumerJoinPublication::publish(&destination, |_| {
        called.set(true);
        Ok(())
    });
    assert!(result.is_err());
    assert!(!called.get());
    assert_eq!(
        fs::read_link(&destination).map_err(|error| error.to_string())?,
        target
    );
    cleanup(&root)
}

#[test]
fn successful_build_is_published_as_one_directory_rename() -> Result<(), String> {
    let root = fixture("success")?;
    let destination = root.join("destination");
    let result = ConsumerJoinPublication::publish(&destination, |temporary| {
        fs::write(temporary.join("part-a"), b"a").map_err(|error| error.to_string())?;
        fs::write(temporary.join("part-b"), b"b").map_err(|error| error.to_string())
    });
    assert!(result.is_ok());
    assert_eq!(
        fs::read(destination.join("part-a")).map_err(|error| error.to_string())?,
        b"a"
    );
    assert_eq!(
        fs::read(destination.join("part-b")).map_err(|error| error.to_string())?,
        b"b"
    );
    assert_eq!(
        fs::read_dir(&root)
            .map_err(|error| error.to_string())?
            .count(),
        1
    );
    cleanup(&root)
}

#[test]
fn build_error_leaves_no_partial_destination_or_temporary() -> Result<(), String> {
    let root = fixture("build-error")?;
    let destination = root.join("destination");
    let result = ConsumerJoinPublication::publish(&destination, |temporary| {
        fs::write(temporary.join("partial"), b"partial").map_err(|error| error.to_string())?;
        Err("build failed".into())
    });
    assert!(result.is_err());
    assert!(!destination.exists());
    assert_eq!(
        fs::read_dir(&root)
            .map_err(|error| error.to_string())?
            .count(),
        0
    );
    cleanup(&root)
}

#[test]
fn destination_created_during_build_is_preserved_and_temp_is_cleaned() -> Result<(), String> {
    let root = fixture("race")?;
    let destination = root.join("destination");
    let result = ConsumerJoinPublication::publish(&destination, |temporary| {
        fs::write(&destination, b"external").map_err(|error| error.to_string())?;
        fs::write(temporary.join("partial"), b"partial").map_err(|error| error.to_string())
    });
    assert!(result.is_err());
    assert_eq!(
        fs::read(&destination).map_err(|error| error.to_string())?,
        b"external"
    );
    assert_eq!(
        fs::read_dir(&root)
            .map_err(|error| error.to_string())?
            .count(),
        1
    );
    cleanup(&root)
}

#[test]
fn rename_error_reports_cleanup_error_without_publishing() -> Result<(), String> {
    let root = fixture("rename-error")?;
    let destination = root.join("destination");
    let result = ConsumerJoinPublication::publish(&destination, |temporary| {
        fs::remove_dir_all(temporary).map_err(|error| error.to_string())
    });
    let error = match result {
        Ok(()) => return Err("rename unexpectedly succeeded".into()),
        Err(error) => error,
    };
    assert!(error.contains("failed to publish consumer join"));
    assert!(error.contains("failed to clean publication temporary"));
    assert!(!destination.exists());
    assert_eq!(
        fs::read_dir(&root)
            .map_err(|error| error.to_string())?
            .count(),
        0
    );
    cleanup(&root)
}
