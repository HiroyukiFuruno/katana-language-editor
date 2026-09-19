use super::KucRootBindingReceipt;

impl KucRootBindingReceipt {
    #[must_use]
    pub fn root_identity(&self) -> &str {
        &self.root_identity
    }
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.presentation_revision
    }
    #[must_use]
    pub const fn presentation_revision(&self) -> u64 {
        self.presentation_revision
    }
    #[must_use]
    pub const fn state_revision(&self) -> u64 {
        self.state_revision
    }
    #[must_use]
    pub const fn dimensions(&self) -> super::EguiTextCommandSurfaceHostRootRecordDimensions {
        self.dimensions
    }
    #[must_use]
    pub fn paint_plan_hash(&self) -> &str {
        &self.paint_plan_hash
    }
    #[must_use]
    pub fn record_hash(&self) -> &str {
        &self.record_hash
    }
    #[must_use]
    pub fn accessibility_snapshot_hash(&self) -> &str {
        &self.accessibility_snapshot_hash
    }
    #[must_use]
    pub fn correlation_fingerprint(&self) -> &str {
        &self.correlation_fingerprint
    }
    #[must_use]
    pub fn event_batch_fingerprint(&self) -> &str {
        &self.event_batch_fingerprint
    }
    #[must_use]
    pub const fn event_cardinality(&self) -> usize {
        self.event_cardinality
    }
    #[must_use]
    pub const fn consumed_once(&self) -> bool {
        self.consumed_once
    }
}
