use std::{env, fs, path::Path, process::Output};

use crate::system::ProcessService;

pub(super) fn verify_native_profile(id: &str, host: &str, cfg: &[u8]) -> Result<(), String> {
    let host_matches = match id {
        "macos-latest" => host.ends_with("-apple-darwin"),
        "windows-latest" => host.contains("-windows-"),
        "ubuntu-latest" => host.contains("-linux-"),
        _ => false,
    };
    if !host_matches {
        return Err(format!(
            "runner profile {id} does not match the actual rustc host {host}; local OS masquerading is rejected"
        ));
    }
    let target_os = match id {
        "macos-latest" => "macos",
        "windows-latest" => "windows",
        "ubuntu-latest" => "linux",
        _ => return Err(format!("unsupported runner profile id: {id}")),
    };
    if !cfg.is_empty()
        && !String::from_utf8_lossy(cfg).contains(&format!("target_os=\"{target_os}\""))
    {
        return Err(format!(
            "runner profile {id} has no native target_os={target_os} cfg"
        ));
    }
    let runner_os = match env::var("RUNNER_OS") {
        Err(_) => return Ok(()),
        Ok(value) => value,
    };
    let expected = match id {
        "macos-latest" => "macOS",
        "windows-latest" => "Windows",
        "ubuntu-latest" => "Linux",
        _ => return Err(format!("unsupported runner profile id: {id}")),
    };
    if runner_os != expected {
        return Err(format!("GitHub RUNNER_OS={runner_os} does not match {id}"));
    }
    Ok(())
}

pub(super) fn rustc_host(vv: &[u8]) -> Result<String, String> {
    std::str::from_utf8(vv)
        .map_err(|error| format!("rustc -vV output is not UTF-8: {error}"))?
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
        .filter(|host| !host.trim().is_empty())
        .ok_or_else(|| "rustc -vV output has no host line".into())
}

pub(super) fn cargo_metadata_args(katana_root: &Path) -> Vec<String> {
    vec![
        "metadata".into(),
        "--locked".into(),
        "--offline".into(),
        "--format-version".into(),
        "1".into(),
        "--manifest-path".into(),
        katana_root
            .join("Cargo.toml")
            .to_string_lossy()
            .into_owned(),
    ]
}

pub(super) fn command_text(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn run_command<S: AsRef<std::ffi::OsStr>>(
    program: &str,
    args: &[S],
    current_dir: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let mut command = ProcessService::create_command(program);
    command.args(args);
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    finish_command(command.output(), program)
}

pub(super) fn git_output(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let mut command = ProcessService::create_command("git");
    command.arg("-C").arg(root).args(args);
    finish_command(command.output(), "git")
}

pub(super) fn finish_command(
    output: std::io::Result<Output>,
    program: &str,
) -> Result<Vec<u8>, String> {
    let output = output.map_err(|error| format!("failed to execute {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed with status {}: {}",
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    if !output.stderr.is_empty() {
        return Err(format!(
            "{program} emitted stderr during a raw capture: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

pub(super) fn git_files(root: &Path, patterns: &[&str]) -> Result<Vec<String>, String> {
    let mut args = vec!["ls-files", "-z", "--"];
    args.extend(patterns.iter().copied());
    let bytes = git_output(root, &args)?;
    let mut paths = bytes
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            String::from_utf8(path.to_vec())
                .map_err(|error| format!("git path is not UTF-8: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub(super) fn read_repo_file(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    super::operational_evidence::validate_relative_path(relative)?;
    let path = root.join(relative);
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("source file is missing: {relative}: {error}"))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(format!("source file is foreign or not a file: {relative}"));
    }
    fs::read(canonical).map_err(|error| format!("source file is unreadable: {relative}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::cargo_metadata_args;
    use std::path::Path;

    #[test]
    fn source_closure_metadata_is_locked_and_offline_after_dependency_fetch() {
        let args = cargo_metadata_args(Path::new("/fixed/katana"));
        assert_eq!(
            args[..5],
            ["metadata", "--locked", "--offline", "--format-version", "1"]
        );
    }
}
