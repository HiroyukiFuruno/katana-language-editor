use katana_language_editor::{HostProjectionProvider, HostProjectionProviderError};
use std::convert::Infallible;

type LeaseResult = Result<OpaqueHostLease, HostProjectionProviderError<Infallible>>;

fn main() {
    let _: fn(&mut UnconnectedHost) -> LeaseResult = UnconnectedHost::retain_lease;
    let _: fn(&mut UnconnectedHost) -> LeaseResult = UnconnectedHost::synchronize_lease;
}

struct OpaqueHostLease;
struct UnconnectedHost;

/* WHY: UIを持たないconsumerの型検証だけで、KatanA接続や編集成功は模擬しない。 */
impl HostProjectionProvider for UnconnectedHost {
    type Lease = OpaqueHostLease;
    type Error = Infallible;

    fn retain_lease(&mut self) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
        Err(HostProjectionProviderError::MissingHostProjection)
    }

    fn synchronize_lease(
        &mut self,
    ) -> Result<Self::Lease, HostProjectionProviderError<Self::Error>> {
        Err(HostProjectionProviderError::MissingHostProjection)
    }
}
