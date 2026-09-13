#![cfg(target_os = "macos")]

use katana_host_e2e_fixed::{
    AxWindowCreatedObserver, FixedSourceHarnessBuilder, KatanACommand, NativePhysicalRunLayout,
};

#[test]
fn native_ax_workspace_observation_is_real_or_typed_failure() -> Result<(), String> {
    let fixed_source =
        FixedSourceHarnessBuilder::required_katana_repo().map_err(|error| error.to_string())?;
    let layout = NativePhysicalRunLayout::from_compiled_repository(fixed_source)
        .map_err(|error| format!("typed physical layout failure: {error}"))?;
    let source = layout
        .load_workspace_observation_contract()
        .map_err(|error| format!("typed source contract failure: {error}"))?;
    let request = layout
        .launch_request()
        .map_err(|error| format!("typed launch request failure: {error}"))?;
    let mut child = KatanACommand::from_request(&request)
        .map_err(|error| format!("typed child command failure: {error}"))?
        .spawn()
        .map_err(|error| format!("typed child spawn failure: {error}"))?;

    let result = (|| {
        let application = child
            .observe_ax_application_element()
            .map_err(|error| format!("typed AX application failure: {error}"))?;
        AxWindowCreatedObserver::register(&application, &child)
            .map_err(|error| format!("typed AX observer registration failure: {error}"))?
            .wait_for_notification()
            .map_err(|error| format!("typed AX current-frame availability failure: {error}"))?;
        application
            .observe_workspace_frame(&source, layout.workspace_basename())
            .map_err(|error| format!("typed native AX observation failure: {error}"))
    })();
    child
        .terminate()
        .map_err(|error| format!("typed child termination failure: {error}"))?;
    result.map(|_| ())
}
