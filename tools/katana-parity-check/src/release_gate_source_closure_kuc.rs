use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::WORKSPACE_MANIFEST;

const KUC_DEPENDENCY_CHECKOUT: &str = "Checkout KUC dependency source";
const KUC_CHECKOUT_EVIDENCE: &[&str] = &[
    "uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
    "repository: HiroyukiFuruno/katana-ui-core",
    "path: katana-ui-core",
];
const SOURCE_CLOSURE_KUC_JOBS: &[&str] = &[
    "capture-profile",
    "assemble-validate-materialize",
    "native-host-e2e",
];

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_kuc_checkout_contract(
        lines: &[&str],
    ) -> Result<(), String> {
        for job_name in SOURCE_CLOSURE_KUC_JOBS {
            let job = Self::job_section(lines, job_name)?;
            validate_kuc_checkout(job, job_name)?;
        }
        Ok(())
    }
}

fn validate_kuc_checkout(lines: &[&str], job_name: &str) -> Result<(), String> {
    let step = ReleaseGateAudit::step_section(lines, KUC_DEPENDENCY_CHECKOUT).map_err(|_| {
        format!("source-closure {job_name} job is missing `{KUC_DEPENDENCY_CHECKOUT}` step")
    })?;
    let expected_ref = expected_kuc_dependency_ref()?;
    for expected in KUC_CHECKOUT_EVIDENCE
        .iter()
        .copied()
        .chain(std::iter::once(expected_ref.as_str()))
    {
        if !step
            .iter()
            .any(|line| matches_checkout_evidence(line, expected))
        {
            return Err(format!(
                "source-closure {job_name} job KUC dependency checkout is missing `{expected}`"
            ));
        }
    }
    Ok(())
}

fn expected_kuc_dependency_ref() -> Result<String, String> {
    let manifest: toml::Value = toml::from_str(WORKSPACE_MANIFEST)
        .map_err(|error| format!("failed to parse workspace manifest: {error}"))?;
    let version = manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("dependencies"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("katana-ui-core"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("version"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| "workspace katana-ui-core dependency version is missing".to_string())?;
    let release_version = version.strip_prefix('=').unwrap_or(version);
    Ok(format!("ref: v{release_version}"))
}

fn matches_checkout_evidence(line: &str, expected: &str) -> bool {
    let line = line.trim();
    if expected.starts_with("uses:") {
        return line.starts_with(expected);
    }
    line == expected
}
