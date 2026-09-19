use std::{
    io,
    path::PathBuf,
};

#[derive(Debug)]
pub enum FixedSourceHarnessError {
    MissingEnvironment {
        variable: &'static str,
    },
    Canonicalize {
        path: PathBuf,
        source: io::Error,
    },
    ReadProfile {
        path: PathBuf,
        source: io::Error,
    },
    InvalidProfileJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    MissingProfileRevision {
        path: PathBuf,
    },
    ProfileRevisionMismatch {
        expected: &'static str,
        actual: String,
    },
    GitCommand {
        command: &'static str,
        source: io::Error,
    },
    GitFailure {
        command: &'static str,
        status: Option<i32>,
        stderr: String,
    },
    HeadMismatch {
        expected: &'static str,
        actual: String,
    },
    DirtyWorktree {
        status: String,
    },
    CreateHarnessDirectory {
        path: PathBuf,
        source: io::Error,
    },
    SystemTime {
        source: std::time::SystemTimeError,
    },
    WriteManifest {
        path: PathBuf,
        source: io::Error,
    },
    CargoCommand {
        command: &'static str,
        source: io::Error,
    },
    CargoFailure {
        command: &'static str,
        status: Option<i32>,
        stderr: String,
    },
    WriteMetadata {
        path: PathBuf,
        source: io::Error,
    },
    ReadCargoLock {
        path: PathBuf,
        source: io::Error,
    },
    InvalidMetadataJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    CanonicalizeMetadataJson {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl std::fmt::Display for FixedSourceHarnessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnvironment { variable } => {
                write!(formatter, "required environment variable {variable} is not set")
            }
            Self::Canonicalize { path, .. } => {
                write!(formatter, "cannot canonicalize {}", path.display())
            }
            Self::ReadProfile { path, .. } => write!(
                formatter,
                "cannot read source-closure profile {}",
                path.display()
            ),
            Self::InvalidProfileJson { path, .. } => write!(
                formatter,
                "invalid source-closure profile JSON {}",
                path.display()
            ),
            Self::MissingProfileRevision { path } => write!(
                formatter,
                "source-closure profile has no katana_revision: {}",
                path.display()
            ),
            Self::ProfileRevisionMismatch { expected, actual } => write!(
                formatter,
                "profile revision {actual} does not match fixed revision {expected}"
            ),
            Self::GitCommand { command, .. } => write!(formatter, "failed to start git {command}"),
            Self::GitFailure {
                command,
                status,
                stderr,
            } => write!(formatter, "git {command} failed ({status:?}): {stderr}"),
            Self::HeadMismatch { expected, actual } => write!(
                formatter,
                "KatanA HEAD {actual} does not match fixed revision {expected}"
            ),
            Self::DirtyWorktree { status } => {
                write!(formatter, "KatanA worktree is dirty: {status}")
            }
            Self::CreateHarnessDirectory { path, .. } => write!(
                formatter,
                "cannot create harness directory {}",
                path.display()
            ),
            Self::SystemTime { .. } => formatter.write_str("system clock is before the Unix epoch"),
            Self::WriteManifest { path, .. } => write!(
                formatter,
                "cannot write generated manifest {}",
                path.display()
            ),
            Self::CargoCommand { command, .. } => write!(formatter, "failed to start {command}"),
            Self::CargoFailure {
                command,
                status,
                stderr,
            } => write!(formatter, "{command} failed ({status:?}): {stderr}"),
            Self::WriteMetadata { path, .. } => {
                write!(formatter, "cannot write cargo metadata {}", path.display())
            }
            Self::ReadCargoLock { path, .. } => {
                write!(
                    formatter,
                    "cannot read generated Cargo.lock {}",
                    path.display()
                )
            }
            Self::InvalidMetadataJson { path, .. } => write!(
                formatter,
                "cargo metadata returned invalid JSON {}",
                path.display()
            ),
            Self::CanonicalizeMetadataJson { path, .. } => write!(
                formatter,
                "cannot canonicalize cargo metadata JSON {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for FixedSourceHarnessError {}
