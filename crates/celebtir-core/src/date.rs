//! Calendar dates from Unix seconds, without a clock or a date crate.

/// Days since 1970-01-01 → (year, month, day), proleptic Gregorian.
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Unix seconds → `YYYY-MM-DD` (UTC).
pub fn ymd(unix_secs: i64) -> String {
    let (y, m, d) = civil(unix_secs.div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// Unix seconds → RFC 3339 UTC with second precision.
pub fn rfc3339(unix_secs: i64) -> String {
    let s = unix_secs.rem_euclid(86_400);
    format!(
        "{}T{:02}:{:02}:{:02}Z",
        ymd(unix_secs),
        s / 3_600,
        (s % 3_600) / 60,
        s % 60
    )
}

/// `YYYY-MM-DD` → days since 1970-01-01, or None if malformed.
pub fn days(ymd: &str) -> Option<i64> {
    let mut parts = ymd.split('-');
    let (y, m, d): (i64, i64, i64) = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let n = era * 146_097 + doe - 719_468;
    (civil(n) == (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)).then_some(n)
}
