use crate::capability_manifest::HostEffectKind;

use super::capability_manifest_host_e2e_syntax::ReachableEvidence;

const CONTEXT_AUTHORING_CASE_COUNT: &str = "30";
const TOOLBAR_AUTHORING_CASE_COUNT: &str = "27";
const FAIL_CLOSED_REASON: &str = "intentionally not executed by host E2E";

pub(super) struct HostEffectContractValidator;

impl HostEffectContractValidator {
    pub(super) fn validate(
        source: &ReachableEvidence,
        effect: HostEffectKind,
    ) -> Result<(), String> {
        Self::validate_required_paths(source, effect)?;
        Self::validate_document_effect(source, effect)?;
        Self::validate_fail_closed_external_intent(source, effect)
    }

    fn validate_required_paths(
        source: &ReachableEvidence,
        effect: HostEffectKind,
    ) -> Result<(), String> {
        for required in Self::required_paths(effect)? {
            if !source.path_starts_with(required) {
                return Err(format!(
                    "KLE host E2E source lacks concrete {} proof: {}",
                    effect.description(),
                    required
                ));
            }
        }
        if matches!(
            effect,
            HostEffectKind::ContextAuthoring | HostEffectKind::ToolbarAuthoring
        ) && (!source.methods.contains("len")
            || !Self::has_authoring_inventory_count(source, effect))
        {
            return Err(format!(
                "KLE host E2E source lacks exact {} inventory cardinality",
                effect.description()
            ));
        }
        Ok(())
    }

    fn required_paths(effect: HostEffectKind) -> Result<&'static [&'static str], String> {
        match effect {
            HostEffectKind::ContextAuthoring => Ok(&["ContextMenuPhysicalInput::case_specs"]),
            HostEffectKind::ToolbarAuthoring => Ok(&["CommandChromePhysicalInput::case_specs"]),
            HostEffectKind::Save => Ok(&["EditorAction::SaveDocument", "fs::read_to_string"]),
            HostEffectKind::Format => Ok(&["EditorAction::FormatDocument", "fs::read_to_string"]),
            HostEffectKind::ExternalIngestFailClosed => Ok(&[
                "EditorAction::IngestImageFile",
                "EditorAction::IngestClipboardImage",
            ]),
            HostEffectKind::Missing => Err("missing actual KLE host-effect coverage".to_string()),
        }
    }

    fn has_authoring_inventory_count(source: &ReachableEvidence, effect: HostEffectKind) -> bool {
        let expected = match effect {
            HostEffectKind::ContextAuthoring => CONTEXT_AUTHORING_CASE_COUNT,
            HostEffectKind::ToolbarAuthoring => TOOLBAR_AUTHORING_CASE_COUNT,
            _ => return false,
        };
        source.integer_literals.contains(expected)
    }

    fn validate_document_effect(
        source: &ReachableEvidence,
        effect: HostEffectKind,
    ) -> Result<(), String> {
        if Self::has_concrete_document_effect(source, effect) {
            Ok(())
        } else {
            Err(format!(
                "KLE host E2E source lacks concrete {} document/state assertion",
                effect.description()
            ))
        }
    }

    fn has_concrete_document_effect(source: &ReachableEvidence, effect: HostEffectKind) -> bool {
        let buffer_changed =
            source.fields.contains("buffer") && source.binary_operators.contains("!=");
        match effect {
            HostEffectKind::Format => buffer_changed,
            HostEffectKind::ContextAuthoring
            | HostEffectKind::ToolbarAuthoring
            | HostEffectKind::Save
            | HostEffectKind::ExternalIngestFailClosed => {
                buffer_changed && source.fields.contains("is_dirty")
            }
            HostEffectKind::Missing => false,
        }
    }

    fn validate_fail_closed_external_intent(
        source: &ReachableEvidence,
        effect: HostEffectKind,
    ) -> Result<(), String> {
        if effect != HostEffectKind::ExternalIngestFailClosed
            || source.contains_text(FAIL_CLOSED_REASON)
        {
            Ok(())
        } else {
            Err(
                "KLE host E2E external ingest evidence does not prove fail-closed rejection"
                    .to_string(),
            )
        }
    }
}
