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
    RUSTFLAGS="-D warnings" {{CARGO}} clippy -j {{JOBS}} --workspace --all-targets --all-features --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports

# Run Rust syntax based structural checks
ast-lint:
    {{CARGO}} test -j {{JOBS}} -p kle-linter ast_linter -- --nocapture

# Run workspace tests
unit-test:
    {{CARGO}} test --workspace --all-targets --all-features --locked

# Alias used by KML-style workflows
test: unit-test

# Run coverage as a required full-check gate
coverage:
    {{CARGO}} llvm-cov --workspace --all-features --locked --summary-only --fail-under-lines {{COVERAGE_MIN_LINES}}

# Run the local quality gate
check: fmt-check check-types lint unit-test ast-lint
    @echo "checks passed"

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

# Verify release branch readiness before merging
release-check: release-target-check release-verify
    bash scripts/release/assert-crates-not-published.sh "{{VERSION}}"

# Show recent Release workflow runs
release-status:
    gh run list --repo {{RELEASE_REPO}} --workflow Release --limit 5
