// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct UsageSample {
    pub session: f32,
    pub weekly: f32,
    // reset timestamps drive the tooltip/popup countdowns (Task 8).
    pub session_reset: i64,
    pub weekly_reset: i64,
    pub ts: i64,
}

#[derive(Deserialize)]
struct RawLine {
    #[serde(default)] ts: f64,
    #[serde(default)] session: f32,
    #[serde(default)] weekly: f32,
    #[serde(default)] session_reset: i64,
    #[serde(default)] weekly_reset: i64,
}

impl UsageSample {
    pub fn worst(&self) -> f32 {
        self.session.max(self.weekly)
    }

    pub fn is_stale(&self, now: i64, stale_after: u64) -> bool {
        now - self.ts > stale_after as i64
    }

    fn from_raw(r: RawLine) -> Self {
        UsageSample {
            session: r.session,
            weekly: r.weekly,
            session_reset: r.session_reset,
            weekly_reset: r.weekly_reset,
            ts: r.ts as i64,
        }
    }
}

/// Read the last *valid* JSON line. Returns None for missing/empty/all-invalid.
pub fn read_latest(path: &Path) -> Option<UsageSample> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(raw) = serde_json::from_str::<RawLine>(line) {
            return Some(UsageSample::from_raw(raw));
        }
    }
    None
}

/// Human countdown: "now", "45m", "2h 14m", "4d 3h".
pub fn format_countdown(secs_remaining: i64) -> String {
    let s = secs_remaining.max(0);
    if s == 0 {
        return "now".to_string();
    }
    let days = s / 86_400;
    let hours = (s % 86_400) / 3_600;
    let mins = (s % 3_600) / 60;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn worst_is_max_of_session_weekly() {
        let s = UsageSample { session: 0.38, weekly: 0.12, session_reset: 0, weekly_reset: 0, ts: 0 };
        assert_eq!(s.worst(), 0.38);
        let s2 = UsageSample { session: 0.10, weekly: 0.42, session_reset: 0, weekly_reset: 0, ts: 0 };
        assert_eq!(s2.worst(), 0.42);
    }

    #[test]
    fn staleness_boundary() {
        let s = UsageSample { session: 0.0, weekly: 0.0, session_reset: 0, weekly_reset: 0, ts: 1000 };
        assert_eq!(s.is_stale(1000 + 600, 600), false); // exactly at limit = fresh
        assert_eq!(s.is_stale(1000 + 601, 600), true);
    }

    #[test]
    fn reads_last_valid_line() {
        let s = read_latest(Path::new("tests/fixtures/valid.jsonl")).unwrap();
        assert_eq!(s.session, 0.38);
        assert_eq!(s.weekly, 0.12);
        assert_eq!(s.ts, 1782580127);
        assert_eq!(s.session_reset, 1782583199);
    }

    #[test]
    fn falls_back_past_trailing_garbage() {
        let s = read_latest(Path::new("tests/fixtures/trailing_garbage.jsonl")).unwrap();
        assert_eq!(s.session, 0.38);
    }

    #[test]
    fn empty_file_is_none() {
        assert!(read_latest(Path::new("tests/fixtures/empty.jsonl")).is_none());
    }

    #[test]
    fn missing_file_is_none() {
        assert!(read_latest(Path::new("tests/fixtures/does-not-exist.jsonl")).is_none());
    }

    #[test]
    fn countdown_formats() {
        assert_eq!(format_countdown(0), "now");
        assert_eq!(format_countdown(60 * 60 * 2 + 60 * 14), "2h 14m");
        assert_eq!(format_countdown(60 * 45), "45m");
        assert_eq!(format_countdown(60 * 60 * 24 * 4 + 60 * 60 * 3), "4d 3h");
    }
}
