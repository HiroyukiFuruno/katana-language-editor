impl NativePhysicalRunLayout {
    pub fn from_compiled_repository(
        fixed_source: impl AsRef<Path>,
    ) -> Result<Self, RequestValidationError> {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let kle_repo_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .ok_or(RequestValidationError::PathNotFound("kle_repo_root"))?;
        Self::from_kle_repo_root(kle_repo_root, fixed_source)
    }

    fn from_kle_repo_root(
        kle_repo_root: impl AsRef<Path>,
        fixed_source: impl AsRef<Path>,
    ) -> Result<Self, RequestValidationError> {
        let kle_repo_root = require_directory(kle_repo_root.as_ref(), "kle_repo_root")?;
        let fixed_source =
            require_clean_fixed_source(fixed_source.as_ref(), FIXED_KATANA_REVISION)?;
        let source_closure = kle_repo_root.join(SOURCE_CLOSURE_DIR);
        let target_record = require_file(
            &source_closure.join(ARTIFACT_DIR).join(TARGET_RECORD),
            "target_record",
        )?;
        let context_menu_manifest = require_file(
            &source_closure.join(ARTIFACT_DIR).join(CONTEXT_MENU_MANIFEST),
            "context_menu_manifest",
        )?;
        let source_closure_profile = require_file(
            &source_closure.join(PROFILE_DIR).join(PROFILE),
            "source_closure_profile",
        )?;

        let id = NEXT_LAYOUT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "katana-host-e2e-physical-{}-{id}",
            std::process::id()
        ));
        std::fs::create_dir(&root).map_err(|_| RequestValidationError::FixtureCreationFailed)?;
        let workspace_fixture = root.join(format!("workspace-{}", id));
        let sandbox = root.join("sandbox");
        if std::fs::create_dir_all(&workspace_fixture).is_err()
            || std::fs::write(workspace_fixture.join("fixture.md"), "# physical fixture\n").is_err()
            || std::fs::create_dir_all(&sandbox).is_err()
        {
            let _ = std::fs::remove_dir_all(&root);
            return Err(RequestValidationError::FixtureCreationFailed);
        }

        Ok(Self {
            root,
            fixed_source,
            source_closure_profile,
            target_record,
            context_menu_manifest,
            workspace_fixture,
            sandbox,
        })
    }

    pub fn load_source_derived_locator(
        &self,
    ) -> Result<AxTargetLocator, SourceDerivedNativeTargetError> {
        SourceDerivedNativeTarget::load(
            &self.target_record,
            &self.fixed_source,
            &self.source_closure_profile,
        )
    }

    pub fn launch_request(&self) -> Result<LaunchRequest, RequestValidationError> {
        LaunchRequest::new(&self.fixed_source, &self.sandbox, &self.workspace_fixture)
    }

    pub fn workspace_basename(&self) -> &str {
        self.workspace_fixture
            .file_name()
            .and_then(|name| name.to_str())
            .expect("run layout workspace basename is valid UTF-8")
    }

    pub fn workspace_fixture(&self) -> &Path {
        &self.workspace_fixture
    }

    pub fn load_workspace_observation_contract(
        &self,
    ) -> Result<crate::NativeAxSourceContract, crate::ContextMenuManifestError> {
        let manifest = ContextMenuManifestLoader::load(
            &self.context_menu_manifest,
            &self.source_closure_profile,
            &self.fixed_source,
        )?;
        crate::native_ax_observation::NativeAxObservation::source_contract_from_context_menu_manifest(
            &manifest,
        )
            .map_err(|_| crate::ContextMenuManifestError::Invalid)
    }
}

impl Drop for NativePhysicalRunLayout {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn require_directory(path: &Path, name: &'static str) -> Result<PathBuf, RequestValidationError> {
    let metadata =
        std::fs::metadata(path).map_err(|_| RequestValidationError::PathNotFound(name))?;
    if !metadata.is_dir() {
        return Err(RequestValidationError::NotDirectory(name));
    }
    std::fs::canonicalize(path).map_err(|_| RequestValidationError::Io(name))
}

fn require_file(path: &Path, name: &'static str) -> Result<PathBuf, RequestValidationError> {
    let metadata =
        std::fs::metadata(path).map_err(|_| RequestValidationError::PathNotFound(name))?;
    if !metadata.is_file() {
        return Err(RequestValidationError::NotFile(name));
    }
    std::fs::canonicalize(path).map_err(|_| RequestValidationError::Io(name))
}

fn require_clean_fixed_source(
    path: &Path,
    expected_revision: &str,
) -> Result<PathBuf, RequestValidationError> {
    let source = require_directory(path, "fixed_katana_root")?;
    let top_level = git_output(&source, &["rev-parse", "--show-toplevel"])?;
    let canonical_top_level = require_directory(Path::new(top_level.trim()), "fixed_katana_root")?;
    if canonical_top_level != source {
        return Err(RequestValidationError::Io("fixed_katana_root"));
    }

    let revision = git_output(&source, &["rev-parse", "HEAD"])?;
    if revision.trim() != expected_revision {
        return Err(RequestValidationError::Io("fixed_katana_revision"));
    }
    let status = git_output(&source, &["status", "--porcelain", "--untracked-files=all"])?;
    if !status.trim().is_empty() {
        return Err(RequestValidationError::Io("fixed_katana_worktree"));
    }
    Ok(source)
}

fn git_output(root: &Path, args: &[&str]) -> Result<String, RequestValidationError> {
    let root = root
        .to_str()
        .ok_or(RequestValidationError::Io("fixed_katana_root"))?;
    let output = crate::system::ProcessService::create_command("git")
        .args(["-C", root])
        .args(args)
        .output()
        .map_err(|_| RequestValidationError::Io("fixed_katana_git"))?;
    if !output.status.success() {
        return Err(RequestValidationError::Io("fixed_katana_git"));
    }
    String::from_utf8(output.stdout).map_err(|_| RequestValidationError::Io("fixed_katana_git"))
}
