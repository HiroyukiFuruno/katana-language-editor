use crate::release_gate::ReleaseGateAudit;

const FIXED_KATANA_SHA: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";

impl ReleaseGateAudit {
    pub(crate) fn validate_fixed_source_closure_katana_checkout(
        lines: &[&str],
    ) -> Result<(), String> {
        let step = Self::step_section(lines, "Checkout fixed KatanA reference")?;
        for expected in [
            "uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
            "repository: HiroyukiFuruno/KatanA",
        ] {
            if !step.iter().any(|line| line.trim().starts_with(expected)) {
                return Err(format!("fixed KatanA checkout is missing `{expected}`"));
            }
        }
        if !step
            .iter()
            .any(|line| line.trim() == format!("ref: {FIXED_KATANA_SHA}"))
        {
            return Err("fixed KatanA checkout does not pin the required SHA".to_string());
        }
        Ok(())
    }
}
