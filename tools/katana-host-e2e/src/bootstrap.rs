use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapError {
    BootstrapUnavailable,
}

impl fmt::Display for BootstrapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("KatanA physical host bootstrap is not implemented")
    }
}

impl std::error::Error for BootstrapError {}

pub struct HostBootstrap;

impl HostBootstrap {
    pub fn bootstrap() -> Result<(), BootstrapError> {
        Err(BootstrapError::BootstrapUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::{BootstrapError, HostBootstrap};

    #[test]
    fn bootstrap_is_typed_fail_closed() {
        assert_eq!(
            HostBootstrap::bootstrap(),
            Err(BootstrapError::BootstrapUnavailable)
        );
    }
}
