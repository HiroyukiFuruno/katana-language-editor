//! Vendor-neutral host projection lease contract.

/// Leaseを共通表現へ変換せず、host固有の所有権とvendor型の分離を保つ。
///
/// ```
/// use katana_language_editor::{HostProjectionProvider, HostProjectionProviderError};
///
/// struct Lease;
/// struct Provider;
/// struct Fault;
///
/// impl HostProjectionProvider for Provider {
///     type Lease = Lease;
///     type Error = Fault;
///
///     fn retain_lease(&mut self) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
///         Ok(Lease)
///     }
///
///     fn synchronize_lease(&mut self) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
///         Ok(Lease)
///     }
/// }
///
/// let mut provider = Provider;
/// let _ = provider.retain_lease();
/// let _ = provider.synchronize_lease();
/// ```
pub trait HostProjectionProvider {
    type Lease;
    type Error;

    fn retain_lease(&mut self) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>>;

    fn synchronize_lease(
        &mut self,
    ) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum HostProjectionProviderError<ProviderError> {
    MissingHostProjection,
    StaleHostProjection,
    DuplicateHostProjection,
    Provider(ProviderError),
}

#[cfg(test)]
mod tests {
    use super::{HostProjectionProvider, HostProjectionProviderError};

    struct OpaqueLease(u8);
    struct ProviderFault(u8);

    struct Fixture {
        retain: Result<OpaqueLease, HostProjectionProviderError<ProviderFault>>,
        synchronize: Result<OpaqueLease, HostProjectionProviderError<ProviderFault>>,
    }

    impl HostProjectionProvider for Fixture {
        type Lease = OpaqueLease;
        type Error = ProviderFault;

        fn retain_lease(
            &mut self,
        ) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
            std::mem::replace(
                &mut self.retain,
                Err(HostProjectionProviderError::MissingHostProjection),
            )
        }

        fn synchronize_lease(
            &mut self,
        ) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
            std::mem::replace(
                &mut self.synchronize,
                Err(HostProjectionProviderError::StaleHostProjection),
            )
        }
    }

    #[test]
    fn opaque_leases_and_non_debug_errors_are_unconstrained() {
        let mut fixture = Fixture {
            retain: Ok(OpaqueLease(1)),
            synchronize: Err(HostProjectionProviderError::Provider(ProviderFault(7))),
        };

        assert!(matches!(fixture.retain_lease(), Ok(OpaqueLease(1))));
        assert!(matches!(
            fixture.synchronize_lease(),
            Err(HostProjectionProviderError::Provider(error)) if error.0 == 7
        ));
    }

    #[test]
    fn provider_errors_are_forwarded_without_erasure() {
        let mut fixture = Fixture {
            retain: Err(HostProjectionProviderError::Provider(ProviderFault(3))),
            synchronize: Err(HostProjectionProviderError::Provider(ProviderFault(7))),
        };

        assert!(matches!(
            fixture.retain_lease(),
            Err(HostProjectionProviderError::Provider(error)) if error.0 == 3
        ));
        assert!(matches!(
            fixture.synchronize_lease(),
            Err(HostProjectionProviderError::Provider(error)) if error.0 == 7
        ));
    }
}
