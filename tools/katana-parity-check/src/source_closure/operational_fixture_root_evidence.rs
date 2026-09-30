use std::fs;
use std::path::Path;

use super::super::super::operational_fixture_write::write_evidence;
use super::super::super::operational_input::{
    EvidenceRef, FIXED_KATANA_REVISION, KatanaTreeEvidence, ProfileProbeInput, RootEvidence,
    RootProvenance, SourceClosureInput,
};
use super::super::super::user_mandated_extensions::CANONICAL_USER_MANDATED_LEAVES;

pub(crate) struct EvidenceRootBuilder;

impl EvidenceRootBuilder {
    pub(crate) fn make_root(
        input_dir: &Path,
        profiles: &[ProfileProbeInput],
        katana_root: &Path,
        seed_path: &str,
        tree_paths: &[&str],
    ) -> Result<RootProvenance, String> {
        let revision_bytes = format!("{FIXED_KATANA_REVISION}\n").into_bytes();
        let revision = write_evidence(
            input_dir,
            "raw/provenance/katana-revision.txt",
            &revision_bytes,
        )?;
        let katana_tree = tree_paths
            .iter()
            .map(|source_path| {
                let bytes = fs::read(katana_root.join(source_path)).map_err(|error| {
                    format!(
                        "actual tree source read failed for {}: {error}",
                        katana_root.join(source_path).display()
                    )
                })?;
                Ok(KatanaTreeEvidence {
                    source_path: (*source_path).into(),
                    evidence: write_evidence(
                        input_dir,
                        &format!("raw/provenance/katana-tree/{source_path}"),
                        &bytes,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let external_ui = vec![write_evidence(
            input_dir,
            "raw/provenance/external-ui.txt",
            b"egui lock source symbol capture",
        )?];
        let user = write_evidence(
            input_dir,
            "raw/provenance/user-mandated-leaves.json",
            CANONICAL_USER_MANDATED_LEAVES,
        )?;
        let source_universe = super::source_universe::capture(input_dir)?;
        let requirement_source_aliases = write_evidence(
            input_dir,
            "raw/provenance/editor-requirement-source-aliases.json",
            include_bytes!("../../../../docs/v0-1-0-editor-requirement-source-aliases.json"),
        )?;
        let kle_tree = vec![write_evidence(
            input_dir,
            "raw/provenance/kle-tree.txt",
            b"kle source tree bytes",
        )?];
        let kuc_tree = vec![write_evidence(
            input_dir,
            "raw/provenance/kuc-tree.txt",
            b"kuc source tree bytes",
        )?];
        let kle_revision = write_evidence(
            input_dir,
            "raw/provenance/kle-revision.txt",
            b"0123456789abcdef0123456789abcdef01234567\n",
        )?;
        let kle_worktree_status =
            write_evidence(input_dir, "raw/provenance/kle-worktree-status.txt", b"")?;
        let kuc_revision = write_evidence(
            input_dir,
            "raw/provenance/kuc-revision.txt",
            b"89abcdef0123456789abcdef0123456789abcdef\n",
        )?;
        let kuc_worktree_status =
            write_evidence(input_dir, "raw/provenance/kuc-worktree-status.txt", b"")?;
        let matrix = profiles
            .iter()
            .map(|profile| {
                Ok(EvidenceRef {
                    runner_label: profile.id.clone(),
                    ..write_evidence(
                        input_dir,
                        &format!("raw/provenance/matrix-{}.json", profile.id),
                        profile.id.as_bytes(),
                    )?
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let generator_binary = write_evidence(
            input_dir,
            "raw/provenance/generator.bin",
            b"generator binary bytes",
        )?;
        let generator_schema = write_evidence(
            input_dir,
            "raw/provenance/generator/schema.json",
            b"generator schema bytes",
        )?;
        let mut root = RootProvenance {
            schema_version: "1".into(),
            katana_revision: FIXED_KATANA_REVISION.into(),
            kle_revision: "0123456789abcdef0123456789abcdef01234567".into(),
            kle_worktree_clean: true,
            kuc_revision: "89abcdef0123456789abcdef0123456789abcdef".into(),
            kuc_worktree_clean: true,
            katana_tree_fingerprint: KatanaTreeEvidence::set_fingerprint(&katana_tree),
            katana_external_ui_fingerprint: EvidenceRef::set_fingerprint(&external_ui),
            user_mandated_extensions_fingerprint: user.sha256.clone(),
            source_universe_fingerprint: source_universe.sha256.clone(),
            requirement_source_aliases_fingerprint: requirement_source_aliases.sha256.clone(),
            kle_tree_fingerprint: EvidenceRef::set_fingerprint(&kle_tree),
            kuc_tree_fingerprint: EvidenceRef::set_fingerprint(&kuc_tree),
            release_profile_matrix_fingerprint: String::new(),
            generator_fingerprint: EvidenceRef::set_fingerprint(&[
                generator_binary.clone(),
                generator_schema.clone(),
            ]),
            generated_at_utc: "2026-08-20T12:34:56Z".into(),
            evidence: RootEvidence {
                katana_revision: EvidenceRef {
                    command_or_source: "git rev-parse HEAD".into(),
                    ..revision
                },
                kle_revision: EvidenceRef {
                    command_or_source: "git rev-parse HEAD".into(),
                    ..kle_revision
                },
                kle_worktree_status: EvidenceRef {
                    command_or_source: "git status --porcelain=v1 --untracked-files=all".into(),
                    ..kle_worktree_status
                },
                kuc_revision: EvidenceRef {
                    command_or_source: "git rev-parse HEAD".into(),
                    ..kuc_revision
                },
                kuc_worktree_status: EvidenceRef {
                    command_or_source: "git status --porcelain=v1 --untracked-files=all".into(),
                    ..kuc_worktree_status
                },
                katana_tree,
                katana_external_ui: external_ui,
                user_mandated_extensions: EvidenceRef {
                    command_or_source: "docs/v0-1-0-user-mandated-leaves.json".into(),
                    ..user
                },
                source_universe: EvidenceRef {
                    command_or_source: "docs/v0-1-0-katana-editor-source-universe.md".into(),
                    ..source_universe
                },
                requirement_source_aliases: EvidenceRef {
                    command_or_source: "docs/v0-1-0-editor-requirement-source-aliases.json".into(),
                    ..requirement_source_aliases
                },
                kle_tree,
                kuc_tree,
                release_profile_matrix: matrix,
                generator_binary,
                generator_schema,
            },
        };
        let input = SourceClosureInput {
            input_schema_version: "1".into(),
            katana_seed_paths: vec![seed_path.into()],
            root: root.clone(),
            profile_probes: profiles.to_vec(),
        };
        root.release_profile_matrix_fingerprint = input.matrix_fingerprint();
        Ok(root)
    }
}
