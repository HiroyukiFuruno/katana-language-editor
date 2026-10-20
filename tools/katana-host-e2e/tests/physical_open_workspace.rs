#![cfg(target_os = "macos")]

use katana_host_e2e_fixed::{
    AxPreflight, AxTargetLocator, AxWindowCreatedObserver, KatanAChild, KatanACommand,
    FixedSourceHarnessBuilder, NativePhysicalRunLayout,
};
use std::time::{Duration, Instant};

const AX_TARGET_READY_TIMEOUT: Duration = Duration::from_secs(30);

#[test]
fn fixed_source_open_workspace_requires_native_selection() -> Result<(), String> {
    AxPreflight::check().map_err(|error| format!("{error}"))?;
    let fixed_source =
        FixedSourceHarnessBuilder::required_katana_repo().map_err(|error| error.to_string())?;
    let layout = NativePhysicalRunLayout::from_compiled_repository(fixed_source)
        .map_err(|error| format!("{error}"))?;
    let locator = layout
        .load_source_derived_locator()
        .map_err(|error| format!("{error}"))?;
    let request = layout
        .launch_request()
        .map_err(|error| format!("{error}"))?;
    let mut child = KatanACommand::from_request(&request)
        .map_err(|error| format!("{error}"))?
        .spawn()
        .map_err(|error| format!("{error}"))?;

    let result = observe_open_workspace(&child, &locator);
    let termination = child.terminate().map_err(|error| format!("{error}"));
    result
        .map(|_| ())
        .and(termination.map_err(|error| error.to_string()))
}

#[test]
fn fixed_source_workspace_restoration_requires_editor_observation() -> Result<(), String> {
    AxPreflight::check().map_err(|error| format!("{error}"))?;
    let fixed_source =
        FixedSourceHarnessBuilder::required_katana_repo().map_err(|error| error.to_string())?;
    let layout = NativePhysicalRunLayout::from_compiled_repository(fixed_source)
        .map_err(|error| format!("{error}"))?;
    let contract = layout
        .load_workspace_observation_contract()
        .map_err(|error| format!("editor source contract load failed: {error}"))?;
    let request = layout
        .launch_request()
        .map_err(|error| format!("{error}"))?;
    let mut child = KatanACommand::from_request(&request)
        .map_err(|error| format!("{error}"))?
        .spawn()
        .map_err(|error| format!("{error}"))?;

    let result = observe_workspace_restoration(&child, &contract, layout.workspace_basename());
    let termination = child.terminate().map_err(|error| format!("{error}"));
    result
        .map(|_| ())
        .and(termination.map_err(|error| error.to_string()))
}

fn observe_open_workspace(child: &KatanAChild, locator: &AxTargetLocator) -> Result<(), String> {
    let application = child
        .ax_application_element()
        .map_err(|error| format!("launched KatanA AX application binding failed: {error}"))?;
    AxWindowCreatedObserver::register(&application, child)
        .map_err(|error| format!("AX KatanA window observer registration failed: {error}"))?
        .wait_for_existing_window_or_notification(&application)
        .map_err(|error| format!("KatanA main window AX notification was not observed: {error}"))?;
    let observer = AxWindowCreatedObserver::register_native_dialog(&application, child)
        .map_err(|error| format!("AX native-dialog observer registration failed: {error}"))?;
    wait_for_ax_target(|| application.locate(locator)).map_err(|error| {
        format!("source-derived Open Workspace target resolution failed: {error}")
    })?;
    application
        .click(locator)
        .map_err(|error| format!("dynamic AX-resolved physical click failed: {error}"))?;
    observer.wait_for_notification().map_err(|error| {
        format!(
            "native dialog observability AXWindowCreated notification was not observed: {error}"
        )
    })?;
    Ok(())
}

fn observe_workspace_restoration(
    child: &KatanAChild,
    contract: &katana_host_e2e_fixed::NativeAxSourceContract,
    workspace_basename: &str,
) -> Result<(), String> {
    let application = child
        .ax_application_element()
        .map_err(|error| format!("launched KatanA AX application binding failed: {error}"))?;
    AxWindowCreatedObserver::register(&application, child)
        .map_err(|error| format!("AX KatanA window observer registration failed: {error}"))?
        .wait_for_existing_window_or_notification(&application)
        .map_err(|error| format!("KatanA main window AX notification was not observed: {error}"))?;
    wait_for_ax_target(|| application.observe_workspace_frame(contract, workspace_basename))
        .map_err(|error| format!("workspace-correlated editor observation failed: {error}"))?;
    Ok(())
}

fn wait_for_ax_target<T, E>(mut resolve: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    let deadline = Instant::now() + AX_TARGET_READY_TIMEOUT;
    loop {
        match resolve() {
            Ok(value) => return Ok(value),
            Err(error) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => return Err(error),
        }
    }
}
