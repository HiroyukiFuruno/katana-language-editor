const ROOT_KEYS: &[&str] = &[
    "no_tty",
    "output",
    "pre-commit",
    "commit-msg",
    "pre-push",
    "post-merge",
];

pub(crate) struct ReleaseGateHookAudit;

impl ReleaseGateHookAudit {
    /* WHY: Lefthook の root override を許さず、監査対象を現在の固定 config に限定する。 */
    pub(crate) fn validate_pre_push_hook_sources(
        config: &str,
        strict_hook: &str,
        guard_hook: &str,
    ) -> Result<(), String> {
        let headers = validate_root_keys(config)?;
        let lines = config.lines().collect::<Vec<_>>();
        let headers = headers
            .into_iter()
            .filter(|index| top_level_key(lines[*index]) == Some("pre-push"))
            .collect::<Vec<_>>();
        if headers.is_empty() {
            return Err("lefthook config is missing the top-level pre-push section".to_string());
        }

        let start = headers[0];
        let end = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| top_level_key(line).is_some())
            .map_or(lines.len(), |(index, _)| index);
        validate_pre_push_section(&lines[start..end])?;
        validate_hook_script(
            strict_hook,
            &[
                "#!/usr/bin/env bash",
                "set -euo pipefail",
                "exec just JOBS=2 pre-push-check",
            ],
            "strict pre-push",
        )?;
        validate_hook_script(
            guard_hook,
            &[
                "#!/usr/bin/env bash",
                "set -euo pipefail",
                "exec bash scripts/governance/validate-push.sh",
            ],
            "governance pre-push",
        )
    }
}

fn validate_root_keys(config: &str) -> Result<Vec<usize>, String> {
    let mut seen = Vec::new();
    let mut key_lines = Vec::new();
    for (index, line) in config.lines().enumerate() {
        let content = line.trim_start();
        if content.is_empty() || content.starts_with('#') {
            continue;
        }
        if line.chars().next().is_some_and(char::is_whitespace) {
            continue;
        }
        let Some(key) = top_level_key(line) else {
            return Err("lefthook config has a complex top-level entry".to_string());
        };
        if !ROOT_KEYS.contains(&key) {
            return Err(format!(
                "lefthook config has an unsupported top-level key: {key}"
            ));
        }
        if seen.contains(&key) {
            return Err(format!("lefthook config duplicates top-level key: {key}"));
        }
        seen.push(key);
        key_lines.push(index);
    }
    Ok(key_lines)
}

fn top_level_key(line: &str) -> Option<&str> {
    if line.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    line.trim_end().split_once(':').map(|(key, _)| key)
}

fn validate_pre_push_section(lines: &[&str]) -> Result<(), String> {
    let actual = lines
        .iter()
        .map(|line| line.trim_end())
        .filter(|line| {
            let content = line.trim_start();
            !content.is_empty() && !content.starts_with('#')
        })
        .collect::<Vec<_>>();
    let expected = [
        "pre-push:",
        "  parallel: false",
        "  piped: true",
        "  jobs:",
        "    - name: issue-and-dependency",
        "      script: issue-and-dependency.sh",
        "      runner: bash",
        "      use_stdin: true",
        "    - name: check-strict",
        "      script: check-strict.sh",
        "      runner: bash",
    ];
    if actual != expected {
        return Err(
            "lefthook pre-push section does not match the required ordered jobs".to_string(),
        );
    }
    Ok(())
}

fn validate_hook_script(script: &str, expected: &[&str], label: &str) -> Result<(), String> {
    let actual = script
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && (!line.starts_with('#') || line.starts_with("#!")))
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(format!(
            "{label} hook script must contain only its required fail-fast commands"
        ));
    }
    Ok(())
}
