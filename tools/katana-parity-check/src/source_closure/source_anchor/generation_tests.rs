use std::{fs, path::Path, time::SystemTime};

use super::git_blob_at;
use crate::source_closure::operational_process::git_output;

#[test]
fn reads_source_bytes_from_a_real_temporary_git_blob() -> Result<(), String> {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let repo = std::env::temp_dir().join(format!(
        "kle-source-anchor-git-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&repo).map_err(|error| error.to_string())?;
    let result = (|| {
        git_output(&repo, &["init", "--quiet"])?;
        git_output(&repo, &["config", "user.name", "Source Anchor Test"])?;
        git_output(
            &repo,
            &["config", "user.email", "source-anchor@example.invalid"],
        )?;
        git_output(&repo, &["config", "core.autocrlf", "false"])?;
        let relative = Path::new("crates/katana-ui/src/fixture.rs");
        let file = repo.join(relative);
        fs::create_dir_all(file.parent().ok_or("fixture path has no parent")?)
            .map_err(|error| error.to_string())?;
        let original = "fn fixture() { let text = \"日本語✈️\"; }\r\n".as_bytes();
        fs::write(&file, original).map_err(|error| error.to_string())?;
        git_output(&repo, &["add", "--", "crates/katana-ui/src/fixture.rs"])?;
        git_output(&repo, &["commit", "--quiet", "-m", "fixture"])?;
        let revision = String::from_utf8(git_output(&repo, &["rev-parse", "HEAD"])?)
            .map_err(|error| error.to_string())?;
        let actual = git_blob_at(&repo, revision.trim(), "crates/katana-ui/src/fixture.rs")?;
        if actual != original {
            return Err("Git blob read changed original UTF-8/CRLF bytes".into());
        }
        Ok(())
    })();
    fs::remove_dir_all(&repo)
        .map_err(|error| format!("failed to remove test repository: {error}"))?;
    result
}
