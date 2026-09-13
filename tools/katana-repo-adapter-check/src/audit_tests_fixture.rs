use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{config::KatanaRepoAdapterCheckConfig, system::ProcessService};

pub(crate) struct RepoFixture {
    root: PathBuf,
    workspace_root: PathBuf,
}

impl RepoFixture {
    pub(crate) fn create(name: &str) -> Result<Self, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let workspace_root = std::env::temp_dir().join(format!(
            "katana-repo-adapter-check-{name}-{}-{nanos}",
            std::process::id()
        ));
        Ok(Self {
            root: workspace_root.join("katana"),
            workspace_root,
        })
    }

    pub(crate) fn config(&self) -> KatanaRepoAdapterCheckConfig {
        KatanaRepoAdapterCheckConfig {
            repo_root: self.root.clone(),
        }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn write_complete(
        &self,
        dependencies: bool,
        module: bool,
        adapter: bool,
    ) -> Result<(), String> {
        self.write(
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/katana-ui\"]\nresolver = \"2\"\n",
        )?;
        self.write("crates/katana-ui/Cargo.toml", manifest(dependencies))?;
        self.write("crates/katana-ui/src/lib.rs", "")?;
        self.write_kle_crates()?;
        self.write(
            "crates/katana-ui/tests/ui_integration_parallel.rs",
            wrapper(module),
        )?;
        if adapter {
            self.write(
                "crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs",
                adapter_source(),
            )?;
        }
        self.write("platforms/linux/ci/compose.yml", compose_fixture())?;
        self.write("platforms/windows/ci/compose.yml", compose_fixture())?;
        for workflow in [
            "test-and-build.yml",
            "release-readiness.yml",
            "build-and-release.yml",
        ] {
            self.write(
                &format!(".github/workflows/{workflow}"),
                workflow_clone_fixture(),
            )?;
        }
        self.generate_lockfile()
    }

    pub(crate) fn write_adapter_with_forbidden_marker(&self) -> Result<(), String> {
        self.write(
            "crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs",
            &format!("{}\n// force_view_mode\n", adapter_source()),
        )
    }

    pub(crate) fn write_adapter_that_mutates_source(&self) -> Result<(), String> {
        let source = adapter_source().replace(
            "assert!(!EVIDENCE.is_empty());",
            "match std::fs::write(\"unexpected-source-change\", \"mutation\") { Ok(()) => assert!(!EVIDENCE.is_empty()), Err(_) => {} }",
        );
        self.write(
            "crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs",
            &source,
        )
    }

    pub(crate) fn write(&self, relative: &str, contents: &str) -> Result<(), String> {
        let path = self.root.join(relative);
        fs::create_dir_all(
            path.parent()
                .ok_or_else(|| "fixture file has no parent".to_string())?,
        )
        .map_err(|error| error.to_string())?;
        fs::write(path, contents).map_err(|error| error.to_string())
    }

    fn write_kle_crates(&self) -> Result<(), String> {
        let kle_root = self.workspace_root.join("katana-language-editor");
        write_sibling_crate(&kle_root, "katana-language-editor")?;
        write_sibling_crate(&kle_root, "katana-language-editor-egui")
    }

    fn generate_lockfile(&self) -> Result<(), String> {
        let output = ProcessService::create_command("cargo")
            .current_dir(&self.root)
            .arg("generate-lockfile")
            .output()
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            return Ok(());
        }
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

impl Drop for RepoFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workspace_root);
    }
}

fn write_sibling_crate(root: &Path, name: &str) -> Result<(), String> {
    let crate_root = root.join("crates").join(name);
    fs::create_dir_all(crate_root.join("src")).map_err(|error| error.to_string())?;
    fs::write(
        crate_root.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )
    .map_err(|error| error.to_string())?;
    fs::write(crate_root.join("src/lib.rs"), "").map_err(|error| error.to_string())
}

fn manifest(dependencies: bool) -> &'static str {
    if dependencies {
        return "[package]\nname = \"katana-ui\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dev-dependencies]\nkatana-language-editor = { path = \"../../../katana-language-editor/crates/katana-language-editor\" }\nkatana-language-editor-egui = { path = \"../../../katana-language-editor/crates/katana-language-editor-egui\" }\n";
    }
    "[package]\nname = \"katana-ui\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
}

fn wrapper(module: bool) -> &'static str {
    if module {
        return "mod editor_ui { #[test] fn test_integration_editor_line_numbers_visibility() {} }\n#[path = \"integration/editor/kle_downstream_adapter.rs\"]\nmod editor_kle_downstream_adapter;\n";
    }
    "mod editor_ui {}\n"
}

fn adapter_source() -> &'static str {
    "const EVIDENCE: &str = \"KatanaKleEditorAdapter::drain_editor EguiLanguageEditor::new(editor_config()) EditorAction::SaveDocument EditorAction::SetViewMode(EditorViewMode::Split) EditorAction::SetSplitDirection(EditorSplitDirection::Vertical) EditorScrollSource::Preview .with_target_line(7) ScrollSource::Neither last_scroll_to_line active_document_buffer(&harness) KatanA document should become dirty after KLE content event std::fs::read_to_string(&second_path) KatanA save action should mark the document clean\";\n#[test]\nfn kle_event_action_stream_updates_real_katana_editor_state() { assert!(!EVIDENCE.is_empty()); }\n"
}

fn compose_fixture() -> &'static str {
    "services:\n  test:\n    volumes:\n      - ../../../../katana-language-editor:/katana-language-editor:ro\n"
}

fn workflow_clone_fixture() -> &'static str {
    "name: workflow\non: [push, pull_request]\njobs:\n  build:\n    steps:\n      - name: Prepare sibling katana-language-editor checkout\n        env:\n          KLE_REF: ${{ vars.KLE_REF }}\n        shell: bash\n        run: |\n          KLE_REF=\"${KLE_REF:-release/v0.1.0}\"\n          if [ ! -d ../katana-language-editor/.git ]; then\n            git clone --depth 1 --branch \"$KLE_REF\" https://github.com/HiroyukiFuruno/katana-language-editor ../katana-language-editor\n          fi\n"
}
