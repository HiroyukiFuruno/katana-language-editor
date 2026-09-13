use crate::host_types::StorybookHost;
use katana_ui_core::egui::text_command_surface::{
    FullTextCommandSurfaceMotionPlan, FullTextCommandSurfaceScenarioFactory,
    FullTextCommandSurfaceScenarioId,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const FULL_EDITOR_SCENARIO_COUNT: usize = 8;
static TEST_ARTIFACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn storybook_uses_kuc_scenario_factory_without_decoding_the_lease() -> TestResult {
    let factory = FullTextCommandSurfaceScenarioFactory::new();
    for id in [
        FullTextCommandSurfaceScenarioId::Resting,
        FullTextCommandSurfaceScenarioId::Selection,
        FullTextCommandSurfaceScenarioId::Find,
        FullTextCommandSurfaceScenarioId::Context,
        FullTextCommandSurfaceScenarioId::Readonly,
        FullTextCommandSurfaceScenarioId::ResizeScrollIme,
        FullTextCommandSurfaceScenarioId::NavigationInput,
        FullTextCommandSurfaceScenarioId::WorkspaceTabs,
    ] {
        let scenario = factory.issue_with_router(id, |_context| Ok(None))?;
        assert_eq!(scenario.id(), id);
        assert!(!scenario.stages().is_empty());
        let _lease = scenario.into_lease()?;
    }
    Ok(())
}

#[test]
fn storybook_default_opens_the_workspace_tabs_editor_surface() -> TestResult {
    let host = StorybookHost::new()?;
    assert_eq!(
        host.scenario_stages().len(),
        4,
        "the interactive default must use the KUC workspace-tabs initial, drag, and release trace"
    );
    assert!(
        host.scenario_stages()
            .iter()
            .skip(1)
            .all(|stage| stage.event_count() > 0),
        "the default editor surface includes KUC-owned physical tab interaction stages"
    );
    Ok(())
}

#[test]
fn storybook_window_smoke_recipe_uses_the_bounded_native_interactive_route() -> TestResult {
    let body = just_recipe_body("storybook-window-smoke")?;

    assert_eq!(
        body,
        ["{{CARGO}} run --locked -p kle-storybook -- --interactive --frames 2"],
        "the live window smoke recipe must use the native interactive route with a two-frame bound"
    );
    Ok(())
}

#[test]
fn storybook_contract_check_renders_every_kuc_scenario() -> TestResult {
    let output_dir = test_artifact_dir("contract-check");
    for id in scenario_ids() {
        let mut host = StorybookHost::for_scenario(id)?;
        let frames = host.render_scenario_frames(&output_dir, id)?;
        assert_eq!(frames.len(), host.scenario_stages().len());
        assert!(!frames.is_empty());
        for frame in frames {
            frame.validate_live_evidence()?;
            assert!(!frame.artifact_receipt.stage_id().is_empty());
            assert!(frame.receipt.consumed_once());
            assert!(!frame.receipt.record_hash().is_empty());
            assert!(!frame.receipt.paint_plan_hash().is_empty());
            assert!(!frame.receipt.accessibility_snapshot_hash().is_empty());
        }
    }
    Ok(())
}

#[test]
fn storybook_interaction_roundtrip_uses_opaque_stage_and_receipt() -> TestResult {
    let output_dir = test_artifact_dir("interaction-roundtrip");
    for id in scenario_ids() {
        let mut host = StorybookHost::for_scenario(id)?;
        let stages = host.scenario_stages().to_vec();
        assert!(
            stages.iter().any(|stage| stage.event_count() > 0)
                || id == FullTextCommandSurfaceScenarioId::Resting
                || id == FullTextCommandSurfaceScenarioId::Selection
                || id == FullTextCommandSurfaceScenarioId::Find
        );
        let frames = host.render_scenario_frames(&output_dir, id)?;
        assert!(
            frames
                .iter()
                .all(|frame| frame.validate_live_evidence().is_ok())
        );
    }
    Ok(())
}

#[test]
fn navigation_input_roundtrip_keeps_the_kle_transit_generic() -> TestResult {
    let output_dir = test_artifact_dir("navigation-input");
    let mut host = StorybookHost::for_scenario(FullTextCommandSurfaceScenarioId::NavigationInput)?;
    let frames = host.render_scenario_frames(
        &output_dir,
        FullTextCommandSurfaceScenarioId::NavigationInput,
    )?;

    assert_eq!(frames.len(), host.scenario_stages().len());
    let submitted = frames
        .last()
        .ok_or("navigation scenario has no submit frame")?;
    submitted.validate_live_evidence()?;
    assert_eq!(submitted.receipt.event_cardinality(), 1);
    assert!(
        submitted
            .class_dispatch_records
            .iter()
            .all(|(_, count)| *count == 0)
    );
    Ok(())
}

#[test]
fn storybook_motion_artifact_uses_the_exact_kuc_plan_without_idle_frames() -> TestResult {
    let output_dir = test_artifact_dir("motion-plan");
    let requested_frames = FullTextCommandSurfaceMotionPlan::minimum_frame_count();
    let mut host = StorybookHost::new()?;
    host.write_search_motion_sequence(&output_dir, requested_frames)?;

    let evidence = fs::read_to_string(output_dir.join("search-motion-evidence.txt"))?;
    let frame_lines = evidence
        .lines()
        .filter(|line| line.starts_with("frame["))
        .collect::<Vec<_>>();
    assert_eq!(frame_lines.len(), requested_frames);
    assert!(frame_lines.iter().all(|line| {
        line.contains("scenario_stage=kuc-motion-")
            && !line.contains("scenario_stage=idle")
            && !line.contains("receipt_record_hash= ")
    }));
    assert!(evidence.contains(&format!("requested_frames={requested_frames}")));
    assert!(evidence.contains(&format!("source_frames={requested_frames}")));
    assert!(evidence.contains(&format!("decoded_frames={requested_frames}")));
    assert!(evidence.contains(&format!("source_viewports={requested_frames}")));
    assert!(evidence.contains("star_scalar_sequence=[11088, 65039]"));
    assert!(evidence.contains("ime_preedit_event_seen=true"));
    assert!(evidence.contains("ime_commit_event_seen=true"));
    assert!(evidence.contains("hit_test_count="));
    assert!(evidence.contains("accesskit_snapshot_hash="));
    Ok(())
}

fn scenario_ids() -> [FullTextCommandSurfaceScenarioId; FULL_EDITOR_SCENARIO_COUNT] {
    [
        FullTextCommandSurfaceScenarioId::Resting,
        FullTextCommandSurfaceScenarioId::Selection,
        FullTextCommandSurfaceScenarioId::Find,
        FullTextCommandSurfaceScenarioId::Context,
        FullTextCommandSurfaceScenarioId::Readonly,
        FullTextCommandSurfaceScenarioId::ResizeScrollIme,
        FullTextCommandSurfaceScenarioId::NavigationInput,
        FullTextCommandSurfaceScenarioId::WorkspaceTabs,
    ]
}

fn test_artifact_dir(name: &str) -> PathBuf {
    let sequence = TEST_ARTIFACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("storybook-contract-tests")
        .join(format!("{name}-{}-{sequence}", std::process::id()))
}

fn just_recipe_body(name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let justfile = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("Justfile");
    let content = fs::read_to_string(justfile)?;
    let recipe = format!("{name}:");
    let lines = content.lines().collect::<Vec<_>>();
    let start = lines
        .iter()
        .position(|line| line.trim_end() == recipe)
        .ok_or_else(|| format!("recipe `{name}` was not found"))?;

    Ok(lines[start + 1..]
        .iter()
        .take_while(|line| line.starts_with("    "))
        .map(|line| line.trim().to_string())
        .collect())
}
