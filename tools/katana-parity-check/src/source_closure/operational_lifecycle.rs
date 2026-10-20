use super::operational_cli::{CliOptions, usage};

pub(super) fn run_cli(args: &[String]) -> Result<(), String> {
    let Some(operation) = args.first().map(String::as_str) else {
        return Err(usage().into());
    };
    let options = CliOptions::parse(&args[1..])?;
    match operation {
        "capture-profile" => {
            options.reject_unknown(&["profile-id", "output-dir", "katana-repo", "run-id"])?;
            super::operational_profile_capture::capture_profile(&options)
        }
        "capture-provenance" => {
            options.reject_unknown(&[
                "output-dir",
                "katana-repo",
                "kle-repo",
                "kuc-repo",
                "user-input",
                "generator-schema",
                "source-universe",
                "requirement-source-aliases",
                "run-id",
            ])?;
            super::operational_provenance_capture::capture_provenance(&options)
        }
        "assemble-input" => {
            options.reject_unknown(&[
                "output-dir",
                "seed-manifest",
                "seed-path",
                "source-universe",
                "katana-repo",
                "run-id",
            ])?;
            super::operational_input_assembly::assemble_input(&options)
        }
        "validate-input" => {
            options.reject_unknown(&["input"])?;
            super::operational_input_assembly::validate_input(&options)
        }
        "materialize-closure" => {
            options.reject_unknown(&["input", "katana-repo", "canonical-root"])?;
            super::operational_input_assembly::materialize_closure(&options)
        }
        "emit-generic-annotation-catalog" => {
            options.reject_unknown(&["input", "source-closure", "branches", "output"])?;
            super::operational_generic_mapping::emit_annotation_catalog(&options)
        }
        "enrich-leaf-manifest" => {
            options.reject_unknown(&[
                "input",
                "source-closure",
                "branches",
                "mapping",
                "output",
            ])?;
            super::operational_generic_mapping::enrich_leaf_manifest(&options)
        }
        "emit-consumer-plan" => {
            options.reject_unknown(&[
                "input",
                "source-closure",
                "branches",
                "mapping",
                "profile-id",
                "run-id",
                "output",
            ])?;
            super::operational_generic_mapping::emit_consumer_plan(&options)
        }
        "validate-consumer-artifact" => {
            options.reject_unknown(&[
                "input",
                "source-closure",
                "branches",
                "mapping",
                "profile-id",
                "run-id",
                "artifact-dir",
            ])?;
            super::operational_generic_mapping::validate_consumer_artifact(&options)
        }
        "join-consumer-artifacts" => {
            options.reject_unknown(&[
                "input",
                "source-closure",
                "branches",
                "mapping",
                "run-id",
                "macos-artifact",
                "windows-artifact",
                "linux-artifact",
                "output-dir",
            ])?;
            super::operational_generic_mapping::join_consumer_artifacts(&options)
        }
        "audit-requirement-bindings" => {
            options.reject_unknown(&["katana-repo", "output"])?;
            super::requirement_diagnostic::audit_requirement_bindings(&options)
        }
        _ => Err(format!(
            "unknown source-closure operation: {operation}
{}",
            usage()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::run_cli;

    #[test]
    fn assemble_input_accepts_the_required_source_universe_option() {
        let arguments = vec![
            "assemble-input".to_string(),
            "--source-universe".to_string(),
            "source-universe.md".to_string(),
        ];

        assert!(matches!(
            run_cli(&arguments),
            Err(error) if error.contains("missing required option --output-dir")
        ));
    }

    #[test]
    fn requirement_binding_diagnostic_dispatches_with_its_required_options() {
        let arguments = vec![
            "audit-requirement-bindings".to_string(),
            "--katana-repo".to_string(),
            "/tmp/katana".to_string(),
        ];

        assert!(matches!(
            run_cli(&arguments),
            Err(error) if error.contains("missing required option --output")
        ));
    }

    #[test]
    fn leaf_enrichment_requires_immutable_source_artifacts() {
        let arguments = vec![
            "enrich-leaf-manifest".to_string(),
            "--input".to_string(),
            "leaf-manifest.json".to_string(),
        ];

        assert!(matches!(
            run_cli(&arguments),
            Err(error) if error.contains("missing required option --source-closure")
        ));
    }

    #[test]
    fn annotation_catalog_requires_the_source_closure_artifact() {
        let arguments = vec![
            "emit-generic-annotation-catalog".to_string(),
            "--input".to_string(),
            "leaf-manifest.json".to_string(),
        ];

        assert!(matches!(
            run_cli(&arguments),
            Err(error) if error.contains("missing required option --source-closure")
        ));
    }

    #[test]
    fn consumer_plan_requires_an_immutable_output() {
        let arguments = vec!["emit-consumer-plan".to_string()];
        assert!(
            matches!(run_cli(&arguments), Err(error) if error.contains("missing required option --output"))
        );
    }

    #[test]
    fn consumer_artifact_validation_requires_canonical_inputs() {
        let arguments = vec![
            "validate-consumer-artifact".to_string(),
            "--artifact-dir".to_string(),
            "artifact".to_string(),
        ];
        assert!(
            matches!(run_cli(&arguments), Err(error) if error.contains("missing required option --input"))
        );
    }
}
