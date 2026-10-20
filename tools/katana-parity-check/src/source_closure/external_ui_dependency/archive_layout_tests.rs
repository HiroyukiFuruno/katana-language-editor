use std::fs;

use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;

use super::{LockPackage, resolve_package_archive};

#[test]
fn archive_selection_requires_matching_layout_version_and_regular_file()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = FixtureBuilder::root()?;
    let registry = fixture.path().join("registry");
    let root = registry.join("src/registry-id/egui-0.36.1");
    let archive = registry.join("cache/registry-id/egui-0.36.1.crate");
    fs::create_dir_all(&root)?;
    fs::create_dir_all(registry.join("cache/registry-id"))?;
    let lock = LockPackage {
        version: "0.36.1".into(),
        ..LockPackage::default()
    };
    let mut unresolved = Vec::new();
    assert!(resolve_package_archive(&root, &lock, &mut unresolved).is_none());
    assert!(
        unresolved
            .iter()
            .any(|reason| reason.contains("unavailable"))
    );
    fs::write(&archive, b"archive selection does not authenticate bytes")?;
    unresolved.clear();
    assert_eq!(
        resolve_package_archive(&root, &lock, &mut unresolved),
        Some(archive)
    );
    assert!(unresolved.is_empty());
    for (path, reason) in [
        (
            registry.join("src/registry-id/egui-0.36.2"),
            "fixed package version",
        ),
        (
            registry.join("vendor/registry-id/egui-0.36.1"),
            "unknown registry layout",
        ),
        (
            registry.join("src/../egui-0.36.1"),
            "unknown registry layout",
        ),
    ] {
        unresolved.clear();
        assert!(resolve_package_archive(&path, &lock, &mut unresolved).is_none());
        assert!(
            unresolved.iter().any(|item| item.contains(reason)),
            "{unresolved:?}"
        );
    }
    let other_root = registry.join("src/other-registry/egui-0.36.1");
    fs::create_dir_all(registry.join("cache/other-registry/egui-0.36.1.crate"))?;
    unresolved.clear();
    assert!(resolve_package_archive(&other_root, &lock, &mut unresolved).is_none());
    assert!(
        unresolved
            .iter()
            .any(|reason| reason.contains("not a regular file"))
    );
    Ok(())
}
