mod audit;
mod audit_evidence;
mod audit_runner;
mod config;
mod repo_paths;
mod system;

#[cfg(test)]
mod audit_tests;
#[cfg(test)]
mod audit_tests_fixture;

use audit::KatanaRepoAdapterAudit;
use config::KatanaRepoAdapterCheckConfig;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = KatanaRepoAdapterCheckConfig::from_env_args()?;
    KatanaRepoAdapterAudit::new(config).validate()?;
    println!("actual KatanA repo KLE downstream adapter evidence is present");
    Ok(())
}
