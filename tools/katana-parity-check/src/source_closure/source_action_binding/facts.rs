use std::collections::BTreeMap;

use super::super::artifact_model::SourceClosureArtifact;
use super::super::requirement_binding::{RequirementBinding, RequirementBindingResult};
use super::super::scan_state::{ActionConstruction, ActionDefinition, DispatchArm};
use super::span::parse_span;
use super::types::{
    IndexedConstruction, IndexedDefinition, IndexedDispatch, SourceActionFact,
    SourceActionUnresolved,
};

pub(super) fn source_file_index(
    source: &SourceClosureArtifact,
) -> Result<BTreeMap<String, String>, String> {
    let mut files = BTreeMap::new();
    for file in &source.files {
        if files
            .insert(file.path.clone(), file.sha256.clone())
            .is_some()
        {
            return Err(format!("duplicate source file path: {}", file.path));
        }
    }
    Ok(files)
}

pub(super) fn verify_requirement_bindings(
    requirements: &RequirementBindingResult,
    source_files: &BTreeMap<String, String>,
) -> Result<(), String> {
    for binding in &requirements.bindings {
        let source_sha256 = source_files.get(&binding.source_path).ok_or_else(|| {
            format!(
                "requirement binding source path is absent from closure: {}",
                binding.source_path
            )
        })?;
        if source_sha256 != &binding.source_sha256 {
            return Err(format!(
                "requirement binding source SHA mismatch: {}",
                binding.source_path
            ));
        }
    }
    requirements.validate_fingerprint()
}

pub(super) fn index_definitions(
    definitions: &[ActionDefinition],
    source_files: &BTreeMap<String, String>,
) -> BTreeMap<String, Result<IndexedDefinition, String>> {
    let mut indexed = BTreeMap::new();
    for definition in definitions {
        let fact = parse_fact(&definition.file, &definition.variant_span, source_files);
        let value = match fact {
            Ok(fact) => Ok(IndexedDefinition { fact }),
            Err(error) => Err(error),
        };
        match indexed.get(&definition.variant) {
            Some(Ok(_)) | Some(Err(_)) => {
                indexed.insert(
                    definition.variant.clone(),
                    Err("multiple AppAction definitions are ambiguous".into()),
                );
            }
            None => {
                indexed.insert(definition.variant.clone(), value);
            }
        }
    }
    indexed
}

pub(super) fn construction_facts(
    construction: &ActionConstruction,
    definitions: &BTreeMap<String, Result<IndexedDefinition, String>>,
    source_files: &BTreeMap<String, String>,
) -> Result<IndexedConstruction, String> {
    if let Some(reason) = &construction.unresolved_reason {
        return Err(reason.clone());
    }
    let variant = construction.variant.clone().ok_or_else(|| {
        format!(
            "unresolved construction has no AppAction variant: {}",
            construction.file
        )
    })?;
    if construction.enum_name != "AppAction" {
        return Err(format!(
            "unresolved construction enum `{}` is not AppAction",
            construction.enum_name
        ));
    }
    if !definitions.contains_key(&variant) {
        return Err("construction has no indexed AppAction definition".into());
    }
    let fact = parse_fact(&construction.file, &construction.span, source_files)?;
    let span = parse_span(&fact.span)?;
    Ok(IndexedConstruction {
        variant,
        path: construction.file.clone(),
        fact,
        span,
    })
}

pub(super) fn dispatch_fact(
    arm: &DispatchArm,
    source_files: &BTreeMap<String, String>,
) -> Result<IndexedDispatch, String> {
    let fact = parse_fact(&arm.file, &arm.arm_span, source_files)?;
    let variant_pattern = parse_fact(&arm.file, &arm.variant_pattern_span, source_files)?;
    Ok(IndexedDispatch {
        action_variant: arm.action_variant.clone(),
        path: arm.file.clone(),
        span: arm.arm_span.clone(),
        fact,
        variant_pattern,
        unresolved_reasons: arm.unresolved_reasons.clone(),
    })
}

fn parse_fact(
    path: &str,
    span: &str,
    source_files: &BTreeMap<String, String>,
) -> Result<SourceActionFact, String> {
    let sha256 = source_files
        .get(path)
        .ok_or_else(|| format!("source fact file is absent from closure: {path}"))?;
    let parsed = parse_span(span)?;
    if parsed.file != path {
        return Err(format!(
            "source fact span path mismatch: {path} != {}",
            parsed.file
        ));
    }
    Ok(SourceActionFact {
        path: path.into(),
        sha256: sha256.clone(),
        span: span.into(),
    })
}

pub(super) fn validate_fact_span(fact: &SourceActionFact) -> Result<(), String> {
    let parsed = parse_span(&fact.span)?;
    if parsed.file != fact.path {
        return Err(format!(
            "source fact span path mismatch: {} != {}",
            fact.path, parsed.file
        ));
    }
    Ok(())
}

pub(super) fn construction_is_within_branch(
    construction: &IndexedConstruction,
    binding: &RequirementBinding,
) -> bool {
    construction.span.file == binding.source_path
        && construction.span.start_line >= binding.span_start_line
        && construction.span.end_line <= binding.span_end_line
}

pub(super) fn binding_id(binding: &RequirementBinding) -> String {
    format!(
        "{}:{}:{}",
        binding.requirement_id, binding.source_path, binding.branch_id
    )
}

pub(super) fn unresolved_construction(
    construction: &ActionConstruction,
    reason: &str,
) -> SourceActionUnresolved {
    SourceActionUnresolved {
        kind: "unresolved_construction".into(),
        source_edge_kind: None,
        source_edge_detail: None,
        source_edge_resolution: None,
        action_variant: construction.variant.clone(),
        path: construction.file.clone(),
        span: Some(construction.span.clone()),
        reason: reason.into(),
    }
}
