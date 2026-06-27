// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, Style};
use crate::indicator::{Gauge, IndicatorState, Level};
use crate::usage::{format_countdown, UsageSample};
use cosmic::iced::{Alignment, Border, Color, Length};
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

/// Neutral accent used by the non-color fill/ring styles.
const ACCENT: Color = Color {
    r: 0.45,
    g: 0.65,
    b: 0.95,
    a: 1.0,
};

fn hex(c: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    )
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
            .align_y(Alignment::Center)
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
            .align_y(Alignment::Center)
            .push(track)
            .push(widget::text(g.label.clone()).size(12))
            .into()
    } else {
        track.into()
    }
}

/// A circular progress ring — a "rolled-up" bar drawn as an inline SVG whose
/// foreground arc length is `value` of the circumference (via stroke-dasharray).
fn ring<'a>(g: &Gauge, cfg: &Config, dim: bool) -> Element<'a, Message> {
    use std::f32::consts::PI;
    let size = 16.0_f32;
    let sw = 2.5_f32; // stroke width
    let c = size / 2.0;
    let r = (size - sw) / 2.0; // keep the stroke inside the viewBox
    let circ = 2.0 * PI * r;
    let filled = g.value.clamp(0.0, 1.0) * circ;
    let fg = match cfg.style {
        Style::RingColor => color(g.level),
        _ => ACCENT,
    };
    let alpha = if dim { 0.45 } else { 1.0 };

    // Background track ring + foreground arc starting at 12 o'clock (rotate -90).
    let doc = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">
<circle cx="{c}" cy="{c}" r="{r:.3}" fill="none" stroke="#ffffff" stroke-opacity="0.18" stroke-width="{sw}"/>
<circle cx="{c}" cy="{c}" r="{r:.3}" fill="none" stroke="{stroke}" stroke-opacity="{alpha:.2}" stroke-width="{sw}" stroke-linecap="round" stroke-dasharray="{filled:.3} {circ:.3}" transform="rotate(-90 {c} {c})"/>
</svg>"##,
        stroke = hex(fg),
    );
    let ring_el = widget::svg(widget::svg::Handle::from_memory(doc.into_bytes()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size));

    if cfg.show_percent {
        widget::Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(ring_el)
            .push(widget::text(g.label.clone()).size(12))
            .into()
    } else {
        ring_el.into()
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
            Style::FillBar | Style::FillColor => bar(g, cfg, dim),
            Style::Ring | Style::RingColor => ring(g, cfg, dim),
        };
        row = row.push(el);
    }
    row.into()
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

/// The click popup contents: both budgets with percent and reset countdowns,
/// plus a button to open the settings panel.
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
        // Small, low-prominence settings link (button::text renders too large/heavy).
        .push(
            widget::button::custom(widget::text("⚙ Settings").size(11))
                .class(cosmic::theme::Button::Text)
                .on_press(Message::ToggleSettings),
        )
        .into()
}

/// The settings panel: dropdowns/toggles/sliders bound to config fields. Each
/// control emits a `Set*` message which mutates and persists the config.
pub fn settings_view<'a>(cfg: &Config) -> Element<'a, Message> {
    use crate::settings::{
        scope_from_index, scope_index, style_from_index, style_index, SCOPE_LABELS, STYLE_LABELS,
    };

    let scope = widget::dropdown(
        &SCOPE_LABELS[..],
        Some(scope_index(cfg.scope)),
        |i| Message::SetScope(scope_from_index(i)),
    );

    let style = widget::dropdown(
        &STYLE_LABELS[..],
        Some(style_index(cfg.style)),
        |i| Message::SetStyle(style_from_index(i)),
    );

    let amber_pct = (cfg.thresholds.amber * 100.0).round() as i64;
    let red_pct = (cfg.thresholds.red * 100.0).round() as i64;
    // The iced slider requires its value type to impl `Into<f64>`, which `u64`
    // does not; use `u32` for the widget and map back to `u64` in the message.
    let stale_mins = (cfg.stale_after / 60).max(1) as u32;

    widget::Column::new()
        .spacing(8)
        .padding(12)
        .push(widget::text("Settings").size(16))
        .push(widget::text("Scope").size(12))
        .push(scope)
        .push(widget::text("Style").size(12))
        .push(style)
        .push(
            widget::toggler(cfg.show_percent)
                .on_toggle(Message::SetShowPercent)
                .label("Show percent".to_string())
                .text_size(14),
        )
        .push(
            widget::toggler(cfg.show_reset)
                .on_toggle(Message::SetShowReset)
                .label("Show reset".to_string())
                .text_size(14),
        )
        .push(widget::text(format!("Amber threshold: {amber_pct}%")).size(12))
        // step 0.01: the slider's default step is 1.0, which on a 0..=1 range
        // would only allow 0% or 100%.
        .push(widget::slider(0.0..=1.0, cfg.thresholds.amber, Message::SetAmber).step(0.01_f32))
        .push(widget::text(format!("Red threshold: {red_pct}%")).size(12))
        .push(widget::slider(0.0..=1.0, cfg.thresholds.red, Message::SetRed).step(0.01_f32))
        .push(widget::text(format!("Stale after: {stale_mins} min")).size(12))
        .push(widget::slider(1..=30u32, stale_mins, |v| {
            Message::SetStaleAfterMins(v as u64)
        }))
        .push(widget::text("History path").size(12))
        .push(
            widget::text_input(
                "~/.claude/usage-history.jsonl",
                cfg.history_path.clone().unwrap_or_default(),
            )
            .on_input(Message::SetHistoryPath),
        )
        .push(widget::button::text("← Back").on_press(Message::ShowInfo))
        .into()
}

#[cfg(test)]
mod ttests {
    use super::*;
    use crate::usage::UsageSample;

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
