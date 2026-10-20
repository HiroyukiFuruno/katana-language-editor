use crate::release_gate_sources::{LEFTHOOK, PRE_PUSH_GUARD_HOOK, PRE_PUSH_STRICT_HOOK};

fn validate(config: &str, strict_hook: &str, guard_hook: &str) -> Result<(), String> {
    crate::release_gate_hook::ReleaseGateHookAudit::validate_pre_push_hook_sources(
        config,
        strict_hook,
        guard_hook,
    )
}

#[test]
fn current_pre_push_config_and_production_scripts_pass() -> Result<(), String> {
    validate(LEFTHOOK, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK)
}

#[test]
fn pre_push_rejects_the_legacy_run_command() {
    let legacy = "pre-push:\n  commands:\n    strict:\n      run: just JOBS=2 pre-push-check\n";
    assert_eq!(
        validate(legacy, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook pre-push section does not match the required ordered jobs".to_string())
    );
}

#[test]
fn pre_push_rejects_missing_stdin_forwarding() {
    let config = LEFTHOOK.replace("      use_stdin: true\n", "");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook pre-push section does not match the required ordered jobs".to_string())
    );
}

#[test]
fn pre_push_rejects_skipped_guard_stdin() {
    let config = LEFTHOOK.replace("      use_stdin: true", "      use_stdin: false");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook pre-push section does not match the required ordered jobs".to_string())
    );
}

#[test]
fn pre_push_rejects_a_skipped_guard_job() {
    let config = LEFTHOOK.replace(
        "      use_stdin: true",
        "      use_stdin: true\n      skip: true",
    );
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook pre-push section does not match the required ordered jobs".to_string())
    );
}

#[test]
fn pre_push_rejects_guard_after_strict_job() {
    let config = LEFTHOOK
        .replace("    - name: issue-and-dependency", "    - name: temporary")
        .replace(
            "    - name: check-strict",
            "    - name: issue-and-dependency",
        )
        .replace("    - name: temporary", "    - name: check-strict");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook pre-push section does not match the required ordered jobs".to_string())
    );
}

#[test]
fn pre_push_rejects_non_piped_jobs() {
    let config = LEFTHOOK.replace("  piped: true", "  piped: false");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook pre-push section does not match the required ordered jobs".to_string())
    );
}

#[test]
fn pre_push_rejects_duplicate_section() {
    let config = format!("{LEFTHOOK}\npre-push:\n  piped: true\n");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook config duplicates top-level key: pre-push".to_string())
    );
}

#[test]
fn fixed_schema_rejects_quoted_duplicate_pre_push_keys() {
    for (quoted_key, parsed_key) in [
        ("'pre-push':", "'pre-push'"),
        ("\"pre-push\":", "\"pre-push\""),
    ] {
        let config = format!("{LEFTHOOK}\n{quoted_key}\n  piped: false\n");
        assert_eq!(
            validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
            Err(format!(
                "lefthook config has an unsupported top-level key: {parsed_key}"
            ))
        );
    }
}

#[test]
fn fixed_schema_rejects_escaped_quoted_pre_push_key() {
    let config = format!("{LEFTHOOK}\n\"pre\\u002dpush\":\n  piped: false\n");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook config has an unsupported top-level key: \"pre\\u002dpush\"".to_string())
    );
}

#[test]
fn fixed_schema_rejects_unknown_override_keys() {
    for key in ["extends", "remotes", "source_dir"] {
        let config = format!("{LEFTHOOK}\n{key}: override\n");
        assert_eq!(
            validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
            Err(format!(
                "lefthook config has an unsupported top-level key: {key}"
            ))
        );
    }
}

#[test]
fn fixed_schema_rejects_complex_top_level_entries() {
    let config = format!("{LEFTHOOK}\n? pre-push\n: override\n");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook config has a complex top-level entry".to_string())
    );
}

#[test]
fn fixed_schema_rejects_a_misindented_pre_push_section() {
    let config = LEFTHOOK.replace("\npre-push:\n", "\n  pre-push:\n");
    assert_eq!(
        validate(&config, PRE_PUSH_STRICT_HOOK, PRE_PUSH_GUARD_HOOK),
        Err("lefthook config is missing the top-level pre-push section".to_string())
    );
}

#[test]
fn guard_script_rejects_early_success_exit() {
    let script = format!("{PRE_PUSH_GUARD_HOOK}exit 0\n");
    assert_eq!(
        validate(LEFTHOOK, PRE_PUSH_STRICT_HOOK, &script),
        Err(
            "governance pre-push hook script must contain only its required fail-fast commands"
                .to_string()
        )
    );
}

#[test]
fn strict_script_rejects_noop() {
    let script = PRE_PUSH_STRICT_HOOK.replace("exec just JOBS=2 pre-push-check", "exec true");
    assert_eq!(
        validate(LEFTHOOK, &script, PRE_PUSH_GUARD_HOOK),
        Err(
            "strict pre-push hook script must contain only its required fail-fast commands"
                .to_string()
        )
    );
}

#[test]
fn pre_push_rejects_missing_or_malformed_section() {
    assert_eq!(
        validate(
            "commit-msg:\n  scripts: {}\n",
            PRE_PUSH_STRICT_HOOK,
            PRE_PUSH_GUARD_HOOK
        ),
        Err("lefthook config is missing the top-level pre-push section".to_string())
    );
    assert_eq!(
        validate(
            "pre-push :\n  piped: true\n",
            PRE_PUSH_STRICT_HOOK,
            PRE_PUSH_GUARD_HOOK
        ),
        Err("lefthook config has an unsupported top-level key: pre-push ".to_string())
    );
}
