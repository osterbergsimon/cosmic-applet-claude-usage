// SPDX-License-Identifier: GPL-3.0-only

//! Pure label/index mapping for the settings dropdowns. No libcosmic types so
//! the mapping can be unit-tested in isolation from the UI.

use crate::config::{ResetDisplay, Scope, Style};

pub const SCOPE_LABELS: [&str; 4] = ["Session (5h)", "Weekly (7d)", "Worst of both", "Both"];
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

// Reset-display options offered per style, as `'static` (label, variant) lists
// so the dropdown can borrow them for the element's lifetime. Universal modes
// come first, then any style-specific arc/bar mode, then the universal time
// column. Incompatible modes (e.g. Dual ring on a bar) simply aren't listed.
pub const DOT_RESET_LABELS: [&str; 5] = ["Off", "Text", "Compact", "Glow", "Time column"];
pub const DOT_RESET_VARIANTS: [ResetDisplay; 5] = [
    ResetDisplay::None,
    ResetDisplay::Text,
    ResetDisplay::Compact,
    ResetDisplay::Glow,
    ResetDisplay::TimeColumn,
];
pub const BAR_RESET_LABELS: [&str; 6] =
    ["Off", "Text", "Compact", "Glow", "Track time", "Time column"];
pub const BAR_RESET_VARIANTS: [ResetDisplay; 6] = [
    ResetDisplay::None,
    ResetDisplay::Text,
    ResetDisplay::Compact,
    ResetDisplay::Glow,
    ResetDisplay::Track,
    ResetDisplay::TimeColumn,
];
pub const RING_RESET_LABELS: [&str; 7] = [
    "Off",
    "Text",
    "Compact",
    "Glow",
    "Dual ring",
    "Track time",
    "Time column",
];
pub const RING_RESET_VARIANTS: [ResetDisplay; 7] = [
    ResetDisplay::None,
    ResetDisplay::Text,
    ResetDisplay::Compact,
    ResetDisplay::Glow,
    ResetDisplay::DualRing,
    ResetDisplay::Track,
    ResetDisplay::TimeColumn,
];

/// The (labels, variants) the Reset-display dropdown should offer for `style`.
/// The two slices are parallel: index `i` maps label `i` to variant `i`.
pub fn resets_for(style: Style) -> (&'static [&'static str], &'static [ResetDisplay]) {
    match style {
        Style::Ring | Style::RingColor => (&RING_RESET_LABELS, &RING_RESET_VARIANTS),
        Style::FillBar | Style::FillColor | Style::VBar => {
            (&BAR_RESET_LABELS, &BAR_RESET_VARIANTS)
        }
        Style::ColorDot => (&DOT_RESET_LABELS, &DOT_RESET_VARIANTS),
    }
}

/// Whether `reset` is a valid choice for `style` (used to auto-correct a stored
/// value when the style changes to one that can't render it).
pub fn reset_valid(style: Style, reset: ResetDisplay) -> bool {
    resets_for(style).1.contains(&reset)
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

    const ALL_STYLES: [Style; 6] = [
        Style::ColorDot,
        Style::FillBar,
        Style::FillColor,
        Style::Ring,
        Style::RingColor,
        Style::VBar,
    ];

    #[test]
    fn reset_options_match_style_capabilities() {
        use ResetDisplay::*;
        // Dot: universal modes only.
        assert!(!reset_valid(Style::ColorDot, Track));
        assert!(!reset_valid(Style::ColorDot, DualRing));
        // Bars and vbar: Track yes, Dual ring no.
        for s in [Style::FillBar, Style::FillColor, Style::VBar] {
            assert!(reset_valid(s, Track));
            assert!(!reset_valid(s, DualRing));
        }
        // Rings: both arc modes.
        for s in [Style::Ring, Style::RingColor] {
            assert!(reset_valid(s, Track));
            assert!(reset_valid(s, DualRing));
        }
        // Universal modes (incl. the time column) are valid for every style.
        for s in ALL_STYLES {
            for r in [None, Text, Compact, Glow, TimeColumn] {
                assert!(reset_valid(s, r), "{r:?} should be valid for {s:?}");
            }
        }
    }

    #[test]
    fn reset_labels_and_variants_are_parallel() {
        for s in ALL_STYLES {
            let (labels, variants) = resets_for(s);
            assert_eq!(labels.len(), variants.len());
        }
    }

    #[test]
    fn label_array_lengths() {
        assert_eq!(SCOPE_LABELS.len(), 4);
        assert_eq!(STYLE_LABELS.len(), 6);
    }
}
