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
}
