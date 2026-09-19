use crate::search_motion_sequence::MotionArtifactFrame;
use katana_ui_core::egui::VariableViewportMotionArtifact;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const EVIDENCE_FILE_NAME: &str = "search-motion-evidence.txt";

pub(super) fn invalidate_completion(output_dir: &Path) -> Result<(), io::Error> {
    let evidence_path = output_dir.join(EVIDENCE_FILE_NAME);
    match fs::remove_file(evidence_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn write(
    output_dir: &Path,
    requested_frames: usize,
    frames: &[MotionArtifactFrame],
    artifact: &VariableViewportMotionArtifact,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut body = manifest_header(output_dir, requested_frames, artifact);
    append_frames(&mut body, frames);
    write_completion_marker(output_dir, &body)?;
    Ok(())
}

fn write_completion_marker(output_dir: &Path, body: &str) -> Result<(), io::Error> {
    let evidence_path = output_dir.join(EVIDENCE_FILE_NAME);
    let pending_path = output_dir.join(format!(
        ".{EVIDENCE_FILE_NAME}.pending-{}",
        std::process::id()
    ));
    let mut pending = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending_path)?;
    pending.write_all(body.as_bytes())?;
    pending.sync_all()?;
    fs::rename(pending_path, evidence_path)
}

fn manifest_header(
    output_dir: &Path,
    requested_frames: usize,
    artifact: &VariableViewportMotionArtifact,
) -> String {
    let manifest = artifact.manifest();
    format!(
        "kle-storybook opaque root variable-viewport motion artifact\noutput_dir={}\nrequested_frames={}\nsource_frames={}\ndecoded_frames={}\nwidth={}\nheight={}\nsource_viewports={}\nanimated_image_path={}\nvideo_path={}\nkuc_manifest_path={}\ncanonical_manifest_sha256={}\nframe_sequence_sha256={}\nstar_scalar_sequence={:?}\nime_preedit_event_seen={}\nime_commit_event_seen={}\nhit_test_count={}\naccesskit_snapshot_hash={}\n",
        output_dir.display(),
        requested_frames,
        manifest.source_frame_count,
        manifest.decoded_frame_count,
        manifest.width,
        manifest.height,
        manifest.source_viewports.len(),
        manifest.gif_path,
        manifest.mp4_path,
        artifact.manifest_path().display(),
        manifest.canonical_sha256,
        manifest.frame_sequence_sha256,
        manifest.semantic_evidence.star_scalar_sequence,
        manifest.semantic_evidence.ime_preedit_event_seen,
        manifest.semantic_evidence.ime_commit_event_seen,
        manifest.semantic_evidence.hit_test_count,
        manifest.semantic_evidence.accesskit_snapshot_hash,
    )
}

fn append_frames(body: &mut String, frames: &[MotionArtifactFrame]) {
    for (index, frame) in frames.iter().enumerate() {
        body.push_str(&format!(
            "frame[{index}] stage={} scenario_stage={} receipt_record_hash={} opaque_event_cardinality={}\n",
            frame.stage_id,
            frame.scenario_stage,
            frame.receipt_record_hash,
            frame.event_cardinality,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{EVIDENCE_FILE_NAME, invalidate_completion, write_completion_marker};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn completion_marker_is_absent_until_the_complete_evidence_is_written()
    -> Result<(), Box<dyn std::error::Error>> {
        let output = temporary_output("completion");
        fs::create_dir_all(&output)?;
        let evidence = output.join(EVIDENCE_FILE_NAME);
        fs::write(&evidence, "previous complete artifact")?;

        invalidate_completion(&output)?;
        assert!(!evidence.exists());

        write_completion_marker(&output, "complete artifact")?;
        assert_eq!(fs::read_to_string(&evidence)?, "complete artifact");
        fs::remove_dir_all(output)?;
        Ok(())
    }

    #[test]
    fn completion_marker_never_overwrites_an_existing_pending_writer()
    -> Result<(), Box<dyn std::error::Error>> {
        let output = temporary_output("pending");
        fs::create_dir_all(&output)?;
        let pending = output.join(format!(
            ".{EVIDENCE_FILE_NAME}.pending-{}",
            std::process::id()
        ));
        fs::write(&pending, "interrupted writer")?;

        assert!(write_completion_marker(&output, "complete artifact").is_err());
        assert!(!output.join(EVIDENCE_FILE_NAME).exists());
        fs::remove_dir_all(output)?;
        Ok(())
    }

    fn temporary_output(name: &str) -> PathBuf {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "kle-storybook-motion-manifest-{name}-{}-{sequence}",
            std::process::id(),
        ))
    }
}
