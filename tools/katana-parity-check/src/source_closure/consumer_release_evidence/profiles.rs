use std::collections::BTreeMap;

use super::super::profile_model::ProfileRecord;
use super::super::root_model::CANONICAL_PROFILE_IDS;
use super::model::ProfileEvidence;

pub(super) fn index_profiles(
    profiles: &[ProfileEvidence],
) -> Result<BTreeMap<&str, &ProfileEvidence>, String> {
    if profiles.len() != CANONICAL_PROFILE_IDS.len() {
        return Err("consumer release bundle must contain exactly three profiles".into());
    }
    let mut indexed = BTreeMap::new();
    for profile in profiles {
        if !CANONICAL_PROFILE_IDS.contains(&profile.profile_id.as_str()) {
            return Err(format!(
                "non-canonical consumer profile: {}",
                profile.profile_id
            ));
        }
        if indexed
            .insert(profile.profile_id.as_str(), profile)
            .is_some()
        {
            return Err(format!(
                "duplicate consumer profile: {}",
                profile.profile_id
            ));
        }
    }
    for profile_id in CANONICAL_PROFILE_IDS {
        if !indexed.contains_key(profile_id) {
            return Err(format!(
                "consumer release bundle is missing profile: {profile_id}"
            ));
        }
    }
    Ok(indexed)
}

pub(super) fn validate_profile_fingerprints(
    source_profiles: &[ProfileRecord],
    bundle_profiles: &BTreeMap<&str, &ProfileEvidence>,
) -> Result<(), String> {
    if source_profiles.len() != CANONICAL_PROFILE_IDS.len() {
        return Err("source profiles must contain exactly three records".into());
    }
    for profile_id in CANONICAL_PROFILE_IDS {
        let mut records = source_profiles
            .iter()
            .filter(|record| record.id == profile_id);
        let record = records.next().ok_or_else(|| {
            format!("source profiles must contain exactly one record for {profile_id}")
        })?;
        if records.next().is_some() {
            return Err(format!(
                "source profiles must contain exactly one record for {profile_id}"
            ));
        }
        let bundle_profile = bundle_profiles
            .get(profile_id)
            .ok_or_else(|| format!("consumer release bundle is missing profile: {profile_id}"))?;
        if record.fingerprint != bundle_profile.profile_fingerprint {
            return Err(format!("profile fingerprint differs for {profile_id}"));
        }
    }
    Ok(())
}

pub(super) fn canonical_required_profile_ids(required: &[String]) -> Result<Vec<&str>, String> {
    let mut ids = required.iter().map(String::as_str).collect::<Vec<_>>();
    if ids.is_empty() {
        return Err("consumer leaf has no required profiles".into());
    }
    if ids.iter().any(|id| !CANONICAL_PROFILE_IDS.contains(id)) {
        return Err("consumer leaf has a non-canonical required profile".into());
    }
    ids.sort_by_key(|profile_id| profile_rank(profile_id));
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("consumer leaf has duplicate required profiles".into());
    }
    Ok(ids)
}

fn profile_rank(profile_id: &str) -> Option<usize> {
    CANONICAL_PROFILE_IDS
        .iter()
        .position(|canonical| canonical == &profile_id)
}

#[cfg(test)]
mod tests {
    use super::canonical_required_profile_ids;

    #[test]
    fn required_profiles_are_canonicalized_and_reject_invalid_input() {
        let ids = vec!["ubuntu-latest".into(), "macos-latest".into()];
        let ordered = canonical_required_profile_ids(&ids)
            .map(|ordered| ordered.into_iter().collect::<Vec<_>>());
        assert_eq!(ordered, Ok(vec!["macos-latest", "ubuntu-latest"]));
        assert!(canonical_required_profile_ids(&["unknown".into()]).is_err());
        assert!(
            canonical_required_profile_ids(&["macos-latest".into(), "macos-latest".into()])
                .is_err()
        );
    }
}
