use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::JUSTFILE;

impl ReleaseGateAudit {
    pub(crate) fn validate_source_closure_input_recipes() -> Result<(), String> {
        Self::validate_source_closure_input_recipes_from_lines(
            &JUSTFILE.lines().collect::<Vec<_>>(),
        )
    }

    pub(crate) fn validate_source_closure_input_recipes_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        let provenance = recipe_body(lines, "source-closure-capture-provenance:")?;
        for expected in [
            "source-closure capture-provenance",
            "--source-universe \"{{REPO_ROOT}}/docs/v0-1-0-katana-editor-source-universe.md\"",
            "--requirement-source-aliases \"{{REPO_ROOT}}/docs/v0-1-0-editor-requirement-source-aliases.json\"",
        ] {
            if !provenance.contains(expected) {
                return Err(format!(
                    "source-closure-capture-provenance recipe is missing `{expected}`"
                ));
            }
        }

        let assemble = recipe_body(lines, "source-closure-assemble-input:")?;
        for expected in [
            "--seed-manifest \"{{REPO_ROOT}}/docs/v0-1-0-source-closure-roots.json\"",
            "--source-universe \"{{REPO_ROOT}}/docs/v0-1-0-katana-editor-source-universe.md\"",
            "--katana-repo \"{{KATANA_REPO}}\"",
        ] {
            if !assemble.contains(expected) {
                return Err(format!(
                    "source-closure-assemble-input recipe is missing `{expected}`"
                ));
            }
        }
        if assemble.contains("--seed-path") {
            return Err("source-closure-assemble-input recipe bypasses the root manifest".into());
        }
        Ok(())
    }

    pub(crate) fn validate_source_closure_materialize_recipe() -> Result<(), String> {
        Self::validate_source_closure_materialize_recipe_from_lines(
            &JUSTFILE.lines().collect::<Vec<_>>(),
        )
    }

    pub(crate) fn validate_source_closure_materialize_recipe_from_lines(
        lines: &[&str],
    ) -> Result<(), String> {
        let recipe = recipe_body(lines, "source-closure-materialize:")?;
        for expected in [
            "source-closure materialize-closure",
            "--input \"{{SOURCE_CLOSURE_INPUT}}\"",
            "--katana-repo \"{{KATANA_REPO}}\"",
            "--canonical-root \"artifacts\"",
        ] {
            if !recipe.contains(expected) {
                return Err(format!(
                    "source-closure-materialize recipe is missing `{expected}`"
                ));
            }
        }
        if recipe.contains("--artifact-dir") {
            return Err(
                "source-closure-materialize recipe uses unsupported `--artifact-dir`".to_string(),
            );
        }
        Ok(())
    }
}

fn recipe_body(lines: &[&str], header: &str) -> Result<String, String> {
    let start = lines
        .iter()
        .position(|line| line.starts_with(header))
        .ok_or_else(|| format!("Justfile is missing {header} recipe"))?;
    Ok(lines
        .iter()
        .skip(start + 1)
        .take_while(|line| line.starts_with("    "))
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join(" "))
}
