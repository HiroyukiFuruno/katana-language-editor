use serde::Deserialize;

const REQUIRED_SCHEMA_VERSION: &str = "1";
const REQUIRED_LEAF_IDS: [&str; 2] = ["replace.current", "replace.all"];
const REQUIRED_ANCHOR: &str =
    "docs/v0-1-0-editor-requirements.md#explicit-user-mandated-extensions";
const REQUIRED_KUC_COMPONENT: &str = "SearchStrip";
const REQUIRED_ACTION: &str = "AppAction::ReplaceText";
const REQUIRED_DISPATCH: &str = "crates/katana-ui/src/app/action/dispatch.rs";
const REQUIRED_HANDLER: &str = "DocumentEditOps::handle_replace_text";
const REQUIRED_PROHIBITIONS: [&str; 5] = [
    "kle_query_state",
    "kle_match_engine",
    "kle_range_or_byte_conversion",
    "kle_content_mutation",
    "katana_direct_ui_claim",
];

#[cfg(test)]
pub(super) const CANONICAL_USER_MANDATED_LEAVES: &[u8] = br#"{
  "schema_version": "1",
  "extensions": [
    {
      "leaf_id": "replace.current",
      "requirement_anchor": "docs/v0-1-0-editor-requirements.md#explicit-user-mandated-extensions",
      "kuc_component": "SearchStrip",
      "katana_direct_ui_origin": false,
      "katana_host_route": {
        "action": "AppAction::ReplaceText",
        "dispatch": "crates/katana-ui/src/app/action/dispatch.rs",
        "handler": "DocumentEditOps::handle_replace_text"
      },
      "prohibitions": [
        "kle_query_state",
        "kle_match_engine",
        "kle_range_or_byte_conversion",
        "kle_content_mutation",
        "katana_direct_ui_claim"
      ]
    },
    {
      "leaf_id": "replace.all",
      "requirement_anchor": "docs/v0-1-0-editor-requirements.md#explicit-user-mandated-extensions",
      "kuc_component": "SearchStrip",
      "katana_direct_ui_origin": false,
      "katana_host_route": {
        "action": "AppAction::ReplaceText",
        "dispatch": "crates/katana-ui/src/app/action/dispatch.rs",
        "handler": "DocumentEditOps::handle_replace_text"
      },
      "prohibitions": [
        "kle_query_state",
        "kle_match_engine",
        "kle_range_or_byte_conversion",
        "kle_content_mutation",
        "katana_direct_ui_claim"
      ]
    }
  ]
}
"#;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UserMandatedLeaves {
    schema_version: String,
    extensions: Vec<UserMandatedExtension>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UserMandatedExtension {
    leaf_id: String,
    requirement_anchor: String,
    kuc_component: String,
    katana_direct_ui_origin: bool,
    katana_host_route: KatanaHostRoute,
    prohibitions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KatanaHostRoute {
    action: String,
    dispatch: String,
    handler: String,
}

pub(super) fn validate(bytes: &[u8]) -> Result<(), String> {
    let manifest: UserMandatedLeaves = serde_json::from_slice(bytes)
        .map_err(|error| format!("invalid user-mandated extension JSON: {error}"))?;
    if manifest.schema_version != REQUIRED_SCHEMA_VERSION {
        return Err("user-mandated extension schema_version must be exactly \"1\"".into());
    }
    if manifest.extensions.len() != REQUIRED_LEAF_IDS.len() {
        return Err(
            "user-mandated extensions must contain exactly replace.current and replace.all".into(),
        );
    }
    for (extension, required_id) in manifest.extensions.iter().zip(REQUIRED_LEAF_IDS) {
        validate_extension(extension, required_id)?;
    }
    Ok(())
}

fn validate_extension(extension: &UserMandatedExtension, required_id: &str) -> Result<(), String> {
    if extension.leaf_id != required_id {
        return Err(format!(
            "user-mandated extension must be canonically ordered as {required_id}"
        ));
    }
    if extension.requirement_anchor != REQUIRED_ANCHOR {
        return Err(format!(
            "{required_id} has an unexpected requirement anchor"
        ));
    }
    if extension.kuc_component != REQUIRED_KUC_COMPONENT {
        return Err(format!(
            "{required_id} must use the generic KUC SearchStrip"
        ));
    }
    if extension.katana_direct_ui_origin {
        return Err(format!(
            "{required_id} must not claim a KatanA direct UI origin"
        ));
    }
    let route = &extension.katana_host_route;
    if route.action != REQUIRED_ACTION
        || route.dispatch != REQUIRED_DISPATCH
        || route.handler != REQUIRED_HANDLER
    {
        return Err(format!(
            "{required_id} must retain only the single-span KatanA ReplaceText resolver"
        ));
    }
    let prohibitions = extension.prohibitions.iter().map(String::as_str);
    if !prohibitions.eq(REQUIRED_PROHIBITIONS) {
        return Err(format!(
            "{required_id} has an invalid KLE ownership prohibition set"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CANONICAL_USER_MANDATED_LEAVES, validate};

    fn mutated_manifest(
        mutator: impl FnOnce(&mut serde_json::Value),
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut manifest = serde_json::from_slice(CANONICAL_USER_MANDATED_LEAVES)?;
        mutator(&mut manifest);
        Ok(serde_json::to_vec(&manifest)?)
    }

    #[test]
    fn accepts_the_only_registered_user_mandated_extensions()
    -> Result<(), Box<dyn std::error::Error>> {
        validate(CANONICAL_USER_MANDATED_LEAVES)?;
        Ok(())
    }

    #[test]
    fn validates_the_repository_user_mandated_extension_contract()
    -> Result<(), Box<dyn std::error::Error>> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("docs/v0-1-0-user-mandated-leaves.json");
        validate(&std::fs::read(path)?)?;
        Ok(())
    }

    #[test]
    fn rejects_a_direct_katana_ui_claim() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = mutated_manifest(|manifest| {
            manifest["extensions"][0]["katana_direct_ui_origin"] = serde_json::json!(true);
        })?;
        assert!(matches!(
            validate(&bytes),
            Err(error) if error.contains("must not claim a KatanA direct UI origin")
        ));
        Ok(())
    }

    #[test]
    fn rejects_a_replace_all_host_route_claim() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = mutated_manifest(|manifest| {
            manifest["extensions"][1]["katana_host_route"]["action"] =
                serde_json::json!("AppAction::ReplaceAll");
        })?;
        assert!(matches!(
            validate(&bytes),
            Err(error) if error.contains("single-span KatanA ReplaceText resolver")
        ));
        Ok(())
    }

    #[test]
    fn rejects_an_unregistered_or_out_of_order_extension() -> Result<(), Box<dyn std::error::Error>>
    {
        let bytes = mutated_manifest(|manifest| {
            manifest["extensions"][0]["leaf_id"] = serde_json::json!("replace.everything");
        })?;
        assert!(matches!(
            validate(&bytes),
            Err(error) if error.contains("canonically ordered")
        ));
        Ok(())
    }
}
