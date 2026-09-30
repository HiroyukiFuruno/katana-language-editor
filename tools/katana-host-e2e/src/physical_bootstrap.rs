use crate::system::ProcessService;
use std::fmt;
#[cfg(test)]
use std::process::Command;
use std::process::ExitStatus;

use crate::physical_bootstrap_types::{
    ChildLaunchError, KatanAChild, KatanACommand, LaunchRequest,
};

impl fmt::Display for ChildLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(error) => write!(formatter, "KatanA command is invalid: {error}"),
            Self::WorkspaceConfig(error) => {
                write!(
                    formatter,
                    "KatanA workspace config preparation failed: {error}"
                )
            }
            Self::Spawn(error) => write!(formatter, "KatanA child failed to start: {error}"),
            Self::Build(status) => write!(formatter, "KatanA build failed: {status}"),
            Self::Wait(error) => write!(formatter, "KatanA child wait failed: {error}"),
            Self::Kill(error) => write!(formatter, "KatanA child termination failed: {error}"),
        }
    }
}

impl std::error::Error for ChildLaunchError {}

impl KatanACommand {
    pub fn from_request(request: &LaunchRequest) -> Result<Self, ChildLaunchError> {
        request
            .prepare_workspace_config()
            .map_err(ChildLaunchError::WorkspaceConfig)?;
        let manifest_path = request.fixed_katana_root.join("Cargo.toml");
        let mut build_command = ProcessService::create_command("cargo");
        build_command
            .arg("build")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("-p")
            .arg("katana-ui")
            .arg("--bin")
            .arg("KatanA")
            .current_dir(request.fixed_katana_root())
            .env("CARGO_TARGET_DIR", &request.target_dir);
        let mut command = ProcessService::create_command(katana_binary_path(request));
        command
            .current_dir(request.fixed_katana_root())
            .env("KATANA_CONFIG_DIR", &request.config_dir);
        Ok(Self {
            build_command,
            command,
        })
    }

    pub fn spawn(mut self) -> Result<KatanAChild, ChildLaunchError> {
        let build_status = self.build_command.status().map_err(ChildLaunchError::Spawn)?;
        if !build_status.success() {
            return Err(ChildLaunchError::Build(build_status));
        }
        let child = self.command.spawn().map_err(ChildLaunchError::Spawn)?;
        Ok(KatanAChild { child })
    }

    #[cfg(test)]
    fn command(&self) -> &Command {
        &self.command
    }

    #[cfg(test)]
    fn build_command(&self) -> &Command {
        &self.build_command
    }
}

fn katana_binary_path(request: &LaunchRequest) -> std::path::PathBuf {
    let binary = if cfg!(windows) { "KatanA.exe" } else { "KatanA" };
    request.target_dir.join("debug").join(binary)
}

impl KatanAChild {
    pub(crate) fn process_id(&self) -> u32 {
        self.child.id()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, ChildLaunchError> {
        self.child.try_wait().map_err(ChildLaunchError::Wait)
    }

    pub fn wait(mut self) -> Result<ExitStatus, ChildLaunchError> {
        self.child.wait().map_err(ChildLaunchError::Wait)
    }

    pub fn terminate(&mut self) -> Result<(), ChildLaunchError> {
        self.child.kill().map_err(ChildLaunchError::Kill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_bootstrap_request::{CONFIG_DIR_NAME, TARGET_DIR_NAME};
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        fixed: PathBuf,
        sandbox: PathBuf,
        workspace: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!("katana-host-e2e-command-{id}"));
            let fixed = root.join("fixed");
            let sandbox = root.join("sandbox");
            let workspace = root.join("workspace");
            fs::create_dir_all(&fixed).expect("fixed root");
            fs::create_dir_all(&workspace).expect("workspace");
            fs::write(fixed.join("Cargo.toml"), "[workspace]\n").expect("manifest");
            fs::write(workspace.join("fixture.md"), "# fixture\n").expect("markdown");
            Self {
                root,
                fixed,
                sandbox,
                workspace,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn command_builds_then_launches_the_fixed_katana_binary() {
        let fixture = Fixture::new();
        let request = LaunchRequest::new(&fixture.fixed, &fixture.sandbox, &fixture.workspace)
            .expect("valid request");
        let command = KatanACommand::from_request(&request).expect("command");
        assert!(request.workspace_config_path().is_file());
        let build_args: Vec<OsString> = command
            .build_command()
            .get_args()
            .map(OsString::from)
            .collect();
        assert_eq!(build_args[0], "build");
        assert_eq!(build_args[1], "--manifest-path");
        assert_eq!(
            build_args[2],
            request.fixed_katana_root().join("Cargo.toml").as_os_str()
        );
        assert_eq!(build_args[3], "-p");
        assert_eq!(build_args[4], "katana-ui");
        assert_eq!(build_args[5], "--bin");
        assert_eq!(build_args[6], "KatanA");
        assert_eq!(
            command
                .build_command()
                .get_envs()
                .find(|(key, _)| *key == "CARGO_TARGET_DIR")
                .map(|(_, value)| value.unwrap()),
            Some(
                request
                    .execution_sandbox()
                    .join(TARGET_DIR_NAME)
                .as_os_str()
            )
        );
        assert_eq!(
            command.command().get_program(),
            request
                .execution_sandbox()
                .join(TARGET_DIR_NAME)
                .join("debug")
                .join(if cfg!(windows) { "KatanA.exe" } else { "KatanA" })
        );
        assert_eq!(
            command
                .command()
                .get_envs()
                .find(|(key, _)| *key == "KATANA_CONFIG_DIR")
                .map(|(_, value)| value.unwrap()),
            Some(
                request
                    .execution_sandbox()
                    .join(CONFIG_DIR_NAME)
                .as_os_str()
            )
        );
    }
}
