use super::{RULE, Visitor, is_allowed_owner};
use crate::diagnostics::Violation;
use crate::rules::RuleRunner;
use crate::workspace::WorkspaceModel;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

fn lint(source: &str, path: &str) -> Result<Vec<Violation>, syn::Error> {
    let file = syn::parse_file(source)?;
    let mut visitor = Visitor::new(PathBuf::from(path));
    visitor.visit_file(&file);
    Ok(visitor.violations)
}

#[test]
fn accepts_references_reexports_strings_and_comments() -> Result<(), syn::Error> {
    let source = r#"
        /* WHY: trait HostProjectionProvider {} */
        pub use crate::HostProjectionProvider;
        fn use_provider(value: HostProjectionProviderError) { let _ = value; }
        const TEXT: &str = "HostProjectionProvider HostProjectionProviderError";
    "#;
    assert!(lint(source, "fixture.rs")?.is_empty());
    Ok(())
}

#[test]
fn rejects_all_owned_definition_kinds() -> Result<(), syn::Error> {
    let source = r#"
        trait HostProjectionProvider {}
        enum HostProjectionProviderError { Missing }
        struct HostProjectionProviderError;
        type HostProjectionProvider = (); 
    "#;
    assert_eq!(lint(source, "fixture.rs")?.len(), 4);
    Ok(())
}

#[test]
fn rejects_private_nested_and_raw_ident_definitions() -> Result<(), syn::Error> {
    let source = r#"
        mod nested {
            trait r#HostProjectionProvider {}
            struct r#HostProjectionProviderError;
        }
        fn build() {
            struct HostProjectionProvider;
            type r#HostProjectionProviderError = ();
        }
    "#;
    assert_eq!(lint(source, "fixture.rs")?.len(), 4);
    Ok(())
}

#[test]
fn allows_only_the_exact_neutral_owner_path() {
    let root = Path::new("/workspace");
    assert!(is_allowed_owner(
        root,
        &root.join("crates/katana-language-editor/src/host_projection.rs")
    ));
    assert!(!is_allowed_owner(
        root,
        Path::new("/other/workspace/crates/katana-language-editor/src/host_projection.rs")
    ));
    assert!(!is_allowed_owner(
        root,
        &root.join("crates/katana-language-editor/src/host_projection_provider.rs")
    ));
}

#[test]
fn registered_rule_rejects_production_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = FixtureRoot::new()?;
    let root = &fixture.path;
    let (source_root, owner, test_fixture) = fixture.write_workspace()?;
    let workspace = WorkspaceModel::load(root)?;
    let violations = RuleRunner::check(&workspace)?;
    assert!(violations.iter().any(|violation| {
        violation.rule == RULE && violation.file == source_root.join("fixture.rs")
    }));
    assert!(
        violations
            .iter()
            .any(|violation| { violation.rule == RULE && violation.file == test_fixture })
    );
    assert!(
        !violations
            .iter()
            .any(|violation| violation.rule == RULE && violation.file == owner)
    );
    Ok(())
}

struct FixtureRoot {
    path: PathBuf,
}

impl FixtureRoot {
    fn write_workspace(&self) -> io::Result<(PathBuf, PathBuf, PathBuf)> {
        let source_root = self.path.join("crates/katana-language-editor/src");
        fs::create_dir_all(&source_root)?;
        self.write_manifests()?;
        self.write_sources(source_root)
    }

    fn write_manifests(&self) -> io::Result<()> {
        fs::write(self.path.join("Cargo.toml"), "[workspace]\nmembers = []\n")?;
        fs::write(
            self.path.join("crates/katana-language-editor/Cargo.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::create_dir_all(self.path.join("crates/katana-language-editor-egui/src"))?;
        fs::write(
            self.path
                .join("crates/katana-language-editor-egui/Cargo.toml"),
            "[package]\nname = \"fixture-egui\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nkatana-language-editor = { path = \"../katana-language-editor\" }\n",
        )?;
        fs::create_dir_all(self.path.join("crates/katana-language-editor-floem/src"))?;
        fs::write(
            self.path
                .join("crates/katana-language-editor-floem/Cargo.toml"),
            "[package]\nname = \"fixture-floem\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nkatana-language-editor = { path = \"../katana-language-editor\" }\nfloem = { workspace = true }\n",
        )?;
        Ok(())
    }

    fn write_sources(&self, source_root: PathBuf) -> io::Result<(PathBuf, PathBuf, PathBuf)> {
        fs::write(
            source_root.join("fixture.rs"),
            "mod nested { trait HostProjectionProvider {} }",
        )?;
        let owner = source_root.join("host_projection.rs");
        fs::write(
            &owner,
            "trait HostProjectionProvider {}\nenum HostProjectionProviderError { Missing }",
        )?;
        let test_fixture = self.path.join("crates/kle-linter/tests/fixture_tests.rs");
        let parent = test_fixture
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing fixture parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(&test_fixture, "struct HostProjectionProviderError;")?;
        Ok((source_root, owner, test_fixture))
    }

    fn new() -> io::Result<Self> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        for attempt in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "kle-host-projection-{}-{timestamp}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate an exclusive fixture directory",
        ))
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
