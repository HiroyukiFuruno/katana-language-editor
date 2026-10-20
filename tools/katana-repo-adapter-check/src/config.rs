use std::{env, path::PathBuf};

const KATANA_REPO_ENV: &str = "KATANA_REPO";
const DEFAULT_KATANA_REPO: &str = "../katana";

#[derive(Debug)]
pub(crate) struct KatanaRepoAdapterCheckConfig {
    pub(crate) repo_root: PathBuf,
}

impl KatanaRepoAdapterCheckConfig {
    pub(crate) fn from_env_args() -> Result<Self, String> {
        let mut repo_root = env::var_os(KATANA_REPO_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_KATANA_REPO));
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--repo" => {
                    let Some(value) = args.next() else {
                        return Err("--repo requires a path".to_string());
                    };
                    repo_root = PathBuf::from(value);
                }
                "--help" | "-h" => return Err(Self::usage()),
                other => return Err(format!("unknown argument: {other}\n{}", Self::usage())),
            }
        }
        Ok(Self { repo_root })
    }

    fn usage() -> String {
        "usage: katana-repo-adapter-check [--repo <path>]".to_string()
    }
}
