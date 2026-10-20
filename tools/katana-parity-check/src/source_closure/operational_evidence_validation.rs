const DATE_LENGTH: usize = 10;
const DATE_MONTH_SEPARATOR: usize = 4;
const DATE_DAY_SEPARATOR: usize = 7;
const CLOCK_LENGTH: usize = 8;
const CLOCK_MINUTE_SEPARATOR: usize = 2;
const CLOCK_SECOND_SEPARATOR: usize = 5;

pub(super) fn validate_timestamp(value: &str) -> Result<(), String> {
    let Some((date, time)) = value.split_once('T') else {
        return Err("generated_at_utc must be RFC3339 UTC".into());
    };
    let Some(time) = time.strip_suffix('Z') else {
        return Err("generated_at_utc must be RFC3339 UTC".into());
    };
    let (clock, fraction) = time
        .split_once('.')
        .map_or((time, None), |parts| (parts.0, Some(parts.1)));
    let valid_date = date.len() == DATE_LENGTH
        && date.as_bytes()[DATE_MONTH_SEPARATOR] == b'-'
        && date.as_bytes()[DATE_DAY_SEPARATOR] == b'-'
        && date.bytes().enumerate().all(|(index, byte)| {
            matches!(index, DATE_MONTH_SEPARATOR | DATE_DAY_SEPARATOR) || byte.is_ascii_digit()
        });
    let valid_clock = clock.len() == CLOCK_LENGTH
        && clock.as_bytes()[CLOCK_MINUTE_SEPARATOR] == b':'
        && clock.as_bytes()[CLOCK_SECOND_SEPARATOR] == b':'
        && clock.bytes().enumerate().all(|(index, byte)| {
            matches!(index, CLOCK_MINUTE_SEPARATOR | CLOCK_SECOND_SEPARATOR)
                || byte.is_ascii_digit()
        });
    let valid_fraction = fraction
        .is_none_or(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()));
    if !valid_date || !valid_clock || !valid_fraction {
        return Err("generated_at_utc must be RFC3339 UTC".into());
    }
    Ok(())
}
