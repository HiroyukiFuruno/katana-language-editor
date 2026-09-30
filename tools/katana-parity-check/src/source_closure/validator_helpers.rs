use std::collections::BTreeSet;

use super::model::CANONICAL_PROFILE_IDS;

const PLACEHOLDER_VALUES: [&str; 12] = [
    "",
    "placeholder",
    "todo",
    "tbd",
    "stub",
    "dummy",
    "default",
    "n/a",
    "unknown",
    "0",
    "sha256:00",
    "<root>",
];
const KLE_MOUNT_MARKERS: [&str; 3] = ["katana-ui-core", "../katana-ui-core", "/katana-ui-core/"];

pub(super) fn is_placeholder(value: &str) -> bool {
    PLACEHOLDER_VALUES.contains(&value.trim().to_ascii_lowercase().as_str())
}

pub(super) fn is_canonical_profile_set(id: &str) -> bool {
    CANONICAL_PROFILE_IDS
        .iter()
        .any(|canonical| canonical == &id)
}

pub(super) fn is_canonical_profile_set_list(ids: &[String]) -> bool {
    if ids.len() != CANONICAL_PROFILE_IDS.len() {
        return false;
    }
    let mut unique = BTreeSet::new();
    for id in ids {
        if !CANONICAL_PROFILE_IDS.contains(&id.as_str()) || !unique.insert(id) {
            return false;
        }
    }
    true
}

pub(super) fn is_terminal_status(status: &str) -> bool {
    status.trim() == "passing"
}

pub(super) fn kle_mounted_in_katana(path: &str) -> bool {
    let lowered = path.to_ascii_lowercase();
    KLE_MOUNT_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

pub(super) fn duplicates<I, T>(values: I) -> bool
where
    I: IntoIterator<Item = T>,
    T: Ord,
{
    let mut seen = BTreeSet::new();
    values.into_iter().any(|value| !seen.insert(value))
}
