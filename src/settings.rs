// SPDX-License-Identifier: GPL-3.0-only

//! Pure label/index mapping for the settings dropdowns. No libcosmic types so
//! the mapping can be unit-tested in isolation from the UI.

use crate::config::{ResetDisplay, Scope, Style};

pub const SCOPE_LABELS: [&str; 4] = ["Session (5h)", "Weekly (7d)", "Worst of both", "Both"];
pub const RESET_LABELS: [&str; 6] = ["Off", "Text", "Compact", "Glow", "Dual ring", "Track time"];
pub const STYLE_LABELS: [&str; 6] = [
    "Color dot",
    "Fill bar",
    "Fill color",
    "Ring",
    "Ring (color)",
    "Vertical bar",
];

pub fn scope_index(s: Scope) -> usize {
    match s {
        Scope::Session => 0,
        Scope::Weekly => 1,
        Scope::Worst => 2,
        Scope::Both => 3,
    }
}

pub fn scope_from_index(i: usize) -> Scope {
    match i {
        0 => Scope::Session,
        1 => Scope::Weekly,
        3 => Scope::Both,
        _ => Scope::Worst,
    }
}

pub fn style_index(s: Style) -> usize {
    match s {
        Style::ColorDot => 0,
        Style::FillBar => 1,
        Style::FillColor => 2,
        Style::Ring => 3,
        Style::RingColor => 4,
        Style::VBar => 5,
    }
}

pub fn style_from_index(i: usize) -> Style {
    match i {
        1 => Style::FillBar,
        2 => Style::FillColor,
        3 => Style::Ring,
        4 => Style::RingColor,
        5 => Style::VBar,
        _ => Style::ColorDot,
    }
}

pub fn reset_index(r: ResetDisplay) -> usize {
    match r {
        ResetDisplay::None => 0,
        ResetDisplay::Text => 1,
        ResetDisplay::Compact => 2,
        ResetDisplay::Glow => 3,
        ResetDisplay::DualRing => 4,
        ResetDisplay::Track => 5,
    }
}

pub fn reset_from_index(i: usize) -> ResetDisplay {
    match i {
        1 => ResetDisplay::Text,
        2 => ResetDisplay::Compact,
        3 => ResetDisplay::Glow,
        4 => ResetDisplay::DualRing,
        5 => ResetDisplay::Track,
        _ => ResetDisplay::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_round_trips_every_variant() {
        for s in [Scope::Session, Scope::Weekly, Scope::Worst, Scope::Both] {
            assert_eq!(scope_from_index(scope_index(s)), s);
        }
    }

    #[test]
    fn style_round_trips_every_variant() {
        for s in [
            Style::ColorDot,
            Style::FillBar,
            Style::FillColor,
            Style::Ring,
            Style::RingColor,
            Style::VBar,
        ] {
            assert_eq!(style_from_index(style_index(s)), s);
        }
    }

    #[test]
    fn scope_indices_are_distinct_and_in_range() {
        assert_eq!(scope_index(Scope::Session), 0);
        assert_eq!(scope_index(Scope::Weekly), 1);
        assert_eq!(scope_index(Scope::Worst), 2);
        assert_eq!(scope_index(Scope::Both), 3);
    }

    #[test]
    fn style_indices_are_distinct_and_in_range() {
        assert_eq!(style_index(Style::ColorDot), 0);
        assert_eq!(style_index(Style::FillBar), 1);
        assert_eq!(style_index(Style::FillColor), 2);
        assert_eq!(style_index(Style::Ring), 3);
        assert_eq!(style_index(Style::RingColor), 4);
        assert_eq!(style_index(Style::VBar), 5);
    }

    #[test]
    fn reset_round_trips_every_variant() {
        for r in [
            ResetDisplay::None,
            ResetDisplay::Text,
            ResetDisplay::Compact,
            ResetDisplay::Glow,
            ResetDisplay::DualRing,
            ResetDisplay::Track,
        ] {
            assert_eq!(reset_from_index(reset_index(r)), r);
        }
    }

    #[test]
    fn label_array_lengths() {
        assert_eq!(SCOPE_LABELS.len(), 4);
        assert_eq!(STYLE_LABELS.len(), 6);
        assert_eq!(RESET_LABELS.len(), 6);
    }
}
