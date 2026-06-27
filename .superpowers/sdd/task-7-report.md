# Task 7 Report: Fill Styles + Percent Polish

## TDD Evidence for `fill_width`

### RED phase
Created `src/fill.rs` with `todo!()` stub and the test from the brief:
```
running 1 test
test fill::tests::fill_clamps ... FAILED
```
Failure: `todo!("implement fill_width")` panic. RED confirmed.

### GREEN phase
Implemented:
```rust
pub fn fill_width(value: f32, full_px: f32) -> f32 {
    value.clamp(0.0, 1.0) * full_px
}
```
```
running 1 test
test fill::tests::fill_clamps ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s
```

## Bar Rendering Approach

Added a `bar()` function in `src/view.rs` that builds a nested container pair:
- **Outer container**: 40px wide, white-15%-alpha track background with rounded corners (radius = height/2 = 3px). Used `move` closure to capture `height`.
- **Inner container**: `fill_width(g.value, 40.0)` pixels wide, fill color with same border radius.

**FillColor** uses `color(g.level)` (green/amber/red); **FillBar** uses neutral accent `Color::from_rgb(0.45, 0.65, 0.95)`. Both apply `fill_color.a = 0.45` when `dim` (stale data). `show_percent` appends a `text(g.label)` at size 12 using `widget::Row::new()`.

The per-gauge dispatch in `indicator_view`:
```rust
let el = match cfg.style {
    Style::ColorDot => dot(g, cfg, dim),
    _ => bar(g, cfg, dim),
};
```

### API Reconciliations vs Brief

| Brief | Actual (what compiles) | Resolution |
|---|---|---|
| `widget::Space::new(filled, height)` | `widget::Space::new().width(...).height(...)` | Used chained builder — matches existing `swatch()` pattern |
| `widget::row().spacing(4)` | `widget::Row::new().spacing(4)` | Matched existing `dot()` / `indicator_view()` pattern |
| Outer `.style(\|_t\|` (non-move) | Needed `move \|_t\|` | Rustc E0373: `height` captured by ref in a closure that may outlive it; added `move` |

### Dead Code Removed
Removed the `let _fill = match cfg.style { ... }` binding from `dot()` — it was computed but never used (left as a placeholder by Task 5).

## Build + Full Test Suite Results

```
cargo build --release: Finished `release` profile [optimized] — 1 pre-existing warning (format_countdown dead_code), no new warnings.

cargo test: test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Test count: 16 (prior) → 17 (after adding `fill::tests::fill_clamps`).

## Files Changed
- `src/fill.rs` — new: `fill_width` function + inline unit test
- `src/main.rs` — added `mod fill;`
- `src/view.rs` — added `bar()`, updated dispatch loop, removed dead `_fill` binding from `dot()`
