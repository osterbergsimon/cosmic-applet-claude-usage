// SPDX-License-Identifier: GPL-3.0-only

/// Returns the pixel width of the filled portion of a bar indicator.
///
/// `value` is a fraction in [0, 1] (clamped); `full_px` is the total bar width.
pub fn fill_width(value: f32, full_px: f32) -> f32 {
    value.clamp(0.0, 1.0) * full_px
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fill_clamps() {
        assert_eq!(fill_width(0.0, 40.0), 0.0);
        assert_eq!(fill_width(0.5, 40.0), 20.0);
        assert_eq!(fill_width(1.5, 40.0), 40.0);
        assert_eq!(fill_width(-0.2, 40.0), 0.0);
    }
}
