use crate::{
    capability_manifest::{
        CapabilityOwner, KleActualFrameHarness, KleHostE2eEvidence, KleSourceLocator,
    },
    matrix::{EvidenceClassification, FeatureVerification},
};

pub(crate) struct DeclarationValidator;

impl DeclarationValidator {
    pub(crate) fn validate(feature: &FeatureVerification) -> Result<(), String> {
        let manifest = feature.manifest;
        Self::validate_required_manifest_values(feature)?;
        Self::validate_owners(manifest.owners)?;
        Self::validate_actual_input(feature)?;
        Self::validate_host_e2e(feature)?;
        Self::validate_classification(feature)
    }

    pub(crate) fn validate_distinct_selectors(
        features: &[FeatureVerification],
    ) -> Result<(), String> {
        for (index, feature) in features.iter().enumerate() {
            for prior in &features[..index] {
                let evidence = &feature.manifest.kle_actual_input;
                let prior_evidence = &prior.manifest.kle_actual_input;
                if evidence.selector == prior_evidence.selector {
                    return Err(format!(
                        "KLE actual-input selector is reused by feature {} and {}: {} ({} and {})",
                        prior.id,
                        feature.id,
                        evidence.selector,
                        prior_evidence.source_path,
                        evidence.source_path,
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_required_manifest_values(feature: &FeatureVerification) -> Result<(), String> {
        if feature.manifest.katana_sources.is_empty() {
            return Err("capability manifest has no KatanA source evidence".to_string());
        }
        if feature.manifest.owners.is_empty() {
            return Err("capability manifest has no ownership declaration".to_string());
        }
        Ok(())
    }

    fn validate_owners(owners: &[CapabilityOwner]) -> Result<(), String> {
        for owner in [
            CapabilityOwner::KucRuntime,
            CapabilityOwner::KleBinding,
            CapabilityOwner::KatanaHost,
        ] {
            if !owners.contains(&owner) {
                return Err(format!("capability manifest is missing owner: {owner:?}"));
            }
        }
        Ok(())
    }

    fn validate_actual_input(feature: &FeatureVerification) -> Result<(), String> {
        let evidence = &feature.manifest.kle_actual_input;
        if evidence.feature_id != feature.id {
            return Err(format!(
                "KLE actual-input evidence feature id mismatch: manifest {} feature {}",
                evidence.feature_id, feature.id
            ));
        }
        if evidence.source_path.is_empty() || evidence.selector.is_empty() {
            return Err("capability manifest has no KLE actual-input evidence".to_string());
        }
        if !evidence.selector.starts_with("public_show_") {
            return Err(format!(
                "KLE actual-input selector must exercise public EguiLanguageEditor::show: {}",
                evidence.selector
            ));
        }
        Self::validate_harness(&evidence.harness)
    }

    fn validate_harness(harness: &KleActualFrameHarness) -> Result<(), String> {
        if [
            harness.public_show_callsite,
            harness.raw_input_root,
            harness.raw_input_construction,
            harness.scenario_implementation,
        ]
        .iter()
        .any(Self::source_locator_is_incomplete)
            || harness.symbol.is_empty()
            || harness.call_path.is_empty()
        {
            return Err(
                "capability manifest has incomplete KLE actual-frame harness evidence".to_string(),
            );
        }
        Self::harness_call_path_is_valid(harness)
    }

    fn source_locator_is_incomplete(locator: &KleSourceLocator) -> bool {
        locator.source_path.is_empty() || locator.line == 0 || locator.marker.is_empty()
    }

    fn harness_call_path_is_valid(harness: &KleActualFrameHarness) -> Result<(), String> {
        let mut parts = harness.call_path.split("::");
        let valid = parts.next() == Some(harness.symbol)
            && parts.next().is_some_and(|method| !method.is_empty())
            && parts.next().is_none();
        valid.then_some(()).ok_or_else(|| {
            format!(
                "KLE actual-frame harness call path must be {}::<method>: {}",
                harness.symbol, harness.call_path
            )
        })
    }

    fn validate_host_e2e(feature: &FeatureVerification) -> Result<(), String> {
        let host_e2e = &feature.manifest.host_e2e;
        if host_e2e.test.target.is_empty()
            || host_e2e.test.source_path.is_empty()
            || host_e2e.test.selector.is_empty()
        {
            return Err("capability manifest has incomplete KLE host E2E evidence".to_string());
        }
        Self::host_target_matches_source(host_e2e)
    }

    fn host_target_matches_source(host_e2e: &KleHostE2eEvidence) -> Result<(), String> {
        let expected = format!("tools/katana-host-e2e/tests/{}.rs", host_e2e.test.target);
        (host_e2e.test.source_path == expected).then_some(()).ok_or_else(|| {
            format!(
                "KLE host E2E source must match its Cargo test target: expected {expected}, got {}",
                host_e2e.test.source_path
            )
        })
    }

    fn validate_classification(feature: &FeatureVerification) -> Result<(), String> {
        if matches!(
            feature.manifest.classification,
            EvidenceClassification::UserMandatedAdditional
        ) && feature.id != "find-replace-ui"
        {
            return Err(
                "user-mandated additional classification is only declared for find-replace-ui"
                    .to_string(),
            );
        }
        Ok(())
    }
}
