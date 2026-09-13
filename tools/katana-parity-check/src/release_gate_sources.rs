pub(crate) const JUSTFILE: &str = include_str!("../../../Justfile");
pub(crate) const LEFTHOOK: &str = include_str!("../../../lefthook.yml");
pub(crate) const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
pub(crate) const RELEASE_WORKFLOW: &str = include_str!("../../../.github/workflows/release.yml");
pub(crate) const TEST_AND_BUILD_WORKFLOW: &str =
    include_str!("../../../.github/workflows/test-and-build.yml");
pub(crate) const SOURCE_CLOSURE_WORKFLOW: &str =
    include_str!("../../../.github/workflows/source-closure.yml");
pub(crate) const PUBLISH_CRATES: &str = include_str!("../../../scripts/release/publish-crates.sh");
pub(crate) const RELEASE_COMPLETION: &str =
    include_str!("../../../scripts/release/verify-release-completion.sh");
pub(crate) const RELEASE_BLOCKERS: &str =
    include_str!("../../../scripts/release/verify-release-blockers.sh");
pub(crate) const RELEASE_DOC: &str = include_str!("../../../docs/release.md");
pub(crate) const RELEASE_READINESS_DOC: &str =
    include_str!("../../../docs/v0-1-0-release-readiness.md");

pub(crate) const RELEASE_GATE_EVIDENCE: &[&str] = &[
    "release-check-clean-generated-artifacts:",
    "release-check: release-target-check release-check-clean-generated-artifacts",
    "    just release-verify",
    "    just kle-release-parity-check",
    "    bash scripts/release/assert-crates-not-published.sh \"{{VERSION}}\"",
    "{{CARGO}} test -p kle-storybook --locked storybook_motion_artifact_uses_the_exact_kuc_plan_without_idle_frames -- --test-threads=1",
    "{{CARGO}} test -j {{JOBS}} -p kle-linter ast_linter -- --nocapture",
    "{{CARGO}} test -j {{JOBS}} -p kle-linter --test ast_linter ast_linter_workspace_rules -- --nocapture",
    "release-completion-audit:",
    "    RELEASE_REPO=\"{{RELEASE_REPO}}\" bash scripts/release/verify-release-completion.sh \"{{VERSION}}\"",
    "release-blocker-audit:",
    "    RELEASE_REPO=\"{{RELEASE_REPO}}\" bash scripts/release/verify-release-blockers.sh \"{{VERSION}}\"",
    "STORYBOOK_MOTION_ARTIFACT_OUTPUT := env_var_or_default(\"STORYBOOK_MOTION_ARTIFACT_OUTPUT\", \"\")",
    "output_dir=\"{{STORYBOOK_MOTION_ARTIFACT_OUTPUT}}\"; if test -z \"$output_dir\"; then output_dir=\"target/acceptance/kle-storybook-motion-artifact-$(date -u +%Y%m%dT%H%M%SZ)-$$\"; fi;",
];

pub(crate) const RELEASE_WORKFLOW_EVIDENCE: &[&str] = &[
    "Manual releases with false are rejected before mutation.",
    "default: true",
    "- name: Reject partial manual release",
    "if: github.event_name == 'workflow_dispatch' && inputs.publish_crates != true",
    "workflow_dispatch releases must publish crates and run completion audits.",
    "tag_commit=\"$(git rev-list -n 1 \"refs/tags/${TAG}\")\"",
    "head_commit=\"$(git rev-parse HEAD)\"",
    "Tag ${TAG} already exists but does not point to current HEAD.",
    "- name: Create GitHub Release",
    "- name: Publish crates.io",
    "- name: Verify public release completion",
    "run: just VERSION=\"${{ steps.version.outputs.version }}\" release-completion-audit",
];

pub(crate) const PUBLISH_CRATES_EVIDENCE: &[&str] = &[
    "publish_if_needed katana-language-editor \"${CARGO_REGISTRY_TOKEN}\"",
    "wait_for_crate katana-language-editor",
    "publish_if_needed katana-language-editor-egui \"${CARGO_REGISTRY_TOKEN}\"",
    "wait_for_crate katana-language-editor-egui",
];

pub(crate) const RELEASE_COMPLETION_EVIDENCE: &[&str] = &[
    "EXPECTED_RELEASE_COMMIT",
    "expected_release_commit=\"${EXPECTED_RELEASE_COMMIT:-$(git rev-parse HEAD)}\"",
    "git ls-remote --tags origin \"refs/tags/${version}\" \"refs/tags/${version}^{}\"",
    "remote_tag_commit=",
    "gh release view \"${version}\" --repo \"${RELEASE_REPO}\" --json tagName,isDraft,isPrerelease,url",
    "cargo info \"${crate}@${version_bare}\" --registry crates-io",
    "PASS: release completion verification passed for ${version}",
];

pub(crate) const RELEASE_BLOCKER_AUDIT_EVIDENCE: &[&str] = &[
    "blocker_labels=()",
    "add_blocker \"remote-tag\"",
    "Remote tag commit mismatch for ${version}",
    "add_blocker \"github-release\"",
    "kuc-registry-dependency",
    "cargo run --locked -p kle-storybook -- --contract-check",
    "add_blocker \"kuc-consumer-artifact\"",
    "cargo check --workspace --all-targets --locked",
    "add_blocker \"dependency-check\"",
    "add_blocker \"future-incompatibility\"",
    "PASS: no release blockers detected for ${version}",
    "Release blockers for ${version}:",
];

pub(crate) const RELEASE_DOC_EVIDENCE: &[&str] = &[
    "`release-check` は公開前の品質ゲートであり、公開完了の証明ではない。",
    "`just VERSION=vX.Y.Z release-completion-audit`",
    "`just VERSION=vX.Y.Z release-blocker-audit`",
    "tag / GitHub Release / crates.io",
    "KatanA #336 の採用は final `v0.1.0` 公開後の下流作業である。",
    "手動実行で `publish_crates=false` が指定された場合、tag / GitHub Release の作成前に失敗させる。",
    "tag は存在だけでなく、既定では audit 実行時の `HEAD`、必要なら `EXPECTED_RELEASE_COMMIT` で指定した release 対象 commit と一致しなければならない",
    "v0.1.0 の現在の完了判定は `docs/v0-1-0-release-readiness.md` に記録する。",
];

pub(crate) const RELEASE_READINESS_EVIDENCE: &[&str] = &[
    "As of 2026-09-02, KLE v0.1.0 is **not release-ready**.",
    "source closure / parity checker |",
    "Historical entries in this file that claimed a passing `release-check`, a",
    "KatanA runtime adoption |",
    "KatanA #336 is a downstream task after final v0.1.0 publication.",
    "KUC root contract | partial",
    "KatanA downstream host E2E | required after final registry v0.1.0 publication",
    "Publication work may begin only after all conditions below are passing",
];
