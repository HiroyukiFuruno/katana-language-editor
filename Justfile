# ============================================================
# katana-language-editor - Development Justfile
# ============================================================
# Stable local, CI/CD, and release task entrypoint.
# Usage:
#   just
#   just <recipe>
#   just VERSION=vX.Y.Z release-check
# ============================================================

set shell := ["bash", "-uc"]

REPO_ROOT := justfile_directory()
RTK := env_var_or_default("RTK", `command -v rtk 2> /dev/null || true`)
RTK_CMD := if RTK == "" { "" } else { RTK + " " }
JOBS := env_var_or_default("JOBS", "2")
CARGO := env_var_or_default("CARGO", RTK_CMD + "cargo")
VERSION := env_var_or_default("VERSION", `awk -F '"' '/^version = / { print $2; exit }' Cargo.toml`)
VERSION_BARE := replace(VERSION, "v", "")
TAG := "v" + VERSION_BARE
COVERAGE_MIN_LINES := env_var_or_default("COVERAGE_MIN_LINES", "64")
RELEASE_REPO := env_var_or_default("RELEASE_REPO", "HiroyukiFuruno/katana-language-editor")
KDV_REPO_NAME := env_var_or_default("KDV_REPO_NAME", "HiroyukiFuruno/katana-document-viewer")
KUC_MANIFEST := env_var_or_default("KUC_MANIFEST", "../katana-ui-core/Cargo.toml")
KUC_REPO := env_var_or_default("KUC_REPO", "../katana-ui-core")
KATANA_REPO := env_var_or_default("KATANA_REPO", "")
KATANA_HOST_E2E_TARGET := env_var_or_default("KATANA_HOST_E2E_TARGET", "/tmp/katana-host-e2e-target")
KDV_REPO := env_var_or_default("KDV_REPO", "../katana-document-viewer")
SOURCE_CLOSURE_ROOT := env_var_or_default("SOURCE_CLOSURE_ROOT", "target/source-closure")
SOURCE_CLOSURE_RUN_ID := env_var_or_default("SOURCE_CLOSURE_RUN_ID", "")
SOURCE_CLOSURE_INPUT := env_var_or_default("SOURCE_CLOSURE_INPUT", SOURCE_CLOSURE_ROOT + "/" + SOURCE_CLOSURE_RUN_ID + "/assembled/source-closure-input.json")
STORYBOOK_MOTION_ARTIFACT_OUTPUT := env_var_or_default("STORYBOOK_MOTION_ARTIFACT_OUTPUT", "")

export RUSTFLAGS := env_var_or_default("RUSTFLAGS", "-D warnings")

default: help

# Show this help
help:
    @just --list --unsorted

# Apply Rust formatting
fmt:
    {{CARGO}} fmt --all

# Check Rust formatting
fmt-check:
    {{CARGO}} fmt --all -- --check

# Check workspace type safety
check-types:
    {{CARGO}} check --workspace --locked

# Run strict Clippy checks
lint:
    {{CARGO}} clippy -j {{JOBS}} --workspace --all-targets --all-features --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports

# Run Rust syntax based structural checks
ast-lint:
    {{CARGO}} test -j {{JOBS}} -p kle-linter ast_linter -- --nocapture
    {{CARGO}} test -j {{JOBS}} -p kle-linter --test ast_linter ast_linter_workspace_rules -- --nocapture

# Launch the interactive KLE Storybook window
storybook:
    {{CARGO}} run --locked -p kle-storybook -- --interactive

# Run a live window smoke check for the KLE Storybook
storybook-window-smoke:
    {{CARGO}} run --locked -p kle-storybook -- --interactive --frames 2

# Verify Storybook state/event/action/callback roundtrips
storybook-interaction-check:
    {{CARGO}} test -p kle-storybook --locked -- --test-threads=1

# Verify Storybook emoji text editing fixture
storybook-emoji-check:
    {{CARGO}} run --locked -p kle-storybook -- --emoji-check
    {{CARGO}} test -p kle-storybook --locked emoji_roundtrip -- --test-threads=1

# Verify Storybook contract behavior and unsupported paths
storybook-contract-check:
    {{CARGO}} test -p kle-storybook --locked -- --test-threads=1

# Verify KLE consumes generic KUC contracts from the resolved registry crate.
kuc-contract-check:
    {{CARGO}} run --locked -p kle-storybook -- --contract-check
    {{CARGO}} test -p kle-storybook --locked kuc -- --test-threads=1

# Verify a KatanA-style host can use only the neutral interface crate
katana-interface-check:
    {{CARGO}} check -p katana-interface-check --locked
    {{CARGO}} tree -p katana-interface-check --locked --edges normal > target/katana-interface-check-tree.txt
    ! grep -E 'katana-language-editor-egui|egui v' target/katana-interface-check-tree.txt

# Diagnostic-only compile/runtime check; not fixed-KatanA host evidence
katana-downstream-check:
    {{CARGO}} run --locked -p katana-downstream-check

# Verify typed image intent only; live KatanA clipboard acquisition is intentionally unproven
katana-host-e2e:
    just katana-host-e2e-no-live-clipboard-acquisition

# Require an explicit KatanA checkout only for commands that inspect or execute it.
require-katana-repo:
    test -n "{{KATANA_REPO}}" || { printf '%s\n' 'KATANA_REPO is required for this KatanA host/source-closure command.' >&2; exit 2; }
    git -C "{{KATANA_REPO}}" rev-parse --show-toplevel >/dev/null

# Verify KLE -> KatanA host integration without invoking OS clipboard acquisition
katana-host-e2e-no-live-clipboard-acquisition: require-katana-repo
    KATANA_REPO="{{KATANA_REPO}}" CARGO_TARGET_DIR="{{KATANA_HOST_E2E_TARGET}}" {{CARGO}} test --manifest-path tools/katana-host-e2e/Cargo.toml --locked

# Verify ContextMenu parity through the actual KLE/KatanA host input suite.
katana-host-e2e-context-menu: require-katana-repo
    KATANA_REPO="{{KATANA_REPO}}" CARGO_TARGET_DIR="{{KATANA_HOST_E2E_TARGET}}" {{CARGO}} test --manifest-path tools/katana-host-e2e/Cargo.toml --locked --test context_menu_input

# Advisory KLE/KUC precheck. It records KatanA #336 only as a downstream
# requirement and is not the final public-release evidence gate.
katana-parity-rc-check:
    {{CARGO}} run --locked -p katana-parity-check -- --mode rc

# Verify KLE pre-publication evidence. It never requires KatanA #336.
kle-release-parity-check:
    {{CARGO}} run --locked -p katana-parity-check -- --mode kle-release

# Verify every post-publication KatanA editor parity row has joined actual host evidence.
full-parity-check:
    {{CARGO}} run --locked -p katana-parity-check -- --mode downstream-full

# Backward-compatible name for the final, fail-closed parity gate.
katana-parity-check: kle-release-parity-check

# Diagnose requirement/source bindings without publishing release evidence.
source-requirement-binding-audit OUTPUT: require-katana-repo
    {{CARGO}} run --locked -p katana-parity-check -- source-closure audit-requirement-bindings --katana-repo "{{KATANA_REPO}}" --output "{{OUTPUT}}"

# Capture one native GitHub runner source-closure profile; no host override is accepted.
source-closure-capture-profile PROFILE_ID: require-katana-repo
    {{CARGO}} run --locked -p katana-parity-check -- source-closure capture-profile --profile-id "{{PROFILE_ID}}" --output-dir "{{SOURCE_CLOSURE_ROOT}}" --katana-repo "{{KATANA_REPO}}" --run-id "{{SOURCE_CLOSURE_RUN_ID}}"

# Capture read-only KatanA/KUC/KLE provenance after all profile artifacts exist.
source-closure-capture-provenance: require-katana-repo
    {{CARGO}} run --locked -p katana-parity-check -- source-closure capture-provenance --output-dir "{{SOURCE_CLOSURE_ROOT}}" --katana-repo "{{KATANA_REPO}}" --kle-repo "{{REPO_ROOT}}" --kuc-repo "{{KUC_REPO}}" --user-input "{{REPO_ROOT}}/docs/v0-1-0-user-mandated-leaves.json" --generator-schema "{{REPO_ROOT}}/docs/v0-1-0-parity-manifest-schema.md" --source-universe "{{REPO_ROOT}}/docs/v0-1-0-katana-editor-source-universe.md" --requirement-source-aliases "{{REPO_ROOT}}/docs/v0-1-0-editor-requirement-source-aliases.json" --run-id "{{SOURCE_CLOSURE_RUN_ID}}"

# Assemble one canonical input from all three profiles and the same-run provenance.
source-closure-assemble-input: require-katana-repo
    SOURCE_CLOSURE_RUN_ID="{{SOURCE_CLOSURE_RUN_ID}}" {{CARGO}} run --locked -p katana-parity-check -- source-closure assemble-input --output-dir "{{SOURCE_CLOSURE_ROOT}}" --seed-manifest "{{REPO_ROOT}}/docs/v0-1-0-source-closure-roots.json" --source-universe "{{REPO_ROOT}}/docs/v0-1-0-katana-editor-source-universe.md" --katana-repo "{{KATANA_REPO}}"

# Validate raw evidence, fixed revision, tree and profile matrix before materialization.
source-closure-validate-input:
    {{CARGO}} run --locked -p katana-parity-check -- source-closure validate-input --input "{{SOURCE_CLOSURE_INPUT}}"

# Materialize only a loader-verified SourceClosureInput.
source-closure-materialize: require-katana-repo
    {{CARGO}} run --locked -p katana-parity-check -- source-closure materialize-closure --input "{{SOURCE_CLOSURE_INPUT}}" --katana-repo "{{KATANA_REPO}}" --canonical-root "artifacts"

# Verify the actual KatanA repo has the KLE-backed downstream adapter wired and passing
katana-repo-adapter-check: require-katana-repo
    {{CARGO}} run --locked -p katana-repo-adapter-check -- --repo "{{KATANA_REPO}}"
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui toolbar_popup -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter::kle_kuc_text_area_ime_commit_star_reaches_real_katana_buffer -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_authoring_heading1_updates_editor_state_via_real_ui_routing -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_save_button_persists_editor_buffer_state -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_context_menu_format_button_updates_formatted_markdown_buffer -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter::kle_document_search_markdown_aware_matches_and_refreshes_via_kle_adapter -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_navigation -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_select_and_jump -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_view_modes -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_toggle_view_modes -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_layout_persistence -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_kle_downstream_adapter::kle_event_action_stream_updates_real_katana_editor_state -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::test_integration_editor_line_numbers_visibility -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::input_assist_code_block_button_updates_editor_buffer -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::code_block_kind_menu_closes_when_editor_is_clicked -- --nocapture
    cd "{{KATANA_REPO}}" && {{RTK_CMD}}cargo test -p katana-ui --test ui_integration_parallel editor_ui::editor_scroll_sync -- --nocapture

# Verify KatanA can build against the released KLE git tag without mutating the KatanA worktree
katana-tag-dependency-check: require-katana-repo
    bash scripts/release/verify-katana-tag-dependency.sh "{{VERSION}}" "{{KATANA_REPO}}" "{{KUC_REPO}}" "https://github.com/{{RELEASE_REPO}}"

# Prepare a post-release KDV preset follow-up issue draft without mutating KDV
kdv-preset-followup:
    bash scripts/release/prepare-kdv-preset-followup.sh "{{VERSION}}" "{{KDV_REPO}}"

# Verify the KDV preset follow-up issue state without mutating KDV
kdv-preset-followup-audit:
    KDV_REPO="{{KDV_REPO}}" KDV_REPO_NAME="{{KDV_REPO_NAME}}" bash scripts/release/verify-kdv-preset-followup.sh "{{VERSION}}"

# Verify the public release is externally complete without mutating repositories
release-completion-audit:
    RELEASE_REPO="{{RELEASE_REPO}}" bash scripts/release/verify-release-completion.sh "{{VERSION}}"

# List all current public release blockers without mutating repositories
release-blocker-audit:
    RELEASE_REPO="{{RELEASE_REPO}}" bash scripts/release/verify-release-blockers.sh "{{VERSION}}"

# Generate a deterministic partial Storybook motion artifact. This is not release evidence.
storybook-motion-artifact:
    output_dir="{{STORYBOOK_MOTION_ARTIFACT_OUTPUT}}"; if test -z "$output_dir"; then output_dir="target/acceptance/kle-storybook-motion-artifact-$(date -u +%Y%m%dT%H%M%SZ)-$$"; fi; {{CARGO}} run --locked -p kle-storybook -- --motion-artifact --artifact-output "$output_dir"

# Verify the partial motion artifact generator. Full release evidence remains source-closure gated.
storybook-motion-artifact-gate:
    just storybook-motion-artifact
    {{CARGO}} test -p kle-storybook --locked storybook_motion_artifact_uses_the_exact_kuc_plan_without_idle_frames -- --test-threads=1

# Run workspace tests
unit-test: diagnostic-probe-test
    {{CARGO}} test --workspace --all-targets --all-features --locked
    {{CARGO}} test --workspace --doc --all-features --locked

# Verify diagnostic protocol and authenticated definition boundaries
diagnostic-probe-test:
    PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/diagnostics -p 'test_*.py' -v

# Alias used by KML-style workflows
test: unit-test

# Run coverage as a required full-check gate
coverage:
    {{CARGO}} llvm-cov --workspace --all-features --locked --summary-only --fail-under-lines {{COVERAGE_MIN_LINES}}

# Run the advisory RC-quality precheck during ordinary development. The required
# public-release gate is `release-check -> kle-release-parity-check`; KatanA #336
# remains a separate post-publication `downstream-full` gate.
check: fmt-check check-types lint unit-test ast-lint kuc-contract-check katana-interface-check katana-downstream-check storybook-motion-artifact-gate katana-parity-rc-check
    @echo "checks passed"

# Run every strict local gate before push. Three-OS source-closure artifacts are
# intentionally excluded because this push publishes the workflow that creates them.
pre-push-check: fmt-check check-types lint unit-test ast-lint kuc-contract-check katana-interface-check katana-downstream-check storybook-motion-artifact-gate
    @echo "pre-push checks passed"

# Compatibility alias for the advisory precheck; it is not a release prerequisite.
kle-rc-check: check

# Sweep old build artifacts locally (older than 7 days)
sweep:
    @{{CARGO}} sweep --time 7 || true

# Remove build artifacts
clean: sweep
    {{CARGO}} clean

# Update dependency crates to latest compatible versions
update-safe:
    {{RTK_CMD}}cargo update

# Upgrade all dependency requirements, then update Cargo.lock
update:
    {{RTK_CMD}}cargo upgrade -i
    {{RTK_CMD}}cargo update

# Verify VERSION follows the published release line
release-target-check:
    bash scripts/release/verify-version.sh "{{VERSION}}"
    python3 scripts/release/verify-release-target.py --target-version "{{VERSION}}" --repo "{{RELEASE_REPO}}"

# Verify package metadata and dry-run the first publishable crate
release-verify: check coverage
    bash scripts/release/verify-version.sh "{{VERSION}}"
    bash scripts/release/verify-internal-dependencies.sh "{{VERSION}}"
    {{CARGO}} package -p katana-language-editor --locked --allow-dirty
    {{CARGO}} package -p katana-language-editor-egui --locked --allow-dirty --list >/dev/null
    {{CARGO}} publish -p katana-language-editor --dry-run --locked --allow-dirty

# release-check の coverage/package 実行後に生成物を解放する
release-check-clean-generated-artifacts:
    {{CARGO}} clean
    for target_dir in target/llvm-cov-target tools/katana-interface-check/target tools/katana-downstream-check/target tools/katana-parity-check/target tools/kle-storybook/target; do \
        if test -f "$target_dir/CACHEDIR.TAG" && test "$(sed -n '1p' "$target_dir/CACHEDIR.TAG")" = "Signature: 8a477f597d28d172789f06886806bc55"; then \
            {{CARGO}} clean --target-dir "$target_dir"; \
        else \
            printf 'skip unsafe or absent Cargo target: %s\n' "$target_dir"; \
        fi; \
    done

# Verify release branch readiness before merging
release-check: release-target-check release-check-clean-generated-artifacts
    just release-verify
    just kle-release-parity-check
    bash scripts/release/assert-crates-not-published.sh "{{VERSION}}"

# Show recent Release workflow runs
release-status:
    gh run list --repo {{RELEASE_REPO}} --workflow Release --limit 5
