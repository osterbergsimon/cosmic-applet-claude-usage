// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, ResetDisplay, Style};
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

/// Time-to-reset context for the soonest-resetting budget.
#[derive(Clone, Copy)]
pub struct ResetInfo {
    /// Fraction of the budget window elapsed (0.0 just reset → 1.0 reset imminent).
    pub elapsed: f32,
    /// Seconds until that reset.
    pub remaining: i64,
}

/// Window lengths for the two budgets, used to turn "seconds remaining" into a
/// fraction-elapsed for the time arcs.
const SESSION_WINDOW: i64 = 5 * 3600;
const WEEKLY_WINDOW: i64 = 7 * 24 * 3600;

pub fn reset_info(sample: &UsageSample, now: i64) -> ResetInfo {
    let (reset, window) = if sample.session_reset <= sample.weekly_reset {
        (sample.session_reset, SESSION_WINDOW)
    } else {
        (sample.weekly_reset, WEEKLY_WINDOW)
    };
    let remaining = (reset - now).max(0);
    let elapsed = (1.0 - remaining as f32 / window as f32).clamp(0.0, 1.0);
    ResetInfo { elapsed, remaining }
}

/// Terse, proximity-coloured countdown: `2h`/`3d` (calm), `38m` (amber as it
/// nears), `soon` (green just before refresh).
fn compact_reset(remaining: i64) -> (String, Color) {
    let neutral = Color::from_rgba(1.0, 1.0, 1.0, 0.7);
    if remaining <= 5 * 60 {
        ("soon".to_string(), color(Level::Green))
    } else if remaining < 30 * 60 {
        (format!("{}m", remaining / 60), color(Level::Amber))
    } else if remaining >= 86_400 {
        (format!("{}d", remaining / 86_400), neutral)
    } else {
        (format!("{}h", remaining / 3_600), neutral)
    }
}

fn colored_text<'a>(s: String, c: Color) -> Element<'a, Message> {
    widget::text(s)
        .size(12)
        .class(cosmic::theme::Text::Color(c))
        .into()
}

/// Append a reset label to the right of the indicator.
fn with_reset_text<'a>(indicator: Element<'a, Message>, s: String, c: Color) -> Element<'a, Message> {
    widget::Row::new()
        .spacing(6)
        .align_y(Alignment::Center)
        .push(indicator)
        .push(colored_text(s, c))
        .into()
}

/// Soft colour halo behind the indicator, fading in as reset approaches
/// (within 30 min). Returns the indicator unchanged when reset is far off.
fn glow_wrap<'a>(indicator: Element<'a, Message>, remaining: i64) -> Element<'a, Message> {
    const NEAR: i64 = 30 * 60;
    if remaining >= NEAR {
        return indicator;
    }
    let t = 1.0 - (remaining.max(0) as f32 / NEAR as f32); // 0 → 1 as reset nears
    // Amber far-ish, green when about to refresh.
    let lvl = if remaining <= 5 * 60 { Level::Green } else { Level::Amber };
    let mut halo = color(lvl);
    halo.a = 0.15 + 0.35 * t;
    widget::container(indicator)
        .padding(4)
        .style(move |_t| widget::container::Style {
            background: Some(halo.into()),
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
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
/// With `reset_display = DualRing` a thin inner arc shows time elapsed toward
/// reset; with `Track` the background track grows with elapsed time.
fn ring<'a>(g: &Gauge, cfg: &Config, dim: bool, reset: Option<&ResetInfo>) -> Element<'a, Message> {
    use std::f32::consts::PI;
    let size = 18.0_f32;
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
    let elapsed = reset.map(|ri| ri.elapsed).unwrap_or(0.0);

    // Track mode: a brighter background arc grows with elapsed time.
    let track_arc = if matches!(cfg.reset_display, ResetDisplay::Track) && reset.is_some() {
        format!(
            r##"<circle cx="{c}" cy="{c}" r="{r:.3}" fill="none" stroke="#ffffff" stroke-opacity="0.32" stroke-width="{sw}" stroke-dasharray="{te:.3} {circ:.3}" transform="rotate(-90 {c} {c})"/>"##,
            te = elapsed * circ,
        )
    } else {
        String::new()
    };

    // Dual ring: a thin inner concentric arc = elapsed time.
    let inner_r = r - 3.5;
    let inner_circ = 2.0 * PI * inner_r;
    let inner_arc = if matches!(cfg.reset_display, ResetDisplay::DualRing) && reset.is_some() {
        format!(
            r##"<circle cx="{c}" cy="{c}" r="{inner_r:.3}" fill="none" stroke="#ffffff" stroke-opacity="0.6" stroke-width="1.5" stroke-linecap="round" stroke-dasharray="{inf:.3} {inner_circ:.3}" transform="rotate(-90 {c} {c})"/>"##,
            inf = elapsed * inner_circ,
        )
    } else {
        String::new()
    };

    // Layers: faint full track, optional time track, usage arc, optional inner arc.
    let doc = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">
<circle cx="{c}" cy="{c}" r="{r:.3}" fill="none" stroke="#ffffff" stroke-opacity="0.15" stroke-width="{sw}"/>
{track_arc}
<circle cx="{c}" cy="{c}" r="{r:.3}" fill="none" stroke="{stroke}" stroke-opacity="{alpha:.2}" stroke-width="{sw}" stroke-linecap="round" stroke-dasharray="{filled:.3} {circ:.3}" transform="rotate(-90 {c} {c})"/>
{inner_arc}
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

pub fn indicator_view<'a>(
    state: &IndicatorState,
    cfg: &Config,
    reset: Option<ResetInfo>,
) -> Element<'a, Message> {
    let (gauges, dim): (&[Gauge], bool) = match state {
        IndicatorState::NoData => {
            // Hollow grey dot when there is no data yet.
            let grey = Color::from_rgba(0.6, 0.6, 0.6, 0.5);
            return widget::Row::new().spacing(4).push(swatch(grey)).into();
        }
        IndicatorState::Live(g) => (g, false),
        IndicatorState::Stale(g) => (g, true),
    };

    let mut row = widget::Row::new().spacing(6).align_y(Alignment::Center);
    for g in gauges {
        let el = match cfg.style {
            Style::ColorDot => dot(g, cfg, dim),
            Style::FillBar | Style::FillColor => bar(g, cfg, dim),
            Style::Ring | Style::RingColor => ring(g, cfg, dim, reset.as_ref()),
        };
        row = row.push(el);
    }
    let indicator: Element<'a, Message> = row.into();

    // Apply the reset-display mode (the click popup always shows reset details).
    let Some(ri) = reset else {
        return indicator;
    };
    let is_ring = matches!(cfg.style, Style::Ring | Style::RingColor);
    match cfg.reset_display {
        ResetDisplay::None => indicator,
        ResetDisplay::Text => with_reset_text(
            indicator,
            format!("resets in {}", format_countdown(ri.remaining)),
            Color::from_rgba(1.0, 1.0, 1.0, 0.7),
        ),
        ResetDisplay::Compact => {
            let (s, c) = compact_reset(ri.remaining);
            with_reset_text(indicator, s, c)
        }
        ResetDisplay::Glow => glow_wrap(indicator, ri.remaining),
        // Ring-specific modes render inside ring(); fall back to compact text
        // on the non-ring styles so the setting is never a no-op.
        ResetDisplay::DualRing | ResetDisplay::Track => {
            if is_ring {
                indicator
            } else {
                let (s, c) = compact_reset(ri.remaining);
                with_reset_text(indicator, s, c)
            }
        }
    }
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

/// A live preview of the indicator using fixed demo data (65% usage, ~18 min to
/// reset, ~60% of the window elapsed) so every style + reset-display combination
/// is visible regardless of the user's real usage. Reflects the current config.
fn settings_preview<'a>(cfg: &Config) -> Element<'a, Message> {
    use crate::indicator::{level_for, Gauge, IndicatorState};
    let g = |v: f32, label: &str| Gauge {
        value: v,
        level: level_for(v, &cfg.thresholds),
        label: label.to_string(),
    };
    let gauges = match cfg.scope {
        crate::config::Scope::Both => vec![g(0.65, "65%"), g(0.30, "30%")],
        _ => vec![g(0.65, "65%")],
    };
    let demo_reset = ResetInfo {
        // Decoupled on purpose: a clearly-partial time arc AND within glow range.
        elapsed: 0.62,
        remaining: 18 * 60,
    };
    let preview = indicator_view(&IndicatorState::Live(gauges), cfg, Some(demo_reset));
    widget::Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(widget::text("Preview").size(12))
        .push(preview)
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
        .push(settings_preview(cfg))
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
        .push(widget::text("Reset display").size(12))
        .push(widget::dropdown(
            &crate::settings::RESET_LABELS[..],
            Some(crate::settings::reset_index(cfg.reset_display)),
            |i| Message::SetResetDisplay(crate::settings::reset_from_index(i)),
        ))
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
    fn reset_info_uses_soonest_budget_window() {
        let now = 2000;
        // session resets sooner (45m) → uses the 5h window.
        let s = UsageSample {
            session: 0.0,
            weekly: 0.0,
            session_reset: now + 60 * 45,
            weekly_reset: now + 60 * 60 * 24 * 4,
            ts: 0,
        };
        let ri = reset_info(&s, now);
        assert_eq!(ri.remaining, 60 * 45);
        // 45m remaining of a 5h window → 1 - 2700/18000 = 0.85 elapsed.
        assert!((ri.elapsed - 0.85).abs() < 0.001);
    }

    #[test]
    fn compact_reset_buckets() {
        assert_eq!(compact_reset(3 * 60).0, "soon");
        assert_eq!(compact_reset(20 * 60).0, "20m");
        assert_eq!(compact_reset(3 * 3600).0, "3h");
        assert_eq!(compact_reset(2 * 86_400).0, "2d");
    }
}
