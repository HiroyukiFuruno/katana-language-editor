use super::ProhibitedColorLiteralRule;
use crate::diagnostics::Violation;
use crate::workspace::WorkspaceModel;
use std::fs;
use std::io;
use std::path::PathBuf;

const SOURCE: &str = "#![allow(kle_lint::prohibited_color_literal)]\nconst COLOR: &str = \"#fff\";";

#[test]
fn path_names_and_file_allow_do_not_exempt_production() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let paths = [
        "crates/katana-language-editor/src/lib.rs",
        "crates/katana-language-editor-egui/src/tests/nested.rs",
        "crates/katana-language-editor-floem/src/kdv-presets/nested.rs",
    ];
    for path in paths {
        fixture.write(path)?;
    }
    let workspace = WorkspaceModel::load(&fixture.root)?;
    assert_eq!(workspace.rust_files().len(), 3);
    let violations = ProhibitedColorLiteralRule::check(&workspace)?;
    assert_eq!(violations.len(), 3);
    for path in paths {
        assert!(violations.iter().any(|violation| {
            violation.file == fixture.root.join(path)
                && violation.rule == "prohibited-color-literal"
                && violation.line == 2
                && has_color_evidence(violation)
        }));
    }
    Ok(())
}

fn has_color_evidence(violation: &Violation) -> bool {
    violation.literal.as_deref() == Some("\"#fff\"")
        && violation
            .hint
            .as_deref()
            .is_some_and(|hint| hint.contains("KUC"))
}

#[test]
fn scanned_non_library_sources_remain_outside_color_rule() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    fixture.write("crates/kle-linter/src/example.rs")?;
    fixture.write("tools/kle-storybook/src/example.rs")?;
    let workspace = WorkspaceModel::load(&fixture.root)?;
    assert_eq!(workspace.rust_files().len(), 2);
    assert!(ProhibitedColorLiteralRule::check(&workspace)?.is_empty());
    Ok(())
}

struct Fixture {
    owned: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> io::Result<Self> {
        let owned = Self::reserve()?;
        let root = owned.join("tests/kdv-presets/workspace");
        Ok(Self { owned, root })
    }

    fn reserve() -> io::Result<PathBuf> {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |value| value.as_nanos());
        for attempt in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "kle-color-scope-{}-{stamp}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(path),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "fixture path collision",
        ))
    }

    fn write(&self, relative: &str) -> io::Result<()> {
        let path = self.root.join(relative);
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "fixture parent missing"))?;
        fs::create_dir_all(parent)?;
        fs::write(path, SOURCE)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.owned);
    }
}
