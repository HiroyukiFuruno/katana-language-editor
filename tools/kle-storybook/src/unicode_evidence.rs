use serde_json::Value;

pub(crate) const REQUIRED_STAR_SCALARS: &str = "[11088,65039]";
const STAR_CODEPOINT: u64 = 11088;
const VS16_CODEPOINT: u64 = 65039;

pub(crate) struct UnicodeEvidenceValidator;

impl UnicodeEvidenceValidator {
    pub(crate) fn validate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let evidence: Value = serde_json::from_slice(bytes)?;
        let root = evidence
            .as_object()
            .ok_or("KUC Unicode evidence must be a JSON object")?;
        Self::validate_schema(root)?;
        Self::validate_graphemes(root)?;
        Self::validate_ime(root)?;
        Self::validate_caret(root)?;
        Self::validate_hit_tests(root)?;
        Self::validate_star(root)
    }

    fn validate_schema(
        root: &serde_json::Map<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if root.get("schema").and_then(Value::as_str) != Some("kuc.unicode-color-glyph-evidence")
            || root.get("schema_version").and_then(Value::as_u64) != Some(1)
        {
            return Err("KUC Unicode evidence schema is invalid".into());
        }
        Ok(())
    }

    fn validate_graphemes(
        root: &serde_json::Map<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !required_array(root, "graphemes")?
            .iter()
            .any(has_exact_star_scalars)
        {
            return Err(
                format!("KUC Unicode evidence is missing `{REQUIRED_STAR_SCALARS}`").into(),
            );
        }
        Ok(())
    }

    fn validate_ime(
        root: &serde_json::Map<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ime = required_object(root, "ime")?;
        if ime.get("preedit_event_seen").and_then(Value::as_bool) != Some(true)
            || ime.get("commit_event_seen").and_then(Value::as_bool) != Some(true)
            || required_array(ime, "preedit_scalar_sequence")?.is_empty()
            || required_array(ime, "commit_scalar_sequence")?.is_empty()
        {
            return Err("KUC Unicode evidence IME events are incomplete".into());
        }
        Ok(())
    }

    fn validate_caret(
        root: &serde_json::Map<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        require_positive_bounds(required_object(root, "caret")?, "caret")
    }

    fn validate_hit_tests(
        root: &serde_json::Map<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !required_array(root, "hit_tests")?.iter().any(|hit| {
            hit.as_object()
                .and_then(|object| object.get("target"))
                .and_then(Value::as_str)
                == Some("star")
        }) {
            return Err("KUC Unicode evidence lacks a star hit-test".into());
        }
        Ok(())
    }

    fn validate_star(
        root: &serde_json::Map<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let star = required_object(root, "star")?;
        require_positive_bounds(star, "star")?;
        if star
            .get("chromatic_pixel_count")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == 0
        {
            return Err("KUC Unicode evidence star is not a color glyph".into());
        }
        Ok(())
    }
}

fn required_object<'a>(
    root: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a serde_json::Map<String, Value>, Box<dyn std::error::Error>> {
    root.get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("KUC Unicode evidence `{key}` must be an object").into())
}

fn required_array<'a>(
    root: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a Vec<Value>, Box<dyn std::error::Error>> {
    root.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("KUC Unicode evidence `{key}` must be an array").into())
}

fn has_exact_star_scalars(grapheme: &Value) -> bool {
    grapheme
        .as_object()
        .and_then(|object| object.get("scalar_sequence"))
        .and_then(Value::as_array)
        .is_some_and(|scalars| {
            scalars
                .iter()
                .map(Value::as_u64)
                .eq([Some(STAR_CODEPOINT), Some(VS16_CODEPOINT)])
        })
}

fn require_positive_bounds(
    value: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let bounds = required_object(value, "bounds")?;
    for dimension in ["width", "height"] {
        if bounds.get(dimension).and_then(Value::as_u64).unwrap_or(0) == 0 {
            return Err(format!("KUC Unicode evidence `{key}` has invalid {dimension}").into());
        }
    }
    Ok(())
}
