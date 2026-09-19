use super::{
    HostProjectionBinding, HostProjectionBindingError, HostProjectionProvider,
    HostProjectionProviderError,
};
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceHostProjectionLease, EguiTextCommandSurfacePresentationToken,
};

struct OneShotProvider<E> {
    retain:
        Option<Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<E>>>,
    synchronize:
        Option<Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<E>>>,
}

impl<E> HostProjectionProvider for OneShotProvider<E> {
    type Lease = EguiTextCommandSurfaceHostProjectionLease;
    type Error = E;

    fn retain_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        self.retain
            .take()
            .ok_or(HostProjectionProviderError::MissingHostProjection)?
    }

    fn synchronize_lease(
        &mut self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, HostProjectionProviderError<Self::Error>>
    {
        self.synchronize
            .take()
            .ok_or(HostProjectionProviderError::MissingHostProjection)?
    }
}

fn opaque_token(payload: &[u8]) -> EguiTextCommandSurfacePresentationToken {
    EguiTextCommandSurfacePresentationToken::from_opaque_bytes(
        1,
        katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostTargetToken::from_opaque_bytes(
            b"host-target",
        ),
        payload,
    )
}

fn opaque_lease(payload: &[u8]) -> EguiTextCommandSurfaceHostProjectionLease {
    EguiTextCommandSurfaceHostProjectionLease::new(opaque_token(payload), |_| Ok(None))
}

#[test]
fn provider_consumes_retain_and_synchronize_leases_once_without_cloning_them() {
    let mut provider = OneShotProvider::<&'static str> {
        retain: Some(Ok(opaque_lease(b"retain"))),
        synchronize: Some(Ok(opaque_lease(b"synchronize"))),
    };

    assert!(provider.retain_lease().is_ok());
    assert!(provider.synchronize_lease().is_ok());
    assert!(provider.retain.is_none());
    assert!(provider.synchronize.is_none());
}

#[test]
fn egui_reexport_is_the_neutral_provider_contract() {
    fn requires_neutral_provider<P: katana_language_editor::HostProjectionProvider>() {}
    fn preserves_neutral_error_identity<E>(
        error: katana_language_editor::HostProjectionProviderError<E>,
    ) -> HostProjectionProviderError<E> {
        error
    }

    requires_neutral_provider::<OneShotProvider<&'static str>>();
    assert!(matches!(
        preserves_neutral_error_identity::<&'static str>(
            katana_language_editor::HostProjectionProviderError::MissingHostProjection
        ),
        HostProjectionProviderError::MissingHostProjection
    ));
}

#[test]
fn provider_errors_are_typed_and_fail_closed() {
    let mut provider = OneShotProvider::<&'static str> {
        retain: Some(Err(HostProjectionProviderError::MissingHostProjection)),
        synchronize: Some(Err(HostProjectionProviderError::Provider("host failed"))),
    };

    assert!(matches!(
        provider.retain_lease(),
        Err(HostProjectionProviderError::MissingHostProjection)
    ));
    assert!(matches!(
        provider.synchronize_lease(),
        Err(HostProjectionProviderError::Provider("host failed"))
    ));
}

#[test]
fn binding_forwards_provider_error_at_the_provider_boundary() {
    let provider = OneShotProvider::<&'static str> {
        retain: Some(Err(HostProjectionProviderError::StaleHostProjection)),
        synchronize: None,
    };

    let result = HostProjectionBinding::new(provider);
    assert!(matches!(
        result,
        Err(HostProjectionBindingError::Provider(
            HostProjectionProviderError::StaleHostProjection
        ))
    ));
}

#[test]
fn binding_retain_failure_is_preserved_as_a_typed_kuc_boundary_error() {
    let provider = OneShotProvider::<&'static str> {
        retain: Some(Ok(opaque_lease(b"invalid"))),
        synchronize: None,
    };

    let result = HostProjectionBinding::new(provider);
    assert!(matches!(result, Err(HostProjectionBindingError::Retain(_))));
}

#[test]
fn production_boundary_does_not_decode_clone_serialize_tokens_or_router_types() {
    let source = include_str!("host_projection_provider.rs");
    assert!(source.contains("retain_lease()"));
    assert!(source.contains("synchronize_lease()"));
    assert!(source.contains("show_and_forward_once(lease"));
    assert!(!source.contains("EguiTextCommandSurfacePresentationToken"));
    assert!(!source.contains("KucRootEffectRouter"));
    assert!(!source.contains("KucRootEventBatchContext"));
    assert!(!source.contains("EguiTextCommandSurfaceRootEventTransport"));
    assert!(!source.contains("dispatch_once"));
    assert!(!source.contains("token.clone()"));
    assert!(!source.contains("decode"));
    assert!(!source.contains("serialize"));
    assert!(!source.contains("format!(\"{:?}\""));
}
