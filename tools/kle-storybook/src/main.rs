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
mod tests;
#[cfg(test)]
mod tests_host_projection;

use args::{StorybookArgs, StorybookMode};
use consumer_artifact::ConsumerArtifactRunner;
use host_types::StorybookHost;
use window::StorybookWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = StorybookArgs::parse(std::env::args().skip(1))?;
    let host = StorybookHost::new()?;
    match args.mode {
        StorybookMode::Interactive => StorybookWindow::new(args.resolved_frames())
            .run(host, args.interactive_frame_limit())?,
        StorybookMode::Smoke => StorybookWindow::new(args.resolved_frames()).run_smoke(host)?,
        StorybookMode::InteractionCheck
        | StorybookMode::EmojiCheck
        | StorybookMode::ContractCheck => {
            if args.mode == StorybookMode::ContractCheck {
                let artifact_dir = ConsumerArtifactRunner::write_full_editor_artifact_for_run(
                    std::path::Path::new("target/acceptance/kle-storybook-consumer-artifact"),
                )?;
                eprintln!("KUC consumer artifact: {}", artifact_dir.display());
            }
            let mut host = host;
            host.run_live_check(args.mode)?;
        }
        StorybookMode::MotionArtifact => {
            let mut host = host;
            host.write_search_motion_sequence(
                &args.resolved_artifact_output(),
                args.resolved_frames(),
            )?;
        }
    }
    Ok(())
}
