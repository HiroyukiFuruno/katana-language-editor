use std::fs;
use std::path::{Path, PathBuf};

use super::resolve_rust_name_path_candidates;
use crate::capability_manifest::capability_manifest_test_fixtures::FixtureBuilder;

fn write_source(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&path, "fn fixture() {}").map_err(|error| error.to_string())?;
    Ok(path)
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

fn canonical(path: PathBuf) -> Result<PathBuf, String> {
    path.canonicalize().map_err(|error| error.to_string())
}

#[test]
fn external_and_missing_paths_do_not_resolve_to_current_or_parent_module() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let current = write_source(fixture.path(), "src/about_info/mod.rs")?;
    let nested = write_source(fixture.path(), "src/outer/inner/mod.rs")?;
    write_source(fixture.path(), "src/outer/mod.rs")?;
    let crate_root = write_source(fixture.path(), "src/lib.rs")?;
    write_source(fixture.path(), "src/existing.rs")?;

    for (path, query) in [
        (&current, names(&["std", "env", "consts", "OS"])),
        (&current, names(&["unknown_crate", "item"])),
        (&current, names(&["Self", "Item"])),
        (&current, names(&["self", "missing"])),
        (&nested, names(&["super", "missing"])),
        (&crate_root, names(&["super", "existing"])),
    ] {
        assert!(
            resolve_rust_name_path_candidates(fixture.path(), path, &query).is_empty(),
            "query {:?} from {} must remain unresolved",
            query,
            path.display()
        );
    }
    Ok(())
}

#[test]
fn crate_self_and_super_paths_resolve_actual_child_modules() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let current = write_source(fixture.path(), "src/parent/mod.rs")?;
    let child = write_source(fixture.path(), "src/parent/child.rs")?;
    let nested = write_source(fixture.path(), "src/parent/child/nested.rs")?;
    let sibling = write_source(fixture.path(), "src/sibling.rs")?;

    assert_eq!(
        resolve_rust_name_path_candidates(
            fixture.path(),
            &current,
            &names(&["crate", "parent", "child"])
        ),
        vec![canonical(child)?]
    );
    assert_eq!(
        resolve_rust_name_path_candidates(fixture.path(), &current, &names(&["self", "child"])),
        vec![canonical(fixture.path().join("src/parent/child.rs"))?]
    );
    assert_eq!(
        resolve_rust_name_path_candidates(
            fixture.path(),
            &current,
            &names(&["crate", "parent", "child", "nested"])
        ),
        vec![canonical(nested)?]
    );
    assert_eq!(
        resolve_rust_name_path_candidates(fixture.path(), &current, &names(&["super", "sibling"])),
        vec![canonical(sibling)?]
    );
    Ok(())
}

#[test]
fn same_level_file_and_directory_module_candidates_remain_ambiguous() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let current = write_source(fixture.path(), "src/about_info/mod.rs")?;
    let file = write_source(fixture.path(), "src/about_info/child.rs")?;
    let module = write_source(fixture.path(), "src/about_info/child/mod.rs")?;

    assert_eq!(
        resolve_rust_name_path_candidates(fixture.path(), &current, &names(&["self", "child"])),
        vec![canonical(file)?, canonical(module)?]
    );
    Ok(())
}
