use std::{fs, path::Path};

const CONTEXT_MENU_HOST_E2E_RECIPE: &str = "katana-host-e2e-context-menu";
const PARITY_RECIPE: &str = "katana-parity-check";
const HOST_E2E_MANIFEST: &str = "--manifest-path tools/katana-host-e2e/Cargo.toml";
const HOST_E2E_TARGET: &str = "--test context_menu_input";
const HOST_E2E_LOCKFILE: &str = "--locked";
const FIRST_PARITY_COMMAND: &str = "just katana-host-e2e-context-menu";

pub(super) fn validate_just_gate(root: &Path) -> Result<(), String> {
    let source = fs::read_to_string(root.join("Justfile"))
        .map_err(|_| "missing Justfile for ContextMenu host E2E gate".to_string())?;
    let host_e2e = recipe_body(&source, CONTEXT_MENU_HOST_E2E_RECIPE)?;
    require_markers(
        &host_e2e,
        &[
            ("ContextMenu host E2E manifest", HOST_E2E_MANIFEST),
            ("ContextMenu host E2E test target", HOST_E2E_TARGET),
            ("ContextMenu host E2E lockfile", HOST_E2E_LOCKFILE),
        ],
        "Justfile ContextMenu host E2E recipe",
    )?;
    let parity_check = recipe_body(&source, PARITY_RECIPE)?;
    let command = parity_check
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty());
    (command == Some(FIRST_PARITY_COMMAND))
        .then_some(())
        .ok_or_else(|| {
            "Justfile katana-parity-check must run ContextMenu host E2E before parity auditing"
                .into()
        })
}

fn recipe_body(source: &str, recipe: &str) -> Result<String, String> {
    let header = format!("{recipe}:");
    let start = source
        .lines()
        .position(|line| line == header)
        .ok_or_else(|| format!("Justfile is missing {recipe} recipe"))?;
    let body = source
        .lines()
        .skip(start + 1)
        .take_while(|line| line.starts_with(' ') || line.starts_with('\t') || line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    (!body.trim().is_empty())
        .then_some(body)
        .ok_or_else(|| format!("Justfile {recipe} recipe has no commands"))
}

fn require_markers(source: &str, requirements: &[(&str, &str)], scope: &str) -> Result<(), String> {
    for &(description, marker) in requirements {
        if !source.contains(marker) {
            return Err(format!("{scope} is missing {description}: {marker:?}"));
        }
    }
    Ok(())
}
