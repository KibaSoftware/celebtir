//! The 7-day change of a daily USD series (VAL-6, methodology v0.3 D5).
//! One definition, used by the collector, the API and the site: two
//! calendar-aligned 7-day windows ending at the newest day; a window with an
//! absent date, or a day below 5 % of the series median, has no total.

use crate::date::{days, ymd};
use crate::snapshot::DailyUsd;

/// A day below this share of the series median (in percent) is treated as
/// missing: DefiLlama reports partial days as near-zero values.
pub const GAP_PCT_OF_MEDIAN: u64 = 5;

/// The series median: the middle element of the sorted values (upper median
/// for even lengths), exactly as the published checker computes it.
pub fn median(series: &[DailyUsd]) -> Option<u64> {
    let mut v: Vec<u64> = series.iter().map(|d| d.usd).collect();
    v.sort_unstable();
    v.get(v.len() / 2).copied()
}

/// `(change in basis points, gap days)`. The change is None when the series
/// is shorter than 14 calendar days, either window has a gap, or the
/// earlier total is zero. Gap days are sorted, oldest first.
pub fn change_7d(series: &[DailyUsd]) -> (Option<i64>, Vec<String>) {
    let Some(last) = series.last().and_then(|d| days(&d.date)) else {
        return (None, Vec::new());
    };
    let median = median(series).unwrap_or(0);
    let by_day: std::collections::BTreeMap<i64, u64> = series
        .iter()
        .filter_map(|d| Some((days(&d.date)?, d.usd)))
        .collect();
    let mut gaps = Vec::new();
    let mut totals = [0u64; 2];
    for (w, total) in totals.iter_mut().enumerate() {
        for i in 0..7 {
            let day = last - (w as i64) * 7 - i;
            match by_day.get(&day) {
                Some(usd) if usd * 100 >= median * GAP_PCT_OF_MEDIAN => *total += usd,
                _ => gaps.push(ymd(day * 86_400)),
            }
        }
    }
    gaps.sort();
    let [cur, prev] = totals;
    let change =
        (gaps.is_empty() && prev > 0).then(|| (cur as i64 - prev as i64) * 10_000 / prev as i64);
    (change, gaps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(values: &[u64]) -> Vec<DailyUsd> {
        values
            .iter()
            .enumerate()
            .map(|(i, usd)| DailyUsd {
                date: ymd((20_000 + i as i64) * 86_400),
                usd: *usd,
            })
            .collect()
    }

    #[test]
    fn val_6_change_7d_compares_two_complete_weeks() {
        let s = series(&[10, 10, 10, 10, 10, 10, 10, 20, 20, 20, 20, 20, 20, 20]);
        assert_eq!(change_7d(&s), (Some(10_000), vec![]));
    }

    #[test]
    fn val_6_change_7d_is_null_over_a_near_zero_day() {
        let mut v = vec![100; 14];
        v[3] = 1;
        let s = series(&v);
        let (change, gaps) = change_7d(&s);
        assert_eq!(change, None);
        assert_eq!(gaps, vec![s[3].date.clone()]);
    }

    #[test]
    fn val_6_change_7d_is_null_when_a_date_is_absent() {
        let mut s = series(&[100; 15]);
        s.remove(5);
        let (change, gaps) = change_7d(&s);
        assert_eq!(change, None);
        assert_eq!(gaps.len(), 1);
    }

    #[test]
    fn val_6_change_7d_needs_fourteen_days() {
        assert_eq!(change_7d(&series(&[5; 10])).0, None);
    }
}
