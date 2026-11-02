//! `celebtir compact` (SNAP-7): snapshots older than 180 days keep one full
//! snapshot per week, the newest day of each Monday-to-Sunday week. Every
//! day keeps its events.json (CHG-4).

use std::collections::BTreeMap;
use std::path::Path;

use crate::{CliError, Format};

pub const KEEP_DAILY_DAYS: i64 = 180;
const DROPPED: &[&str] = &["facts.json", "scores.json", "run.json"];

/// Monday-based week number: 1970-01-01 was a Thursday.
fn week(days: i64) -> i64 {
    (days + 3).div_euclid(7)
}

pub fn run(data: &Path, as_of: &str, format: Format) -> Result<(), CliError> {
    let today = celebtir_core::date::days(as_of)
        .ok_or_else(|| CliError::Validation(format!("--as-of {as_of} is not YYYY-MM-DD")))?;
    let mut old: BTreeMap<i64, Vec<(i64, std::path::PathBuf)>> = BTreeMap::new();
    for entry in std::fs::read_dir(data.join("snapshots"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let Some(d) = entry
            .file_name()
            .to_str()
            .and_then(celebtir_core::date::days)
        else {
            continue;
        };
        if today - d > KEEP_DAILY_DAYS {
            old.entry(week(d)).or_default().push((d, entry.path()));
        }
    }
    let mut removed = 0;
    for days in old.values_mut() {
        days.sort();
        days.pop(); // the newest day of the week stays whole
        for (_, dir) in days.iter() {
            for f in DROPPED {
                let p = dir.join(f);
                if p.exists() {
                    std::fs::remove_file(&p).map_err(|e| {
                        CliError::Validation(format!("cannot remove {}: {e}", p.display()))
                    })?;
                    removed += 1;
                }
            }
        }
    }
    match format {
        Format::Json => println!("{{\"removed_files\":{removed}}}"),
        Format::Text => println!(
            "compacted: removed {removed} files from snapshots older than {KEEP_DAILY_DAYS} days"
        ),
    }
    Ok(())
}
