use std::path::Path;

use super::super::fingerprint::sha256_hex;
use super::super::model::{
    CANONICAL_PROFILE_IDS, ManifestRoot, StorybookArtifactEntry, StorybookArtifacts,
};

pub(super) fn complete_storybook_artifacts(
    dir: &Path,
    root: ManifestRoot,
    leaf_id: &str,
) -> Result<StorybookArtifacts, String> {
    let artifacts = CANONICAL_PROFILE_IDS
        .iter()
        .enumerate()
        .map(|(index, profile_id)| {
            let media_path = format!("media/story-format-bold-{}.png", index + 1);
            let media_bytes = one_pixel_png();
            let absolute_media_path = dir.join(&media_path);
            let parent = absolute_media_path
                .parent()
                .ok_or_else(|| "media path has no parent".to_string())?;
            std::fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create media directory {}: {error}",
                    parent.display()
                )
            })?;
            std::fs::write(&absolute_media_path, &media_bytes).map_err(|error| {
                format!(
                    "failed to write media artifact {}: {error}",
                    absolute_media_path.display()
                )
            })?;
            Ok(StorybookArtifactEntry {
                story_id: format!("story-format-bold-{profile_id}"),
                leaf_id: leaf_id.to_string(),
                stage_id: "story-format-bold".to_string(),
                frame_record_id: format!("frame-format-bold-{profile_id}"),
                media_path,
                media_sha256: sha256_hex(&media_bytes),
                profile_id: (*profile_id).to_string(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(StorybookArtifacts { root, artifacts })
}

fn one_pixel_png() -> Vec<u8> {
    vec![
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 4,
        0, 0, 0, 181, 28, 12, 2, 0, 0, 0, 11, 73, 68, 65, 84, 120, 218, 99, 252, 255, 31, 0, 2,
        235, 1, 245, 22, 21, 20, 118, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ]
}
