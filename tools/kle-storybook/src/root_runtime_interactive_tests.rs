use crate::host_types::{
    STORYBOOK_VIEWPORT_HEIGHT, STORYBOOK_VIEWPORT_WIDTH, StorybookHost, StorybookProjectionProvider,
};
use crate::root_runtime::{InjectedProjection, StorybookProjection};
use katana_language_editor_egui::{
    HostProjectionProvider, HostProjectionProviderError, KucRootBindingReceipt,
};
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceHostProjectionLease, FullTextCommandSurfaceRawInputStage,
    FullTextCommandSurfaceScenarioId,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn interactive_root_forwards_two_real_kuc_frames_once_with_stable_identity() -> TestResult {
    let mut host = StorybookHost::for_scenario(FullTextCommandSurfaceScenarioId::WorkspaceTabs)?;
    let context = egui::Context::default();

    let first = render_interactive_frame(&context, &mut host)?;
    let second = render_interactive_frame(&context, &mut host)?;

    assert!(first.consumed_once());
    assert!(second.consumed_once());
    assert_eq!(first.root_identity(), second.root_identity());
    for receipt in [&first, &second] {
        assert!(!receipt.record_hash().is_empty());
        assert!(!receipt.paint_plan_hash().is_empty());
        assert!(!receipt.accessibility_snapshot_hash().is_empty());
    }
    Ok(())
}

#[test]
fn retained_provider_synchronizes_each_real_scenario_frame_before_dispatch() -> TestResult {
    let (provider, stages) =
        StorybookProjectionProvider::for_scenario(FullTextCommandSurfaceScenarioId::WorkspaceTabs)?;
    assert!(!stages.is_empty());
    assert!(stages.iter().any(|stage| stage.event_count() > 0));

    let counts = Arc::new(ProjectionProviderCounts::default());
    let mut projection = InjectedProjection::new(CountingProjectionProvider {
        inner: provider,
        counts: Arc::clone(&counts),
    })?;
    let context = egui::Context::default();

    let receipts = stages
        .iter()
        .map(|stage| render_scenario_interactive_frame(&context, &mut projection, stage))
        .collect::<Result<Vec<_>, _>>()?;

    assert!(receipts.iter().all(KucRootBindingReceipt::consumed_once));
    assert!(
        receipts
            .iter()
            .all(|receipt| !receipt.record_hash().is_empty())
    );
    assert_eq!(counts.retained.load(Ordering::Relaxed), 1);
    assert_eq!(counts.synchronized.load(Ordering::Relaxed), receipts.len());
    assert!(counts.synchronized.load(Ordering::Relaxed) > 0);
    Ok(())
}

fn render_interactive_frame(
    context: &egui::Context,
    host: &mut StorybookHost,
) -> Result<KucRootBindingReceipt, Box<dyn std::error::Error>> {
    let mut result = None;
    let mut output = context.run_ui(interactive_input(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| result = Some(host.show_interactive(ui)));
    });
    output.textures_delta.clear();
    Ok(result.ok_or("interactive root did not render")??)
}

fn render_scenario_interactive_frame(
    context: &egui::Context,
    projection: &mut impl StorybookProjection,
    stage: &FullTextCommandSurfaceRawInputStage,
) -> Result<KucRootBindingReceipt, Box<dyn std::error::Error>> {
    let mut input = interactive_input();
    stage.apply_to(&mut input);
    let mut result = None;
    let mut output = context.run_ui(input, |ctx| {
        egui::CentralPanel::default()
            .show(ctx, |ui| result = Some(projection.show_interactive(ui)));
    });
    output.textures_delta.clear();
    Ok(result.ok_or("interactive scenario root did not render")??)
}

fn interactive_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                STORYBOOK_VIEWPORT_WIDTH as f32,
                STORYBOOK_VIEWPORT_HEIGHT as f32,
            ),
        )),
        ..egui::RawInput::default()
    }
}

#[derive(Default)]
struct ProjectionProviderCounts {
    retained: AtomicUsize,
    synchronized: AtomicUsize,
}

struct CountingProjectionProvider {
    inner: StorybookProjectionProvider,
    counts: Arc<ProjectionProviderCounts>,
}

impl HostProjectionProvider for CountingProjectionProvider {
    type Lease = EguiTextCommandSurfaceHostProjectionLease;
    type Error = String;

    fn retain_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        self.counts.retained.fetch_add(1, Ordering::Relaxed);
        self.inner.retain_lease()
    }

    fn synchronize_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        self.counts.synchronized.fetch_add(1, Ordering::Relaxed);
        self.inner.synchronize_lease()
    }
}
