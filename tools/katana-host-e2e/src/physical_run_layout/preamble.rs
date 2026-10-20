use crate::physical_bootstrap_types::RequestValidationError;
use crate::{
    AxTargetLocator, ContextMenuManifestLoader, LaunchRequest, SourceDerivedNativeTarget,
    SourceDerivedNativeTargetError,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const SOURCE_CLOSURE_DIR: &str = "target/source-closure";
const ARTIFACT_DIR: &str = "artifacts";
const PROFILE_DIR: &str = "profiles/macos-latest";
const TARGET_RECORD: &str = "source-derived-native-target.json";
const CONTEXT_MENU_MANIFEST: &str = "context-menu-target-manifest.json";
const PROFILE: &str = "probe.json";
pub(crate) const FIXED_KATANA_REVISION: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";

static NEXT_LAYOUT_ID: AtomicU64 = AtomicU64::new(0);
