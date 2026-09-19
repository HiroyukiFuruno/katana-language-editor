use std::path::PathBuf;
use std::process::{Child, Command};

#[derive(Debug, Eq, PartialEq)]
pub enum RequestValidationError {
    EmptyPath(&'static str),
    PathNotFound(&'static str),
    NotDirectory(&'static str),
    NotFile(&'static str),
    MissingCargoManifest,
    MarkdownFixtureMissing,
    Io(&'static str),
    FixtureCreationFailed,
    WorkspacePathNotAbsolute,
    WorkspacePathTraversal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceConfigError {
    Serialization,
    Write,
    Verification,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchRequest {
    pub(crate) fixed_katana_root: PathBuf,
    pub(crate) execution_sandbox: PathBuf,
    pub(crate) workspace_fixture: PathBuf,
    pub(crate) target_dir: PathBuf,
    pub(crate) config_dir: PathBuf,
}

#[derive(Debug)]
pub enum ChildLaunchError {
    Command(RequestValidationError),
    WorkspaceConfig(WorkspaceConfigError),
    Spawn(std::io::Error),
    Wait(std::io::Error),
    Kill(std::io::Error),
}

pub struct KatanACommand {
    pub(crate) command: Command,
}

pub struct KatanAChild {
    pub(crate) child: Child,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxPreflightError {
    UnsupportedPlatform,
    AccessibilityNotTrusted,
    InputAuthorizationNotTrusted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxApplicationElementError {
    UnsupportedPlatform,
    AccessibilityNotTrusted,
    InputAuthorizationNotTrusted,
    NullElement,
    Unreachable,
    SystemFailure,
}

pub struct AxApplicationElement {
    #[cfg(target_os = "macos")]
    pub(crate) element: objc2_core_foundation::CFRetained<objc2_application_services::AXUIElement>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxClickError {
    UnsupportedPlatform,
    AccessibilityNotTrusted,
    TargetSelection,
    TargetRevalidationFailed,
    BoundsMissing,
    BoundsTypeMismatch,
    BoundsValueInvalid,
    EventAuthorizationNotTrusted,
    EventSourceCreationFailed,
    EventCreationFailed,
    EventPostFailed,
    SystemFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxPhysicalInput {
    PrimaryPointer,
    SecondaryPointer,
    ShiftF10,
    AccessibilityPress,
}

pub struct AxPreflight;
