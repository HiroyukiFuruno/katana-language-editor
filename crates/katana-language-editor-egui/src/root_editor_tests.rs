use super::*;
use crate::host_projection_provider::{HostProjectionProvider, HostProjectionProviderError};
use katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease;

struct FailingProvider;
impl HostProjectionProvider for FailingProvider {
    type Lease = EguiTextCommandSurfaceHostProjectionLease;
    type Error = &'static str;
    fn retain_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        Err(HostProjectionProviderError::Provider("provider failed"))
    }
    fn synchronize_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        Err(HostProjectionProviderError::Provider("provider failed"))
    }
}

fn production_source() -> &'static str {
    include_str!("root_editor.rs")
        .split_once("#[cfg(test)]")
        .map_or(include_str!("root_editor.rs"), |(source, _)| source)
}

#[test]
fn failed_provider_is_typed_and_fail_closed() {
    let result = EguiTextCommandSurfaceEditor::new(FailingProvider);
    assert!(matches!(
        result,
        Err(EguiTextCommandSurfaceEditorError::Binding(
            HostProjectionBindingError::Provider(HostProjectionProviderError::Provider(
                "provider failed"
            ))
        ))
    ));
}

#[test]
fn source_contract_has_only_the_binding_field() -> Result<(), &'static str> {
    let source = production_source();
    let fields = source
        .split_once("pub struct EguiTextCommandSurfaceEditor<Provider> {")
        .and_then(|(_, rest)| rest.split_once("}\n\n#[derive"))
        .map(|(fields, _)| fields)
        .ok_or("editor definition was not found")?;
    assert_eq!(fields.matches(':').count(), 1);
    assert!(fields.contains("binding: HostProjectionBinding<Provider>"));
    Ok(())
}

#[test]
fn show_delegates_to_binding_once() {
    assert_eq!(
        production_source()
            .matches("self.binding\n            .show_and_forward_once(")
            .count(),
        1
    );
}

#[cfg(feature = "storybook-artifacts")]
#[test]
fn artifact_show_delegates_to_binding_once() {
    assert_eq!(
        production_source()
            .matches("self.binding\n            .show_write_artifact_and_forward_once(")
            .count(),
        1
    );
}

#[test]
fn production_source_has_no_domain_semantic_state_or_token_operations() {
    let source = production_source();
    for forbidden in [
        "EguiTextCommandSurfacePresentationToken",
        "KucRootEffectRouter",
        "KucRootEventBatchContext",
        "EguiTextCommandSurfaceRootEventTransport",
        "dispatch_once",
        "TextContent",
        "query",
        "selection",
        "cursor",
        "font",
        "emoji",
        "child",
        "host state",
        "semantic",
        "decode",
        "clone",
        "serialize",
        "inspect",
    ] {
        assert!(!source.contains(forbidden), "source contains `{forbidden}`");
    }
}
