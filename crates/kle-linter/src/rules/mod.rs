mod action_host_boundary;
mod architecture;
mod attributes;
mod authoring_host_boundary;
mod clipboard_host_acquisition;
mod egui_duplication;
mod file_length;
mod function_length;
mod host_projection_ownership;
mod kuc_text_runtime;
mod kuc_text_runtime_context_menu;
mod lazy_code;
mod legacy_config_generic_fields;
mod manifest_boundary;
mod manifest_reader;
mod method_calls;
mod nesting_depth;
mod prohibited_color_literal;
mod prohibited_color_literal_patterns;
mod pub_free_fn;
mod search_host_boundary;
mod source_address_ownership_boundary;
mod status_diagnostics_ownership_boundary;
mod storybook_contract;
mod tab_strip_ownership_boundary;
mod ui_dependency_policy;
mod user_visible_string_literals;

use crate::diagnostics::{KleLintError, Violation};
use crate::workspace::WorkspaceModel;
use action_host_boundary::ActionHostBoundaryRule;
use architecture::ArchitectureRule;
use attributes::ProhibitedAttributeRule;
use authoring_host_boundary::AuthoringHostBoundaryRule;
use clipboard_host_acquisition::ClipboardHostAcquisitionRule;
use file_length::FileLengthRule;
use function_length::FunctionLengthRule;
use host_projection_ownership::HostProjectionOwnershipRule;
use kuc_text_runtime::KucTextRuntimeRule;
use kuc_text_runtime_context_menu::ContextMenuRuntimeRule;
use lazy_code::LazyCodeRule;
use legacy_config_generic_fields::LegacyConfigGenericFieldsRule;
use method_calls::ProhibitedMethodRule;
use nesting_depth::NestingDepthRule;
use prohibited_color_literal::ProhibitedColorLiteralRule;
use pub_free_fn::PublicFreeFunctionRule;
use search_host_boundary::SearchHostBoundaryRule;
use source_address_ownership_boundary::SourceAddressOwnershipBoundaryRule;
use status_diagnostics_ownership_boundary::StatusDiagnosticsOwnershipBoundaryRule;
use storybook_contract::StorybookContractRule;
use tab_strip_ownership_boundary::TabStripOwnershipBoundaryRule;
use user_visible_string_literals::UserVisibleStringLiteralRule;

type RuleCheck = fn(&WorkspaceModel) -> Result<Vec<Violation>, KleLintError>;
const RULE_COUNT: usize = 22;

pub struct RuleRunner;

impl RuleRunner {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for rule in Self::rules() {
            violations.extend(rule(workspace)?);
        }
        Ok(violations)
    }

    fn rules() -> [RuleCheck; RULE_COUNT] {
        [
            FileLengthRule::check,
            FunctionLengthRule::check,
            HostProjectionOwnershipRule::check,
            NestingDepthRule::check,
            PublicFreeFunctionRule::check,
            ProhibitedMethodRule::check,
            LazyCodeRule::check,
            LegacyConfigGenericFieldsRule::check,
            ProhibitedAttributeRule::check,
            ProhibitedColorLiteralRule::check,
            UserVisibleStringLiteralRule::check,
            StorybookContractRule::check,
            ArchitectureRule::check,
            ActionHostBoundaryRule::check,
            KucTextRuntimeRule::check,
            ContextMenuRuntimeRule::check,
            ClipboardHostAcquisitionRule::check,
            SearchHostBoundaryRule::check,
            AuthoringHostBoundaryRule::check,
            SourceAddressOwnershipBoundaryRule::check,
            StatusDiagnosticsOwnershipBoundaryRule::check,
            TabStripOwnershipBoundaryRule::check,
        ]
    }
}
