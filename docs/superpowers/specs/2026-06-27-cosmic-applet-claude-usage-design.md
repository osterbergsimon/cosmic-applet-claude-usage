# Design: cosmic-applet-claude-usage

**Date:** 2026-06-27
**Status:** Approved (design phase)

## Purpose

A minimal COSMIC top-panel applet that shows Claude Code usage at a glance —
always present, never in the way. By default it is a single small color-coded
dot; exact numbers and reset countdowns appear only on hover or click.

## Data source

Claude Code's status line appends one line per render to
`~/.claude/usage-history.jsonl`:

```json
{"ts": 1782580127.5, "session": 0.38, "weekly": 0.12, "session_reset": 1782583199, "weekly_reset": 1783022399}
```

Fields:

- `session` — fraction (0.0–1.0) of the 5-hour session budget used.
- `weekly` — fraction (0.0–1.0) of the 7-day weekly budget used.
- `session_reset` / `weekly_reset` — Unix timestamps when each budget resets.
- `ts` — Unix timestamp the line was written.

The applet reads the **last line** of this file as the current state.

### Freshness constraint

The file only grows while Claude Code is actively rendering its status line.
Between sessions the values are stale-but-valid: usage does not change when
Claude is not in use, so "last known" is correct. Reset timestamps remain
meaningful because they are absolute times. The applet treats data older than
`stale_after` as stale and dims the indicator (see Staleness).

## Technology

Native **libcosmic** applet written in Rust, scaffolded from
`cosmic-applet-template`. This is the only first-class way to live inside the
COSMIC panel with proper theming, popups, and panel integration. Alternatives
(StatusNotifier tray icon, a separate bar such as Waybar) were rejected because
they do not truly live in the COSMIC panel.

**Prerequisite:** Rust toolchain via `rustup` (cargo is not currently installed
on this machine).

## Components

### 1. Data layer (`usage.rs`)

- `UsageSample { session: f32, weekly: f32, session_reset: i64, weekly_reset: i64, ts: i64 }`
- `read_latest(path) -> Result<UsageSample>` — reads and parses the final line.
- `worst(&self) -> f32` — `max(session, weekly)`.
- `is_stale(&self, now, stale_after) -> bool`.
- File watch via inotify (e.g. `notify` crate) for live updates, plus a 30s
  timer tick to refresh reset countdowns and re-evaluate staleness.

### 2. Config (`config.rs`)

Stored via `cosmic-config` (RON). Fields with defaults:

| Field          | Type                              | Default     |
|----------------|-----------------------------------|-------------|
| `scope`        | `session \| weekly \| worst \| both` | `worst`     |
| `style`        | `color-dot \| fill-bar \| fill-color` | `color-dot` |
| `show_percent` | `bool`                            | `false`     |
| `thresholds`   | `{ amber: f32, red: f32 }`        | `0.50 / 0.80` |
| `stale_after`  | seconds                           | `600` (10m) |
| `history_path` | path override                     | `~/.claude/usage-history.jsonl` |

`scope` and `style` are orthogonal. `scope: both` renders two indicators side by
side; `scope: worst` renders one driven by `max(session, weekly)`.

### 3. Bar indicator (`view.rs`)

Renders per `style`, colored per `thresholds`:

- `color-dot` — single dot, color = green/amber/red by value.
- `fill-bar` — small progress bar filled 0–100% by value.
- `fill-color` — progress bar that fills *and* shifts color.

When `show_percent: true`, the numeric value(s) render as text beside the
indicator (e.g. `●38%`, or `●38% ●12%` for `scope: both`).

Color mapping: `value < amber` → green; `amber ≤ value < red` → amber;
`value ≥ red` → red.

### 4. Staleness

If `now - sample.ts > stale_after`, the indicator renders **dimmed/hollow** to
distinguish "stale, last known" from "live." Reset countdowns continue to tick
because they are derived from absolute timestamps, not from `ts`.

### 5. Interaction

- **Hover** → tooltip: `Session 38% · Weekly 12%`.
- **Click** → COSMIC popup (`applet::Context` popup) showing, for each budget, a
  labeled percentage with a small bar, plus a human countdown
  (`resets in 2h 14m` / `resets in 4d 3h`).

## Data flow

```
inotify / 30s tick
        │
        ▼
read_latest(history_path) ──► UsageSample ──► worst()/is_stale()
        │                                          │
        ▼                                          ▼
   Config (scope/style/...) ─────────────► view: bar indicator
                                                   │
                                  hover → tooltip ─┤
                                  click → popup ───┘
```

## Error handling

- Missing/empty/unreadable file → neutral "no data" indicator (hollow grey dot),
  tooltip `No Claude usage data yet`. Not an error state; expected before first
  Claude Code run.
- Malformed final line → fall back to scanning upward for the last valid line;
  if none, treat as no-data.
- Parse of individual fields tolerant: a missing field defaults to 0 / now.

## Testing

- Unit tests for `usage.rs`: parsing valid/partial/malformed lines, `worst()`,
  `is_stale()` boundaries, countdown formatting.
- Config defaults and round-trip (de)serialization.
- View logic: threshold → color mapping is pure and unit-tested; rendering
  smoke-tested manually in the panel.
- Fixture jsonl files under `tests/fixtures/`.

## Out of scope (v1, YAGNI)

Documented as possible future work, deliberately excluded now:

- Threshold-crossing notification / red flash at 80%.
- Historical usage graphs or sparklines.
- Cost / dollar display.
- Multi-account support.

## Open prerequisites

1. Install Rust toolchain (`rustup`).
2. Scaffold from `cosmic-applet-template`.
