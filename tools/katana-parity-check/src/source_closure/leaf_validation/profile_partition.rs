use std::collections::BTreeSet;

use super::super::model::{CANONICAL_PROFILE_IDS, InactiveProfilePredicate};

pub(super) fn has_complete_profile_partition(
    active_profile_ids: &[String],
    inactive_profile_predicates: &[InactiveProfilePredicate],
) -> bool {
    let canonical = CANONICAL_PROFILE_IDS.into_iter().collect::<BTreeSet<_>>();
    let active = active_profile_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let inactive = inactive_profile_predicates
        .iter()
        .map(|predicate| predicate.profile_id.as_str())
        .collect::<BTreeSet<_>>();
    active.len() == active_profile_ids.len()
        && inactive.len() == inactive_profile_predicates.len()
        && active.is_subset(&canonical)
        && inactive.is_subset(&canonical)
        && active.is_disjoint(&inactive)
        && active.union(&inactive).copied().collect::<BTreeSet<_>>() == canonical
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_complete_cfg_profile_partition() {
        let inactive = vec![InactiveProfilePredicate {
            profile_id: "windows-latest".to_string(),
            predicate: "target_os = windows".to_string(),
        }];
        assert!(has_complete_profile_partition(
            &["macos-latest".to_string(), "ubuntu-latest".to_string()],
            &inactive,
        ));
    }

    #[test]
    fn rejects_overlapping_or_incomplete_cfg_profile_partitions() {
        let overlap = vec![InactiveProfilePredicate {
            profile_id: "windows-latest".to_string(),
            predicate: "target_os = windows".to_string(),
        }];
        assert!(!has_complete_profile_partition(
            &["macos-latest".to_string(), "windows-latest".to_string()],
            &overlap,
        ));
        assert!(!has_complete_profile_partition(
            &["macos-latest".to_string()],
            &overlap,
        ));
    }
}
