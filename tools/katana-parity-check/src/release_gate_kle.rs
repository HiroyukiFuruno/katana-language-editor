use crate::release_gate::ReleaseGateAudit;
use crate::release_gate_sources::JUSTFILE;

impl ReleaseGateAudit {
    pub(crate) fn validate_kle_release_gate() -> Result<(), String> {
        let lines: Vec<&str> = JUSTFILE.lines().collect();
        Self::validate_kle_release_gate_from_lines(&lines)
    }

    pub(crate) fn validate_kle_release_gate_from_lines(lines: &[&str]) -> Result<(), String> {
        let motion = recipe_body_lines(lines, "storybook-motion-artifact-gate")?;
        require_recipe_command(
            &motion,
            "{{CARGO}} test -p kle-storybook --locked storybook_motion_artifact_uses_the_exact_kuc_plan_without_idle_frames -- --test-threads=1",
            "storybook-motion-artifact-gate",
        )?;
        require_recipe_command(
            &recipe_body_lines(lines, "storybook-motion-artifact")?,
            "output_dir=\"{{STORYBOOK_MOTION_ARTIFACT_OUTPUT}}\"; if test -z \"$output_dir\"; then output_dir=\"target/acceptance/kle-storybook-motion-artifact-$(date -u +%Y%m%dT%H%M%SZ)-$$\"; fi; {{CARGO}} run --locked -p kle-storybook -- --motion-artifact --artifact-output \"$output_dir\"",
            "storybook-motion-artifact",
        )?;

        let ast = recipe_body_lines(lines, "ast-lint")?;
        for command in [
            "{{CARGO}} test -j {{JOBS}} -p kle-linter ast_linter -- --nocapture",
            "{{CARGO}} test -j {{JOBS}} -p kle-linter --test ast_linter ast_linter_workspace_rules -- --nocapture",
        ] {
            require_recipe_command(&ast, command, "ast-lint")?;
        }
        Ok(())
    }
}

fn recipe_body_lines<'a>(lines: &'a [&'a str], recipe: &str) -> Result<Vec<&'a str>, String> {
    let header = format!("{recipe}:");
    let start = lines
        .iter()
        .position(|line| **line == header)
        .ok_or_else(|| format!("Justfile is missing {recipe} recipe"))?;
    let body = lines
        .iter()
        .skip(start + 1)
        .take_while(|line| line.starts_with("    ") || line.starts_with('\t') || line.is_empty())
        .copied()
        .collect::<Vec<_>>();
    body.iter()
        .any(|line| !line.trim().is_empty())
        .then_some(())
        .ok_or_else(|| format!("Justfile {recipe} recipe has no commands"))?;
    Ok(body)
}

fn require_recipe_command(body: &[&str], command: &str, recipe: &str) -> Result<(), String> {
    body.iter()
        .any(|line| line.trim() == command)
        .then_some(())
        .ok_or_else(|| format!("Justfile {recipe} recipe is missing `{command}`"))
}
