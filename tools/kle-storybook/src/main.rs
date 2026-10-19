mod args;
mod consumer_artifact;
mod host_runtime;
mod host_types;
mod host_types_runtime;
mod root_runtime;
mod scenario_manifest;
mod scenario_manifest_catalog;
mod scenario_manifest_document_find;
mod scenario_manifest_leaf;
mod scenario_manifest_replace;
#[cfg(test)]
mod scenario_manifest_tests;
mod scenario_manifest_types;
mod search_motion_sequence;
mod window;

#[cfg(test)]
mod args_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_host_projection;

use args::{StorybookArgs, StorybookMode};
use consumer_artifact::ConsumerArtifactRunner;
use host_types::StorybookHost;
use window::StorybookWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = StorybookArgs::parse(std::env::args().skip(1))?;
    run(args)
}

fn run(args: StorybookArgs) -> Result<(), Box<dyn std::error::Error>> {
    match args.mode {
        StorybookMode::Interactive | StorybookMode::Smoke => run_window(args),
        StorybookMode::InteractionCheck | StorybookMode::ContractCheck => run_check(args),
        StorybookMode::MotionArtifact => run_motion(args),
        StorybookMode::ConsumerArtifactPlan => run_consumer_plan(args),
    }
}

fn run_window(args: StorybookArgs) -> Result<(), Box<dyn std::error::Error>> {
    let host = StorybookHost::new()?;
    match args.mode {
        StorybookMode::Interactive => StorybookWindow::new(args.resolved_frames())
            .run(host, args.interactive_frame_limit())?,
        StorybookMode::Smoke => StorybookWindow::new(args.resolved_frames()).run_smoke(host)?,
        _ => return Err("window runner received a non-window mode".into()),
    }
    Ok(())
}

fn run_check(args: StorybookArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.mode == StorybookMode::ContractCheck {
        let artifact_dir = ConsumerArtifactRunner::write_full_editor_artifact_for_run(
            std::path::Path::new("target/acceptance/kle-storybook-consumer-artifact"),
        )?;
        eprintln!("KUC consumer artifact: {}", artifact_dir.display());
    }
    StorybookHost::new()?.run_live_check(args.mode)?;
    Ok(())
}

fn run_motion(args: StorybookArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut host = StorybookHost::new()?;
    host.write_search_motion_sequence(&args.resolved_artifact_output(), args.resolved_frames())?;
    Ok(())
}

fn run_consumer_plan(args: StorybookArgs) -> Result<(), Box<dyn std::error::Error>> {
    let artifact_dir = ConsumerArtifactRunner::write_source_leaf_artifact(
        args.required_consumer_plan_input()?,
        &args.resolved_artifact_output(),
    )?;
    eprintln!(
        "KUC source-leaf consumer artifact: {}",
        artifact_dir.display()
    );
    Ok(())
}
