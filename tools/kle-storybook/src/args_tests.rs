use super::args::{StorybookArgs, StorybookMode};
use std::path::PathBuf;

#[test]
fn parses_interaction_check() -> Result<(), String> {
    let args = StorybookArgs::parse(["--interaction-check"].into_iter().map(str::to_string))?;
    assert_eq!(args.mode, StorybookMode::InteractionCheck);
    Ok(())
}

#[test]
fn resolves_default_motion_sequence_output() -> Result<(), String> {
    let args = StorybookArgs::parse(["--motion-artifact"].into_iter().map(str::to_string))?;
    assert_eq!(args.mode, StorybookMode::MotionArtifact);
    assert!(
        args.resolved_artifact_output()
            .ends_with("kle-storybook-motion-artifact")
    );
    Ok(())
}

#[test]
fn rejects_removed_aliases() {
    for alias in ["--live-acceptance-artifact", "--emoji-check"] {
        let result = StorybookArgs::parse([alias].into_iter().map(str::to_string));
        assert!(matches!(result, Err(error) if error.contains("unknown KLE Storybook argument")));
    }
}

#[test]
fn interactive_frames_are_explicit_and_positive() -> Result<(), String> {
    let default_args = StorybookArgs::parse(std::iter::empty())?;
    assert_eq!(default_args.interactive_frame_limit(), None);
    let explicit = StorybookArgs::parse(
        ["--interactive", "--frames", "1"]
            .into_iter()
            .map(str::to_string),
    )?;
    assert_eq!(explicit.interactive_frame_limit(), Some(1));
    Ok(())
}

#[test]
fn rejects_zero_interactive_frames() {
    let result = StorybookArgs::parse(
        ["--interactive", "--frames", "0"]
            .into_iter()
            .map(str::to_string),
    );
    assert!(matches!(result, Err(error) if error.contains("greater than 0")));
}

#[test]
fn parses_consumer_artifact_plan_input() -> Result<(), String> {
    let args = StorybookArgs::parse(
        [
            "--consumer-artifact-plan",
            "--consumer-plan-input",
            "plan.json",
            "--artifact-output",
            "out",
        ]
        .into_iter()
        .map(str::to_string),
    )?;
    assert_eq!(args.mode, StorybookMode::ConsumerArtifactPlan);
    assert_eq!(
        args.required_consumer_plan_input()?,
        &PathBuf::from("plan.json")
    );
    assert_eq!(args.resolved_artifact_output(), PathBuf::from("out"));
    Ok(())
}
