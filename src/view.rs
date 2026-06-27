// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, Style};
use crate::indicator::{Gauge, IndicatorState, Level};
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
    // FillBar/FillColor partial-fill rendering is refined in Task 7's polish.
    // For now every style draws a solid colored dot.
    let _fill = match cfg.style {
        Style::ColorDot => 1.0,
        Style::FillBar | Style::FillColor => g.value.clamp(0.0, 1.0),
    };

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
        row = row.push(dot(g, cfg, dim));
    }
    row.into()
}
