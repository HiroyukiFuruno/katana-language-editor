use super::{require_clean_fixed_source, NativePhysicalRunLayout, FIXED_KATANA_REVISION};
use crate::FixedSourceHarnessBuilder;
use crate::physical_bootstrap_types::RequestValidationError;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn fixed_source() -> PathBuf {
    FixedSourceHarnessBuilder::required_katana_repo().expect("KATANA_REPO must be explicitly set")
}

struct TempRoot(PathBuf);
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> (TempRoot, PathBuf) {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("katana-host-layout-test-{id}"));
    let kle = root.join("kle");
    fs::create_dir_all(kle.join("target/source-closure/artifacts")).expect("artifacts");
    fs::create_dir_all(kle.join("target/source-closure/profiles/macos-latest")).expect("profiles");
    fs::write(
        kle.join("target/source-closure/artifacts/source-derived-native-target.json"),
        "{}",
    )
    .expect("target record");
    fs::write(
        kle.join("target/source-closure/artifacts/context-menu-target-manifest.json"),
        "{}",
    )
    .expect("context menu manifest");
    fs::write(
        kle.join("target/source-closure/profiles/macos-latest/probe.json"),
        "{}",
    )
    .expect("profile");
    (TempRoot(root), kle)
}

fn git_repo() -> (TempRoot, PathBuf, String) {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let parent = std::env::temp_dir().join(format!("katana-fixed-source-test-{id}"));
    let root = parent.join("repo");
    fs::create_dir_all(&root).expect("repo");
    run_git(&root, &["init", "--quiet"]);
    run_git(&root, &["config", "user.email", "test@example.invalid"]);
    run_git(&root, &["config", "user.name", "fixed-source-test"]);
    fs::write(root.join("Cargo.toml"), "[workspace]").expect("manifest");
    run_git(&root, &["add", "Cargo.toml"]);
    run_git(&root, &["commit", "--quiet", "-m", "fixture"]);
    let revision = git_stdout(&root, &["rev-parse", "HEAD"]);
    (TempRoot(parent), root, revision)
}

fn run_git(root: &Path, args: &[&str]) {
    let output = crate::system::ProcessService::create_command("git")
        .args(["-C", root.to_str().expect("utf8")])
        .args(args)
        .output()
        .expect("git");
    assert!(output.status.success(), "git failed: {output:?}");
}

fn git_stdout(root: &Path, args: &[&str]) -> String {
    String::from_utf8(
        crate::system::ProcessService::create_command("git")
            .args(["-C", root.to_str().expect("utf8")])
            .args(args)
            .output()
            .expect("git")
            .stdout,
    )
    .expect("utf8")
    .trim()
    .to_owned()
}

#[test]
fn canonical_layout_reads_standard_inputs_and_creates_internal_fixture() {
    let (_root, kle) = fixture();
    let source = fixed_source();
    let layout = NativePhysicalRunLayout::from_kle_repo_root(&kle, &source).expect("layout");
    assert_eq!(layout.fixed_source, fs::canonicalize(source).expect("source"));
    assert!(layout
        .target_record
        .ends_with("source-derived-native-target.json"));
    assert!(layout.source_closure_profile.ends_with("probe.json"));
    assert!(layout.workspace_fixture.join("fixture.md").is_file());
    assert!(layout.sandbox.is_dir());
    assert!(layout.workspace_basename().starts_with("workspace-"));
}

#[test]
fn consecutive_layouts_use_distinct_non_secret_workspace_basenames() {
    let (_root, kle) = fixture();
    let source = fixed_source();
    let first = NativePhysicalRunLayout::from_kle_repo_root(&kle, &source).expect("first");
    let second = NativePhysicalRunLayout::from_kle_repo_root(&kle, &source).expect("second");
    assert_ne!(first.workspace_basename(), second.workspace_basename());
    assert!(!first.workspace_basename().contains('/'));
    assert!(!second.workspace_basename().contains('/'));
}

#[test]
fn missing_standard_artifact_fails_closed() {
    let (_root, kle) = fixture();
    fs::remove_file(kle.join("target/source-closure/artifacts/source-derived-native-target.json"))
        .expect("remove record");
    assert!(matches!(
        NativePhysicalRunLayout::from_kle_repo_root(&kle, fixed_source()),
        Err(RequestValidationError::PathNotFound("target_record"))
    ));
}

#[test]
fn missing_context_menu_manifest_fails_closed() {
    let (_root, kle) = fixture();
    fs::remove_file(kle.join(
        "target/source-closure/artifacts/context-menu-target-manifest.json",
    ))
    .expect("remove manifest");
    assert!(matches!(
        NativePhysicalRunLayout::from_kle_repo_root(&kle, fixed_source()),
        Err(RequestValidationError::PathNotFound("context_menu_manifest"))
    ));
}

#[test]
fn non_directory_fixed_source_fails_closed() {
    let (_root, kle) = fixture();
    let invalid = kle.parent().expect("parent").join("source-file");
    fs::write(&invalid, "not a directory").expect("file");
    assert!(matches!(
        NativePhysicalRunLayout::from_kle_repo_root(&kle, &invalid),
        Err(RequestValidationError::NotDirectory("fixed_katana_root"))
    ));
}

#[test]
fn accepts_clean_checkout_at_exact_expected_revision() {
    let (_temp, root, revision) = git_repo();
    assert_eq!(
        require_clean_fixed_source(&root, &revision).expect("clean repo"),
        fs::canonicalize(&root).expect("canonical repo")
    );
}

#[test]
fn rejects_checkout_at_wrong_revision() {
    let (_temp, root, _revision) = git_repo();
    assert!(matches!(
        require_clean_fixed_source(&root, FIXED_KATANA_REVISION),
        Err(RequestValidationError::Io("fixed_katana_revision"))
    ));
}

#[test]
fn rejects_dirty_checkout_before_any_fixture_is_created() {
    let (_temp, root, revision) = git_repo();
    fs::write(root.join("untracked.txt"), "dirty").expect("dirty marker");
    assert!(matches!(
        require_clean_fixed_source(&root, &revision),
        Err(RequestValidationError::Io("fixed_katana_worktree"))
    ));
}

#[test]
fn explicit_source_is_used_even_when_sibling_katana_is_invalid() {
    let (_temp, root, revision) = git_repo();
    let sibling = root.parent().expect("parent").join("katana");
    fs::write(&sibling, "must not be consulted").expect("sibling");
    assert_eq!(
        require_clean_fixed_source(&root, &revision).expect("explicit source"),
        fs::canonicalize(&root).expect("canonical repo")
    );
    let _ = fs::remove_file(sibling);
}
