use std::collections::BTreeMap;

pub(super) const PROFILE_IDS: [&str; 3] = ["macos-latest", "windows-latest", "ubuntu-latest"];
pub(super) const PROFILE_SOURCE_PREFIX: &str = "profiles";
pub(super) const PROVENANCE_PREFIX: &str = "provenance";

pub(super) fn usage() -> &'static str {
    "usage: katana-parity-check source-closure <capture-profile|capture-provenance|assemble-input|validate-input|materialize-closure|audit-requirement-bindings> [options]"
}

pub(super) struct CliOptions {
    pub(super) values: BTreeMap<String, Vec<String>>,
}

impl CliOptions {
    pub(super) fn parse(args: &[String]) -> Result<Self, String> {
        let mut values = BTreeMap::<String, Vec<String>>::new();
        let mut index = 0;
        while index < args.len() {
            let flag = args[index].as_str();
            if !flag.starts_with("--") {
                return Err(format!("unexpected argument: {flag}"));
            }
            let key = flag.trim_start_matches('-').to_string();
            if key.is_empty() || index + 1 >= args.len() || args[index + 1].starts_with("--") {
                return Err(format!("option {flag} requires a value"));
            }
            values.entry(key).or_default().push(args[index + 1].clone());
            index += 2;
        }
        Ok(Self { values })
    }

    pub(super) fn one(&self, key: &str) -> Result<&str, String> {
        let values = self
            .values
            .get(key)
            .ok_or_else(|| format!("missing required option --{key}"))?;
        if values.len() != 1 {
            return Err(format!("option --{key} must occur exactly once"));
        }
        Ok(&values[0])
    }

    pub(super) fn many(&self, key: &str) -> &[String] {
        self.values.get(key).map_or(&[], Vec::as_slice)
    }

    pub(super) fn reject_unknown(&self, allowed: &[&str]) -> Result<(), String> {
        if let Some(key) = self
            .values
            .keys()
            .find(|key| !allowed.iter().any(|allowed| allowed == key))
        {
            return Err(format!("unknown source-closure option: --{key}"));
        }
        Ok(())
    }
}
