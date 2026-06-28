// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, Scope, Thresholds};
use crate::usage::UsageSample;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level { Green, Amber, Red }

#[derive(Debug, Clone)]
pub struct Gauge {
    pub value: f32,
    pub level: Level,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum IndicatorState {
    NoData,
    Live(Vec<Gauge>),
    Stale(Vec<Gauge>),
}

pub fn level_for(value: f32, t: &Thresholds) -> Level {
    if value >= t.red { Level::Red }
    else if value >= t.amber { Level::Amber }
    else { Level::Green }
}

fn gauge(value: f32, t: &Thresholds) -> Gauge {
    Gauge {
        value,
        level: level_for(value, t),
        label: format!("{}%", (value * 100.0).round() as i64),
    }
}

pub fn gauges(sample: &UsageSample, cfg: &Config) -> Vec<Gauge> {
    let t = &cfg.thresholds;
    match cfg.scope {
        Scope::Session => vec![gauge(sample.session, t)],
        Scope::Weekly => vec![gauge(sample.weekly, t)],
        Scope::Worst => vec![gauge(sample.worst(), t)],
        Scope::Both => vec![gauge(sample.session, t), gauge(sample.weekly, t)],
    }
}

pub fn indicator_state(sample: Option<&UsageSample>, now: i64, cfg: &Config) -> IndicatorState {
    match sample {
        None => IndicatorState::NoData,
        Some(s) => {
            let g = gauges(s, cfg);
            if s.is_stale(now, cfg.stale_after) {
                IndicatorState::Stale(g)
            } else {
                IndicatorState::Live(g)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use crate::usage::UsageSample;

    fn cfg(scope: Scope) -> Config {
        Config { scope, ..Config::default() }
    }

    fn sample(session: f32, weekly: f32, ts: i64) -> UsageSample {
        UsageSample { session, weekly, session_reset: 0, weekly_reset: 0, ts }
    }

    #[test]
    fn level_thresholds() {
        let t = Thresholds { amber: 0.50, red: 0.80 };
        assert!(matches!(level_for(0.49, &t), Level::Green));
        assert!(matches!(level_for(0.50, &t), Level::Amber));
        assert!(matches!(level_for(0.79, &t), Level::Amber));
        assert!(matches!(level_for(0.80, &t), Level::Red));
    }

    #[test]
    fn worst_scope_one_gauge() {
        let g = gauges(&sample(0.38, 0.12, 0), &cfg(Scope::Worst));
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].label, "38%");
        assert!(matches!(g[0].level, Level::Green));
    }

    #[test]
    fn session_scope_uses_session() {
        let g = gauges(&sample(0.55, 0.90, 0), &cfg(Scope::Session));
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].label, "55%");
        assert!(matches!(g[0].level, Level::Amber));
    }

    #[test]
    fn both_scope_two_gauges_session_first() {
        let g = gauges(&sample(0.38, 0.12, 0), &cfg(Scope::Both));
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].label, "38%");
        assert_eq!(g[1].label, "12%");
    }

    #[test]
    fn state_no_data() {
        assert!(matches!(indicator_state(None, 100, &cfg(Scope::Worst)), IndicatorState::NoData));
    }

    #[test]
    fn state_live_vs_stale() {
        let s = sample(0.2, 0.1, 1000);
        let mut c = cfg(Scope::Worst);
        c.stale_after = 600;
        assert!(matches!(indicator_state(Some(&s), 1500, &c), IndicatorState::Live(_)));
        assert!(matches!(indicator_state(Some(&s), 2000, &c), IndicatorState::Stale(_)));
    }
}
