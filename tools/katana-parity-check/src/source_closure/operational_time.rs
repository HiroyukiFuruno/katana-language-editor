use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: u64 = 86_400;
const SECONDS_PER_HOUR: u64 = 3_600;
const DAYS_TO_CIVIL_EPOCH: i64 = 719_468;
const NEGATIVE_ERA_OFFSET: i64 = 146_096;
const DAYS_PER_ERA: i64 = 146_097;
const DAYS_PER_LEAP_CYCLE: i64 = 1_460;
const DAYS_PER_CENTURY: i64 = 36_524;
const DAYS_PER_YEAR: i64 = 365;
const YEARS_PER_ERA: i64 = 400;
const YEARS_PER_QUADRENNIUM: i64 = 4;
const YEARS_PER_CENTURY_CYCLE: i64 = 100;
const MONTH_FORMULA_MULTIPLIER: i64 = 5;
const MONTH_FORMULA_OFFSET: i64 = 2;
const DAYS_PER_MONTH_CYCLE: i64 = 153;
const MONTH_DAY_OFFSET: i64 = 2;
const MONTH_CYCLE_DIVISOR: i64 = 5;
const MARCH_BASE_MONTH: i64 = 3;
const MONTHS_BEFORE_MARCH: i64 = 10;
const JANUARY_ADJUSTMENT: i64 = -9;
const LEAP_YEAR_END_MONTH: i64 = 2;
const SECONDS_PER_MINUTE: u64 = 60;

pub(super) fn utc_timestamp() -> Result<String, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before UNIX epoch: {error}"))?
        .as_secs();
    let days = seconds / SECONDS_PER_DAY;
    let day_seconds = seconds % SECONDS_PER_DAY;
    let (year, month, day) = civil_date(days as i64);
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / SECONDS_PER_HOUR,
        (day_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE,
        day_seconds % SECONDS_PER_MINUTE
    ))
}

pub(super) fn civil_date(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + DAYS_TO_CIVIL_EPOCH;
    let era = if z >= 0 { z } else { z - NEGATIVE_ERA_OFFSET } / DAYS_PER_ERA;
    let day_of_era = z - era * DAYS_PER_ERA;
    let year_of_era = (day_of_era - day_of_era / DAYS_PER_LEAP_CYCLE
        + day_of_era / DAYS_PER_CENTURY
        - day_of_era / NEGATIVE_ERA_OFFSET)
        / DAYS_PER_YEAR;
    let year = year_of_era + era * YEARS_PER_ERA;
    let day_of_year = day_of_era
        - (DAYS_PER_YEAR * year_of_era + year_of_era / YEARS_PER_QUADRENNIUM
            - year_of_era / YEARS_PER_CENTURY_CYCLE);
    let month_part =
        (MONTH_FORMULA_MULTIPLIER * day_of_year + MONTH_FORMULA_OFFSET) / DAYS_PER_MONTH_CYCLE;
    let day = day_of_year
        - (DAYS_PER_MONTH_CYCLE * month_part + MONTH_DAY_OFFSET) / MONTH_CYCLE_DIVISOR
        + 1;
    let month = month_part
        + if month_part < MONTHS_BEFORE_MARCH {
            MARCH_BASE_MONTH
        } else {
            JANUARY_ADJUSTMENT
        };
    (year + i64::from(month <= LEAP_YEAR_END_MONTH), month, day)
}
