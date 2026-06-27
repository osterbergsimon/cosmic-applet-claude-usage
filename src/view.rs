// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, Style};
use crate::indicator::{Gauge, IndicatorState, Level};
use crate::usage::{format_countdown, UsageSample};
use cosmic::iced::{Border, Color, Length};
use cosmic::widget;
use cosmic::Element;

use crate::Message;

const DOT: f32 = 12.0;

fn color(level: Level) -> Color {
    match level {
        Level::Green => Color::from_rgb(0.30, 0.78, 0.36),
        Level::Amber => Color::from_rgb(0.95, 0.70, 0.10),
        Level::Red => Color::from_rgb(0.90, 0.25, 0.25),
    }
}

/// A small filled circle, built from a fixed-size container with a rounded
/// background — the simplest portable primitive across libcosmic revs.
fn swatch<'a>(c: Color) -> Element<'a, Message> {
    widget::container(
        widget::Space::new()
            .width(Length::Fixed(DOT))
            .height(Length::Fixed(DOT)),
    )
    .style(move |_theme| widget::container::Style {
        background: Some(c.into()),
        border: Border {
            radius: (DOT / 2.0).into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn dot<'a>(g: &Gauge, cfg: &Config, dim: bool) -> Element<'a, Message> {
    let mut c = color(g.level);
    if dim {
        c.a = 0.45;
    }
    let circle = swatch(c);
    if cfg.show_percent {
        widget::Row::new()
            .spacing(4)
            .push(circle)
            .push(widget::text(g.label.clone()).size(12))
            .into()
    } else {
        circle
    }
}

fn bar<'a>(g: &Gauge, cfg: &Config, dim: bool) -> Element<'a, Message> {
    let full = 40.0_f32;
    let height = 6.0_f32;
    let filled = crate::fill::fill_width(g.value, full);

    // FillColor uses the level color; FillBar uses a neutral accent.
    let mut fill_color = match cfg.style {
        Style::FillColor => color(g.level),
        _ => Color::from_rgb(0.45, 0.65, 0.95),
    };
    if dim {
        fill_color.a = 0.45;
    }

    let track = widget::container(
        widget::container(
            widget::Space::new()
                .width(Length::Fixed(filled))
                .height(Length::Fixed(height)),
        )
        .style(move |_t| widget::container::Style {
            background: Some(fill_color.into()),
            border: Border {
                radius: (height / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    )
    .width(Length::Fixed(full))
    .style(move |_t| widget::container::Style {
        background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.15).into()),
        border: Border {
            radius: (height / 2.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    if cfg.show_percent {
        widget::Row::new()
            .spacing(4)
            .push(track)
            .push(widget::text(g.label.clone()).size(12))
            .into()
    } else {
        track.into()
    }
}

pub fn indicator_view<'a>(state: &IndicatorState, cfg: &Config) -> Element<'a, Message> {
    let (gauges, dim): (&[Gauge], bool) = match state {
        IndicatorState::NoData => {
            // Hollow grey dot when there is no data yet.
            let grey = Color::from_rgba(0.6, 0.6, 0.6, 0.5);
            return widget::Row::new().spacing(4).push(swatch(grey)).into();
        }
        IndicatorState::Live(g) => (g, false),
        IndicatorState::Stale(g) => (g, true),
    };

    let mut row = widget::Row::new().spacing(6);
    for g in gauges {
        let el = match cfg.style {
            Style::ColorDot => dot(g, cfg, dim),
            _ => bar(g, cfg, dim),
        };
        row = row.push(el);
    }
    row.into()
}

/// One-line hover summary: `Session X% (resets in …) · Weekly Y% (resets in …)`.
pub fn tooltip_text(sample: &UsageSample, now: i64) -> String {
    format!(
        "Session {}% (resets in {}) · Weekly {}% (resets in {})",
        (sample.session * 100.0).round() as i64,
        format_countdown(sample.session_reset - now),
        (sample.weekly * 100.0).round() as i64,
        format_countdown(sample.weekly_reset - now),
    )
}

/// Soonest upcoming reset across both budgets, as "resets in X" — used for the
/// optional reset text beside the indicator when `show_reset` is enabled.
pub fn reset_label(sample: &UsageSample, now: i64) -> String {
    let soonest = sample.session_reset.min(sample.weekly_reset) - now;
    format!("resets in {}", format_countdown(soonest))
}

/// A single budget block for the popup: name + percent, then a reset countdown.
fn budget_row<'a>(name: &str, value: f32, reset: i64, now: i64) -> Element<'a, Message> {
    let pct = (value * 100.0).round() as i64;
    let countdown = format_countdown(reset - now);
    widget::Column::new()
        .spacing(2)
        .push(widget::text(format!("{name}: {pct}%")).size(14))
        .push(widget::text(format!("resets in {countdown}")).size(11))
        .into()
}

/// The click popup contents: both budgets with percent and reset countdowns.
pub fn popup_view<'a>(sample: &UsageSample, now: i64, _cfg: &Config) -> Element<'a, Message> {
    widget::Column::new()
        .spacing(12)
        .padding(12)
        .push(budget_row(
            "Session (5h)",
            sample.session,
            sample.session_reset,
            now,
        ))
        .push(budget_row(
            "Weekly (7d)",
            sample.weekly,
            sample.weekly_reset,
            now,
        ))
        .into()
}

#[cfg(test)]
mod ttests {
    use super::*;
    use crate::usage::UsageSample;

    #[test]
    fn tooltip_renders_both_with_resets() {
        let now = 2000;
        let s = UsageSample {
            session: 0.38,
            weekly: 0.12,
            session_reset: now + 60 * 60 * 2 + 60 * 14, // 2h 14m
            weekly_reset: now + 60 * 60 * 24 * 4 + 60 * 60 * 3, // 4d 3h
            ts: 0,
        };
        assert_eq!(
            tooltip_text(&s, now),
            "Session 38% (resets in 2h 14m) · Weekly 12% (resets in 4d 3h)"
        );
    }

    #[test]
    fn reset_label_uses_soonest() {
        let now = 2000;
        let s = UsageSample {
            session: 0.0,
            weekly: 0.0,
            session_reset: now + 60 * 45, // 45m (soonest)
            weekly_reset: now + 60 * 60 * 24 * 4, // 4d
            ts: 0,
        };
        assert_eq!(reset_label(&s, now), "resets in 45m");
    }
}
