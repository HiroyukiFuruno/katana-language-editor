use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{read_regular, write_bytes};

fn directory() -> Result<PathBuf, String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "consumer-join-input-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&root).map_err(|error| error.to_string())?;
    Ok(root)
}

#[test]
fn copy_preserves_bytes_and_never_overwrites() -> Result<(), String> {
    let root = directory()?;
    let path = root.join("data.json");
    let bytes = b"{\r\n  \"unchanged\": true\r\n}\r\n";
    write_bytes(&path, bytes)?;
    assert_eq!(read_regular(&path)?, bytes);
    assert!(write_bytes(&path, b"replacement").is_err());
    assert_eq!(read_regular(&path)?, bytes);
    fs::remove_dir_all(&root).map_err(|error| error.to_string())
}

#[test]
fn input_rejects_directory_and_missing_file() -> Result<(), String> {
    let root = directory()?;
    assert!(read_regular(&root).is_err());
    assert!(read_regular(&root.join("missing")).is_err());
    fs::remove_dir_all(&root).map_err(|error| error.to_string())
}

#[cfg(unix)]
#[test]
fn input_rejects_regular_and_dangling_symlinks() -> Result<(), String> {
    let root = directory()?;
    let target = root.join("target");
    write_bytes(&target, b"original")?;
    let link = root.join("link");
    std::os::unix::fs::symlink(&target, &link).map_err(|error| error.to_string())?;
    assert!(read_regular(&link).is_err());
    let dangling = root.join("dangling");
    std::os::unix::fs::symlink(root.join("absent"), &dangling)
        .map_err(|error| error.to_string())?;
    assert!(read_regular(&dangling).is_err());
    assert_eq!(read_regular(&target)?, b"original");
    fs::remove_dir_all(&root).map_err(|error| error.to_string())
}

#[test]
fn join_cli_dispatch_requires_canonical_input() {
    let args = vec!["join-consumer-artifacts".to_string()];
    let result = crate::source_closure::operational_lifecycle::run_cli(&args);
    assert!(matches!(result, Err(error) if error.contains("missing required option --input")));
}

#[test]
fn join_cli_rejects_unverified_plan_option() {
    let args = vec![
        "join-consumer-artifacts".to_string(),
        "--plan".to_string(),
        "untrusted.json".to_string(),
    ];
    let result = crate::source_closure::operational_lifecycle::run_cli(&args);
    assert!(
        matches!(result, Err(error) if error.contains("unknown source-closure option: --plan"))
    );
}
