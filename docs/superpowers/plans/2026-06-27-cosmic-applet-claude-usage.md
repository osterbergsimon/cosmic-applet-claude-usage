# cosmic-applet-claude-usage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimal COSMIC top-panel applet that shows Claude Code usage as a quiet color-coded indicator, with exact numbers and reset countdowns on hover/click.

**Architecture:** A native libcosmic applet (iced-based `Application`) that reads the last line of `~/.claude/usage-history.jsonl`, watches it via inotify plus a 30s timer, and renders a configurable indicator (dot/bar) into the panel. Pure logic (parsing, threshold→color, staleness, countdown formatting) lives in plain modules with unit tests; libcosmic wiring is integration-tested by running it in the panel.

**Tech Stack:** Rust, libcosmic (with `applet` feature), `cosmic-config` (RON), `notify` (inotify file watch), `serde`/`serde_json`.

## Global Constraints

- Data source: last line of `~/.claude/usage-history.jsonl` (override via config `history_path`). Line shape: `{"ts": f64, "session": f64, "weekly": f64, "session_reset": i64, "weekly_reset": i64}`.
- `session`/`weekly` are fractions in `[0.0, 1.0]`. `worst = max(session, weekly)`.
- Default config: `scope=worst`, `style=color-dot`, `show_percent=false`, `show_reset=false`, thresholds `amber=0.50 red=0.80`, `stale_after=600s`.
- Color mapping: `value < amber` → green; `amber ≤ value < red` → amber; `value ≥ red` → red.
- libcosmic depends on a relatively recent stable Rust. Pin the libcosmic git rev in `Cargo.toml` and do not float it.
- TDD for all pure logic. Frequent commits. No `unwrap()` on I/O paths — degrade to a no-data state instead.

---

### Task 1: Scaffold project, toolchain, and a static applet in the panel

**Deliverable:** An applet that builds, installs, and shows a static placeholder dot in the COSMIC panel.

**Files:**
- Create: `flake.nix`, `Cargo.toml`, `src/main.rs`, `data/co.osterberg.ClaudeUsage.desktop`, `justfile`, `README.md`, `.gitignore`

**Interfaces:**
- Produces: a binary named `cosmic-applet-claude-usage`; desktop entry id `co.osterberg.ClaudeUsage`.

**NixOS note:** This machine is a NixOS flake (`~/dotfiles`, host `eclipse`, home-manager user `tux`). Rust is NOT globally installed. The toolchain and libcosmic's native build/runtime deps come from a project `flake.nix` devShell entered with `nix develop`. There is no `rustup`, and there is no `sudo install` to `/usr` — dev installs go to `~/.local`, and the permanent install is a Nix derivation (Task 9).

- [ ] **Step 1: Create the project `flake.nix` devShell (replaces rustup)**

```nix
{
  description = "COSMIC applet showing Claude usage";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        # Native deps libcosmic/iced/winit need to build and run on Wayland.
        runtimeLibs = with pkgs; [
          wayland libxkbcommon vulkan-loader libGL
          fontconfig freetype expat
        ];
        nativeDeps = with pkgs; [ pkg-config makeWrapper ];
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = nativeDeps ++ (with pkgs; [ rustc cargo rustfmt clippy just ]);
          buildInputs = runtimeLibs;
          # winit/iced dlopen wayland & vulkan at runtime; expose them.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
          RUST_BACKTRACE = "1";
        };
      });
}
```

- [ ] **Step 1b: Enter the dev shell and confirm the toolchain**

```bash
cd ~/code/cosmic-applet-claude-usage
nix develop -c cargo --version   # expect: cargo 1.x.y from nixpkgs 26.05
```

All `cargo`/`just` commands in later tasks run inside `nix develop -c …` (or after `nix develop` once interactively).

- [ ] **Step 2: Fetch the official template as the structural reference**

```bash
git clone https://github.com/pop-os/cosmic-applet-template /tmp/cosmic-applet-template
ls /tmp/cosmic-applet-template/src   # note exact Application trait wiring + generated names
```

Use the template's `src/main.rs`, `src/window.rs`, and `Cargo.toml` as the canonical pattern for the libcosmic API in this libcosmic rev. Where this plan's code and the template disagree on an exact symbol name (e.g. `Core` import path, `applet::run` signature), follow the template — note any divergence in the commit message.

- [ ] **Step 3: Write `.gitignore`**

```gitignore
/target
```

- [ ] **Step 4: Write `Cargo.toml`**

```toml
[package]
name = "cosmic-applet-claude-usage"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
notify = "6"
dirs = "5"

[dependencies.libcosmic]
git = "https://github.com/pop-os/libcosmic"
# Pin to the rev that the cloned template's Cargo.toml references. Replace REV below.
rev = "REV"
default-features = false
features = ["applet", "tokio", "wayland", "multi-window"]
```

Pin `rev` to exactly what `/tmp/cosmic-applet-template/Cargo.toml` uses so the API matches the template you are following.

- [ ] **Step 5: Write a minimal `src/main.rs` that renders a static dot**

```rust
use cosmic::app::Core;
use cosmic::iced::{Length, Subscription};
use cosmic::{applet, Application, Element};

fn main() -> cosmic::iced::Result {
    applet::run::<Window>(())
}

struct Window {
    core: Core,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application for Window {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "co.osterberg.ClaudeUsage";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: ()) -> (Self, cosmic::app::Task<Message>) {
        (Window { core }, cosmic::app::Task::none())
    }

    fn update(&mut self, _message: Message) -> cosmic::app::Task<Message> {
        cosmic::app::Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn view(&self) -> Element<Message> {
        // Static placeholder: a small green circle, sized to the panel.
        self.core
            .applet
            .icon_button("display-symbolic") // temporary stand-in glyph
            .on_press_down(())
            .into()
    }
}
```

If `icon_button` signature differs in the pinned rev, match the template's `view`. The goal of this task is only "something appears in the panel."

- [ ] **Step 6: Write the desktop entry `data/co.osterberg.ClaudeUsage.desktop`**

```ini
[Desktop Entry]
Name=Claude Usage
Type=Application
Exec=cosmic-applet-claude-usage
Icon=display-symbolic
Terminal=false
Categories=COSMIC;
Keywords=COSMIC;Applet;
NoDisplay=true
X-CosmicApplet=true
```

- [ ] **Step 7: Write `justfile` for build + dev install to `~/.local`**

On NixOS we do not write to `/usr`. The dev-install target drops the binary in `~/.local/bin` and the desktop entry in `~/.local/share/applications`, both of which COSMIC and the user PATH already include. The `Exec=` is rewritten to the absolute `~/.local/bin` path so the panel launches the dev build regardless of PATH.

```just
bin-dst := env_var('HOME') / '.local/bin/cosmic-applet-claude-usage'
desktop-dst := env_var('HOME') / '.local/share/applications/co.osterberg.ClaudeUsage.desktop'

build:
    cargo build --release

# Dev install (no sudo, no /usr) — for iterating before the Nix derivation exists.
install-dev: build
    install -Dm0755 target/release/cosmic-applet-claude-usage {{bin-dst}}
    mkdir -p $(dirname {{desktop-dst}})
    sed 's|^Exec=.*|Exec={{bin-dst}}|' data/co.osterberg.ClaudeUsage.desktop > {{desktop-dst}}

test:
    cargo test
```

- [ ] **Step 8: Build inside the dev shell**

Run: `nix develop -c cargo build --release`
Expected: compiles to `target/release/cosmic-applet-claude-usage` (first build is slow — it fetches and builds libcosmic).

- [ ] **Step 9: Dev-install and add to the panel, then verify visually**

```bash
nix develop -c just install-dev
```
Then add `co.osterberg.ClaudeUsage` to the panel: COSMIC Settings → Desktop → Panel → Applets, or edit the panel RON config and restart `cosmic-panel`. Verify the placeholder dot appears in the top bar. (If the entry does not show, confirm `~/.local/share/applications` is in `XDG_DATA_DIRS`; under home-manager it normally is.)

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: scaffold libcosmic applet with static panel indicator"
```

---

### Task 2: Config module

**Deliverable:** A typed config with defaults, loaded via cosmic-config, unit-tested.

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs` (add `mod config;`)
- Test: inline `#[cfg(test)]` in `src/config.rs`

**Interfaces:**
- Produces:
  - `enum Scope { Session, Weekly, Worst, Both }`
  - `enum Style { ColorDot, FillBar, FillColor }`
  - `struct Thresholds { amber: f32, red: f32 }`
  - `struct Config { scope: Scope, style: Style, show_percent: bool, show_reset: bool, thresholds: Thresholds, stale_after: u64, history_path: Option<String> }`
  - `Config::default()`, `Config::history_path_resolved(&self) -> PathBuf`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let c = Config::default();
        assert!(matches!(c.scope, Scope::Worst));
        assert!(matches!(c.style, Style::ColorDot));
        assert_eq!(c.show_percent, false);
        assert_eq!(c.show_reset, false);
        assert_eq!(c.thresholds.amber, 0.50);
        assert_eq!(c.thresholds.red, 0.80);
        assert_eq!(c.stale_after, 600);
        assert!(c.history_path.is_none());
    }

    #[test]
    fn resolves_default_history_path() {
        let c = Config::default();
        let p = c.history_path_resolved();
        assert!(p.ends_with(".claude/usage-history.jsonl"));
    }

    #[test]
    fn resolves_override_history_path() {
        let mut c = Config::default();
        c.history_path = Some("/x/y.jsonl".into());
        assert_eq!(c.history_path_resolved(), std::path::PathBuf::from("/x/y.jsonl"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `nix develop -c cargo test config::tests`
Expected: FAIL (module/types not defined).

- [ ] **Step 3: Implement `src/config.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope { Session, Weekly, Worst, Both }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Style { ColorDot, FillBar, FillColor }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Thresholds { pub amber: f32, pub red: f32 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub scope: Scope,
    pub style: Style,
    pub show_percent: bool,
    pub show_reset: bool,
    pub thresholds: Thresholds,
    pub stale_after: u64,
    pub history_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scope: Scope::Worst,
            style: Style::ColorDot,
            show_percent: false,
            show_reset: false,
            thresholds: Thresholds { amber: 0.50, red: 0.80 },
            stale_after: 600,
            history_path: None,
        }
    }
}

impl Config {
    pub fn history_path_resolved(&self) -> PathBuf {
        if let Some(p) = &self.history_path {
            return PathBuf::from(p);
        }
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        home.join(".claude/usage-history.jsonl")
    }
}
```

- [ ] **Step 4: Add `mod config;` to `src/main.rs`**

```rust
mod config;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `nix develop -c cargo test config::tests`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat: typed config with spec defaults and path resolution"
```

---

### Task 3: Usage data layer

**Deliverable:** Parsing the jsonl tail into a `UsageSample` with `worst`, `is_stale`, and countdown formatting — fully unit-tested.

**Files:**
- Create: `src/usage.rs`, `tests/fixtures/valid.jsonl`, `tests/fixtures/trailing_garbage.jsonl`, `tests/fixtures/empty.jsonl`
- Modify: `src/main.rs` (add `mod usage;`)
- Test: inline `#[cfg(test)]` in `src/usage.rs`

**Interfaces:**
- Produces:
  - `struct UsageSample { session: f32, weekly: f32, session_reset: i64, weekly_reset: i64, ts: i64 }`
  - `UsageSample::worst(&self) -> f32`
  - `UsageSample::is_stale(&self, now: i64, stale_after: u64) -> bool`
  - `fn read_latest(path: &Path) -> Option<UsageSample>`
  - `fn format_countdown(secs_remaining: i64) -> String`

- [ ] **Step 1: Write fixtures**

`tests/fixtures/valid.jsonl` (two lines; last line is the truth):
```
{"ts": 1782487115.6, "session": 0.07, "weekly": 0.01, "session_reset": 1782502199, "weekly_reset": 1783022399}
{"ts": 1782580127.5, "session": 0.38, "weekly": 0.12, "session_reset": 1782583199, "weekly_reset": 1783022399}
```

`tests/fixtures/trailing_garbage.jsonl` (last line malformed; must fall back to prior valid line):
```
{"ts": 1782580127.5, "session": 0.38, "weekly": 0.12, "session_reset": 1782583199, "weekly_reset": 1783022399}
{not valid json
```

`tests/fixtures/empty.jsonl`: (zero bytes — create with `: > tests/fixtures/empty.jsonl`)

- [ ] **Step 2: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn worst_is_max_of_session_weekly() {
        let s = UsageSample { session: 0.38, weekly: 0.12, session_reset: 0, weekly_reset: 0, ts: 0 };
        assert_eq!(s.worst(), 0.38);
        let s2 = UsageSample { session: 0.10, weekly: 0.42, session_reset: 0, weekly_reset: 0, ts: 0 };
        assert_eq!(s2.worst(), 0.42);
    }

    #[test]
    fn staleness_boundary() {
        let s = UsageSample { session: 0.0, weekly: 0.0, session_reset: 0, weekly_reset: 0, ts: 1000 };
        assert_eq!(s.is_stale(1000 + 600, 600), false); // exactly at limit = fresh
        assert_eq!(s.is_stale(1000 + 601, 600), true);
    }

    #[test]
    fn reads_last_valid_line() {
        let s = read_latest(Path::new("tests/fixtures/valid.jsonl")).unwrap();
        assert_eq!(s.session, 0.38);
        assert_eq!(s.weekly, 0.12);
        assert_eq!(s.ts, 1782580127);
        assert_eq!(s.session_reset, 1782583199);
    }

    #[test]
    fn falls_back_past_trailing_garbage() {
        let s = read_latest(Path::new("tests/fixtures/trailing_garbage.jsonl")).unwrap();
        assert_eq!(s.session, 0.38);
    }

    #[test]
    fn empty_file_is_none() {
        assert!(read_latest(Path::new("tests/fixtures/empty.jsonl")).is_none());
    }

    #[test]
    fn missing_file_is_none() {
        assert!(read_latest(Path::new("tests/fixtures/does-not-exist.jsonl")).is_none());
    }

    #[test]
    fn countdown_formats() {
        assert_eq!(format_countdown(0), "now");
        assert_eq!(format_countdown(60 * 60 * 2 + 60 * 14), "2h 14m");
        assert_eq!(format_countdown(60 * 45), "45m");
        assert_eq!(format_countdown(60 * 60 * 24 * 4 + 60 * 60 * 3), "4d 3h");
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `nix develop -c cargo test usage::tests`
Expected: FAIL (types/functions undefined).

- [ ] **Step 4: Implement `src/usage.rs`**

```rust
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct UsageSample {
    pub session: f32,
    pub weekly: f32,
    pub session_reset: i64,
    pub weekly_reset: i64,
    pub ts: i64,
}

#[derive(Deserialize)]
struct RawLine {
    #[serde(default)] ts: f64,
    #[serde(default)] session: f32,
    #[serde(default)] weekly: f32,
    #[serde(default)] session_reset: i64,
    #[serde(default)] weekly_reset: i64,
}

impl UsageSample {
    pub fn worst(&self) -> f32 {
        self.session.max(self.weekly)
    }

    pub fn is_stale(&self, now: i64, stale_after: u64) -> bool {
        now - self.ts > stale_after as i64
    }

    fn from_raw(r: RawLine) -> Self {
        UsageSample {
            session: r.session,
            weekly: r.weekly,
            session_reset: r.session_reset,
            weekly_reset: r.weekly_reset,
            ts: r.ts as i64,
        }
    }
}

/// Read the last *valid* JSON line. Returns None for missing/empty/all-invalid.
pub fn read_latest(path: &Path) -> Option<UsageSample> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(raw) = serde_json::from_str::<RawLine>(line) {
            return Some(UsageSample::from_raw(raw));
        }
    }
    None
}

/// Human countdown: "now", "45m", "2h 14m", "4d 3h".
pub fn format_countdown(secs_remaining: i64) -> String {
    let s = secs_remaining.max(0);
    if s == 0 {
        return "now".to_string();
    }
    let days = s / 86_400;
    let hours = (s % 86_400) / 3_600;
    let mins = (s % 3_600) / 60;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {:02}m", hours, mins).replace(" 0", " ").replace("h 0", "h ")
    } else {
        format!("{}m", mins)
    }
}
```

Note: the `2h 14m` test expects no zero-padding. Implement `format_countdown`'s hour branch as `format!("{}h {}m", hours, mins)` (simpler and matches the test exactly):

```rust
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
```

Use this simpler hour branch; delete the `.replace(...)` version above.

- [ ] **Step 5: Add `mod usage;` to `src/main.rs`**

```rust
mod usage;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `nix develop -c cargo test usage::tests`
Expected: PASS (7 tests).

- [ ] **Step 7: Commit**

```bash
git add src/usage.rs src/main.rs tests/fixtures
git commit -m "feat: usage jsonl tail parser with worst/staleness/countdown"
```

---

### Task 4: Indicator state — threshold→color and render kind

**Deliverable:** Pure functions mapping a sample + config into the visual parameters the view needs, unit-tested. No libcosmic types involved, so this is fully testable.

**Files:**
- Create: `src/indicator.rs`
- Modify: `src/main.rs` (add `mod indicator;`)
- Test: inline `#[cfg(test)]` in `src/indicator.rs`

**Interfaces:**
- Consumes: `config::{Config, Scope, Style, Thresholds}`, `usage::UsageSample`
- Produces:
  - `enum Level { Green, Amber, Red }`
  - `fn level_for(value: f32, t: &Thresholds) -> Level`
  - `struct Gauge { value: f32, level: Level, label: String }` (label = e.g. `"38%"`)
  - `fn gauges(sample: &UsageSample, cfg: &Config) -> Vec<Gauge>` (1 gauge for session/weekly/worst, 2 for both, in order session-then-weekly)
  - `enum IndicatorState { NoData, Live(Vec<Gauge>), Stale(Vec<Gauge>) }`
  - `fn indicator_state(sample: Option<&UsageSample>, now: i64, cfg: &Config) -> IndicatorState`

- [ ] **Step 1: Write failing tests**

```rust
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `nix develop -c cargo test indicator::tests`
Expected: FAIL (undefined).

- [ ] **Step 3: Implement `src/indicator.rs`**

```rust
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
```

- [ ] **Step 4: Add `mod indicator;` to `src/main.rs`**

```rust
mod indicator;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `nix develop -c cargo test indicator::tests`
Expected: PASS (6 tests).

- [ ] **Step 6: Commit**

```bash
git add src/indicator.rs src/main.rs
git commit -m "feat: pure indicator state (threshold->color, scope->gauges, staleness)"
```

---

### Task 5: Render the indicator in the panel from real data

**Deliverable:** The applet reads the file once at startup and renders the real indicator (dot/bar, color, optional percent, dim-when-stale) instead of the placeholder.

**Files:**
- Modify: `src/main.rs`
- Create: `src/view.rs`

**Interfaces:**
- Consumes: `indicator::{IndicatorState, Gauge, Level}`, `config::{Config, Style}`
- Produces: `fn indicator_view<'a>(state: &IndicatorState, cfg: &Config) -> Element<'a, Message>`

- [ ] **Step 1: Implement `src/view.rs`**

```rust
use crate::config::{Config, Style};
use crate::indicator::{Gauge, IndicatorState, Level};
use cosmic::iced::{Color, Length};
use cosmic::widget;
use cosmic::Element;

use crate::Message;

fn color(level: Level) -> Color {
    match level {
        Level::Green => Color::from_rgb(0.30, 0.78, 0.36),
        Level::Amber => Color::from_rgb(0.95, 0.70, 0.10),
        Level::Red => Color::from_rgb(0.90, 0.25, 0.25),
    }
}

fn dot<'a>(g: &Gauge, cfg: &Config, dim: bool) -> Element<'a, Message> {
    let mut c = color(g.level);
    if dim {
        c.a = 0.45;
    }
    // A small filled circle. Use a container with a rounded background as the
    // simplest portable primitive across libcosmic revs.
    let size = 12.0_f32;
    let fill = match cfg.style {
        Style::ColorDot => 1.0,
        Style::FillBar | Style::FillColor => g.value.clamp(0.0, 1.0),
    };
    let _ = fill; // bar fill applied in fill styles below; dot ignores it.

    let circle = widget::container(widget::Space::new(Length::Fixed(size), Length::Fixed(size)))
        .style(move |_theme| widget::container::Style {
            background: Some(c.into()),
            border: cosmic::iced::Border {
                radius: (size / 2.0).into(),
                ..Default::default()
            },
            ..Default::default()
        });

    if cfg.show_percent {
        widget::row()
            .spacing(4)
            .push(circle)
            .push(widget::text(g.label.clone()).size(12))
            .into()
    } else {
        circle.into()
    }
}

pub fn indicator_view<'a>(state: &IndicatorState, cfg: &Config) -> Element<'a, Message> {
    let (gauges, dim): (&[Gauge], bool) = match state {
        IndicatorState::NoData => {
            // Hollow grey dot when there is no data yet.
            let grey = Gauge { value: 0.0, level: Level::Green, label: String::new() };
            let mut row = widget::row().spacing(4);
            let c = Color::from_rgba(0.6, 0.6, 0.6, 0.5);
            row = row.push(
                widget::container(widget::Space::new(Length::Fixed(12.0), Length::Fixed(12.0)))
                    .style(move |_t| widget::container::Style {
                        background: Some(c.into()),
                        border: cosmic::iced::Border { radius: 6.0.into(), ..Default::default() },
                        ..Default::default()
                    }),
            );
            let _ = grey;
            return row.into();
        }
        IndicatorState::Live(g) => (g, false),
        IndicatorState::Stale(g) => (g, true),
    };

    let mut row = widget::row().spacing(6);
    for g in gauges {
        row = row.push(dot(g, cfg, dim));
    }
    row.into()
}
```

Reconcile widget paths (`widget::container::Style`, `Border`) against the pinned libcosmic rev; the template's `view` shows the exact import names. The `FillBar`/`FillColor` partial-fill rendering is refined in Task 7's polish — for now a colored dot for every style is acceptable to keep this task's deliverable testable-by-eye.

- [ ] **Step 2: Wire real data + state into `src/main.rs`**

```rust
mod config;
mod usage;
mod indicator;
mod view;

use cosmic::app::Core;
use cosmic::iced::Subscription;
use cosmic::{applet, Application, Element};

use config::Config;
use indicator::{indicator_state, IndicatorState};
use usage::UsageSample;

fn main() -> cosmic::iced::Result {
    applet::run::<Window>(())
}

struct Window {
    core: Core,
    config: Config,
    sample: Option<UsageSample>,
    now: i64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Reload,
    TogglePopup, // used in Task 8
}

fn unix_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

impl Application for Window {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "co.osterberg.ClaudeUsage";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: ()) -> (Self, cosmic::app::Task<Message>) {
        let config = Config::default();
        let sample = usage::read_latest(&config.history_path_resolved());
        (Window { core, config, sample, now: unix_now() }, cosmic::app::Task::none())
    }

    fn update(&mut self, message: Message) -> cosmic::app::Task<Message> {
        match message {
            Message::Reload => {
                self.sample = usage::read_latest(&self.config.history_path_resolved());
                self.now = unix_now();
            }
            Message::TogglePopup => {}
        }
        cosmic::app::Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none() // replaced in Task 6
    }

    fn view(&self) -> Element<Message> {
        let state: IndicatorState = indicator_state(self.sample.as_ref(), self.now, &self.config);
        let inner = view::indicator_view(&state, &self.config);
        // Wrap in the applet's autosize/button so it sizes to the panel.
        self.core.applet.applet_button(inner, Message::TogglePopup)
    }
}
```

The exact panel-wrapping call (`applet_button` here) must match the template — substitute the template's equivalent that produces a press target sized to the panel. If no such helper exists in the pinned rev, wrap `inner` in `self.core.applet.autosize_window(...)` plus a `widget::button`.

- [ ] **Step 3: Build and verify in panel**

Run: `nix develop -c just install-dev`
Then restart the panel (log out/in or `pkill cosmic-panel`). Expected: the dot now reflects your real `worst` usage color (green at low usage), or a hollow grey dot if the file is missing.

- [ ] **Step 4: Run the full test suite**

Run: `nix develop -c cargo test`
Expected: PASS (all prior unit tests still green).

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/view.rs
git commit -m "feat: render real usage indicator from jsonl at startup"
```

---

### Task 6: Live updates — inotify watch + 30s tick

**Deliverable:** The indicator updates automatically when the jsonl changes and re-evaluates staleness/countdowns every 30s, without restarting the applet.

**Files:**
- Create: `src/watch.rs`
- Modify: `src/main.rs` (`subscription`)

**Interfaces:**
- Consumes: `Message::Reload`
- Produces: `fn file_subscription(path: PathBuf) -> Subscription<Message>` (emits `Message::Reload` on file change) and a time-based tick.

- [ ] **Step 1: Implement `src/watch.rs`**

```rust
use cosmic::iced::futures::{SinkExt, Stream};
use cosmic::iced::stream;
use notify::{Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;

use crate::Message;

/// Emit Message::Reload whenever the watched file (or its parent dir) changes.
pub fn file_stream(path: PathBuf) -> impl Stream<Item = Message> {
    stream::channel(16, move |mut output| async move {
        let (tx, rx) = mpsc::channel::<()>();
        // Watch the parent dir: editors/append may replace the inode.
        let watch_dir = path.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let mut watcher = match notify::recommended_watcher(move |res: notify::Result<Event>| {
            if res.is_ok() {
                let _ = tx.send(());
            }
        }) {
            Ok(w) => w,
            Err(_) => return,
        };
        if watcher.watch(&watch_dir, RecursiveMode::NonRecursive).is_err() {
            return;
        }
        // Keep watcher alive for the stream's lifetime.
        loop {
            // Block on the std channel in a blocking task to avoid stalling the runtime.
            let got = tokio::task::block_in_place(|| rx.recv());
            if got.is_err() {
                break;
            }
            let _ = output.send(Message::Reload).await;
        }
        drop(watcher);
    })
}
```

If `block_in_place` is unavailable with the configured runtime, swap to `notify`'s async variant or poll the file mtime in the tick subscription instead (see Step 2 fallback note).

- [ ] **Step 2: Wire subscriptions in `src/main.rs`**

```rust
mod watch;
use cosmic::iced::time;
use std::time::Duration;

// inside impl Application:
fn subscription(&self) -> Subscription<Message> {
    let path = self.config.history_path_resolved();
    let file = Subscription::run_with_id(
        "claude-usage-file",
        watch::file_stream(path),
    );
    let tick = time::every(Duration::from_secs(30)).map(|_| Message::Reload);
    Subscription::batch([file, tick])
}
```

Fallback if the inotify stream proves flaky on this libcosmic rev: drop `file` and rely on the 30s `tick` alone — `Message::Reload` already re-reads the file, so a 30s worst-case latency is acceptable for usage that only changes while you are actively working. Note the choice in the commit message.

- [ ] **Step 3: Build and verify live update**

Run: `nix develop -c just install-dev`, restart panel. Then append a line to the file and confirm the dot updates within ~1s (inotify) or ≤30s (tick fallback):

```bash
echo '{"ts": '"$(date +%s)"'.0, "session": 0.85, "weekly": 0.2, "session_reset": '"$(date +%s)"', "weekly_reset": '"$(date +%s)"'}' >> ~/.claude/usage-history.jsonl
```
Expected: dot turns red (0.85 ≥ 0.80).

- [ ] **Step 4: Run tests**

Run: `nix develop -c cargo test`
Expected: PASS (unchanged unit tests).

- [ ] **Step 5: Commit**

```bash
git add src/watch.rs src/main.rs
git commit -m "feat: live file-watch + 30s tick subscriptions"
```

---

### Task 7: Fill styles + percent polish in the view

**Deliverable:** `fill-bar` and `fill-color` styles render a real partial-fill bar; `color-dot` stays a dot. Pure fill-fraction math is unit-tested.

**Files:**
- Modify: `src/view.rs`
- Create: `src/fill.rs`
- Modify: `src/main.rs` (`mod fill;`)
- Test: inline `#[cfg(test)]` in `src/fill.rs`

**Interfaces:**
- Produces: `fn fill_width(value: f32, full_px: f32) -> f32` (clamped 0..=full_px)

- [ ] **Step 1: Write failing test**

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `nix develop -c cargo test fill::tests`
Expected: FAIL.

- [ ] **Step 3: Implement `src/fill.rs`**

```rust
pub fn fill_width(value: f32, full_px: f32) -> f32 {
    (value.clamp(0.0, 1.0)) * full_px
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `nix develop -c cargo test fill::tests`
Expected: PASS.

- [ ] **Step 5: Render bar styles in `src/view.rs`**

Replace the `dot` function body's style handling so `FillBar`/`FillColor` draw a track + fill:

```rust
fn bar<'a>(g: &Gauge, cfg: &Config, dim: bool) -> Element<'a, Message> {
    use crate::config::Style;
    let full = 40.0_f32;
    let height = 6.0_f32;
    let filled = crate::fill::fill_width(g.value, full);

    // FillColor uses the level color; FillBar uses a neutral accent.
    let mut fill_color = match cfg.style {
        Style::FillColor => color(g.level),
        _ => Color::from_rgb(0.45, 0.65, 0.95),
    };
    if dim { fill_color.a = 0.45; }

    let track = widget::container(
        widget::container(widget::Space::new(
            cosmic::iced::Length::Fixed(filled),
            cosmic::iced::Length::Fixed(height),
        ))
        .style(move |_t| widget::container::Style {
            background: Some(fill_color.into()),
            border: cosmic::iced::Border { radius: (height/2.0).into(), ..Default::default() },
            ..Default::default()
        }),
    )
    .width(cosmic::iced::Length::Fixed(full))
    .style(|_t| widget::container::Style {
        background: Some(Color::from_rgba(1.0,1.0,1.0,0.15).into()),
        border: cosmic::iced::Border { radius: (height/2.0).into(), ..Default::default() },
        ..Default::default()
    });

    if cfg.show_percent {
        widget::row().spacing(4).push(track).push(widget::text(g.label.clone()).size(12)).into()
    } else {
        track.into()
    }
}
```

Then in the per-gauge loop choose `bar` vs `dot` by style:

```rust
for g in gauges {
    let el = match cfg.style {
        crate::config::Style::ColorDot => dot(g, cfg, dim),
        _ => bar(g, cfg, dim),
    };
    row = row.push(el);
}
```

- [ ] **Step 6: Build and eyeball each style**

Run: `nix develop -c just install-dev`, restart panel. Temporarily set `style` in `Config::default()` to each variant and confirm dot / fill-bar / fill-color all render correctly, then revert to `ColorDot`.

- [ ] **Step 7: Commit**

```bash
git add src/fill.rs src/view.rs src/main.rs
git commit -m "feat: fill-bar and fill-color indicator styles"
```

---

### Task 8: Hover tooltip + click popup with reset countdowns

**Deliverable:** Hover shows `Session X% · Weekly Y%`; click opens a COSMIC popup with each budget's percent, a small bar, and `resets in …` countdowns.

**Files:**
- Modify: `src/main.rs` (popup state + `view_window`), `src/view.rs` (popup content builder)

**Interfaces:**
- Consumes: `usage::{format_countdown}`, `indicator::gauges`
- Produces: `fn popup_view<'a>(sample: &UsageSample, now: i64, cfg: &Config) -> Element<'a, Message>`; `fn tooltip_text(sample: &UsageSample, now: i64) -> String`; `fn reset_label(sample: &UsageSample, now: i64) -> String`

- [ ] **Step 1: Add a tooltip-text test**

```rust
// in src/view.rs tests
#[cfg(test)]
mod ttests {
    use super::*;
    use crate::usage::UsageSample;
    #[test]
    fn tooltip_renders_both_with_resets() {
        let now = 2000;
        let s = UsageSample {
            session: 0.38, weekly: 0.12,
            session_reset: now + 60 * 60 * 2 + 60 * 14,      // 2h 14m
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
            session: 0.0, weekly: 0.0,
            session_reset: now + 60 * 45,                     // 45m (soonest)
            weekly_reset: now + 60 * 60 * 24 * 4,             // 4d
            ts: 0,
        };
        assert_eq!(reset_label(&s, now), "resets in 45m");
    }
}
```

- [ ] **Step 2: Implement `tooltip_text` and `popup_view` in `src/view.rs`**

```rust
use crate::usage::{format_countdown, UsageSample};

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
/// optional bar text when `show_reset` is enabled.
pub fn reset_label(sample: &UsageSample, now: i64) -> String {
    let soonest = sample.session_reset.min(sample.weekly_reset) - now;
    format!("resets in {}", format_countdown(soonest))
}

fn budget_row<'a>(name: &str, value: f32, reset: i64, now: i64) -> Element<'a, Message> {
    let pct = (value * 100.0).round() as i64;
    let countdown = format_countdown(reset - now);
    widget::column()
        .spacing(2)
        .push(widget::text(format!("{name}: {pct}%")).size(14))
        .push(widget::text(format!("resets in {countdown}")).size(11))
        .into()
}

pub fn popup_view<'a>(sample: &UsageSample, now: i64, _cfg: &Config) -> Element<'a, Message> {
    widget::column()
        .spacing(12)
        .padding(12)
        .push(budget_row("Session (5h)", sample.session, sample.session_reset, now))
        .push(budget_row("Weekly (7d)", sample.weekly, sample.weekly_reset, now))
        .into()
}
```

- [ ] **Step 3: Add popup + tooltip wiring in `src/main.rs`**

Add popup id state and the multi-window hooks. Follow the template's popup example exactly for `get_popup`/`destroy_popup` and `view_window`:

```rust
use cosmic::iced::window::Id;
use cosmic::iced::Limits;

// add to struct Window:
//   popup: Option<Id>,
// init: set popup: None

// extend Message with: PopupClosed(Id)

fn update(&mut self, message: Message) -> cosmic::app::Task<Message> {
    match message {
        Message::Reload => {
            self.sample = usage::read_latest(&self.config.history_path_resolved());
            self.now = unix_now();
        }
        Message::TogglePopup => {
            return if let Some(id) = self.popup.take() {
                cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(id)
            } else {
                let new_id = Id::unique();
                self.popup = Some(new_id);
                let mut popup_settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(), new_id, None, None, None,
                );
                popup_settings.positioner.size_limits = Limits::NONE
                    .max_width(280.0).min_width(200.0).max_height(200.0).min_height(80.0);
                cosmic::iced::platform_specific::shell::commands::popup::get_popup(popup_settings)
            };
        }
        Message::PopupClosed(id) => {
            if self.popup == Some(id) { self.popup = None; }
        }
    }
    cosmic::app::Task::none()
}

fn view_window(&self, _id: Id) -> Element<Message> {
    match &self.sample {
        Some(s) => self.core.applet.popup_container(
            view::popup_view(s, self.now, &self.config)
        ).into(),
        None => self.core.applet.popup_container(
            cosmic::widget::text("No Claude usage data yet").size(14)
        ).into(),
    }
}
```

Wrap the panel `view()` element in a tooltip:

```rust
fn view(&self) -> Element<Message> {
    let state = indicator_state(self.sample.as_ref(), self.now, &self.config);
    let inner = view::indicator_view(&state, &self.config);
    let button = self.core.applet.applet_button(inner, Message::TogglePopup);
    match &self.sample {
        Some(s) => {
            let tip: Element<Message> = cosmic::widget::tooltip(
                button,
                cosmic::widget::text(view::tooltip_text(s, self.now)),
                cosmic::widget::tooltip::Position::Bottom,
            ).into();
            if self.config.show_reset {
                // Append the soonest reset countdown beside the indicator.
                cosmic::widget::row()
                    .spacing(4)
                    .push(tip)
                    .push(cosmic::widget::text(view::reset_label(s, self.now)).size(12))
                    .into()
            } else {
                tip
            }
        }
        None => button,
    }
}
```

Exact symbol paths (`popup::get_popup`, `popup_container`, `tooltip`) must be reconciled with the pinned libcosmic rev via the template's applet example. The template's `cosmic-applet-*` in `pop-os/cosmic-applets` (e.g. the battery applet) is the closest working reference for popup wiring.

- [ ] **Step 4: Run tooltip unit test**

Run: `nix develop -c cargo test view::ttests`
Expected: PASS.

- [ ] **Step 5: Build and verify hover + click**

Run: `nix develop -c just install-dev`, restart panel. Hover → tooltip appears; click → popup shows both budgets with `resets in …`. Click again closes it.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs src/view.rs
git commit -m "feat: hover tooltip and click popup with reset countdowns"
```

---

### Task 9: Persisted config via cosmic-config + Nix packaging + README

**Deliverable:** Config loads from cosmic-config (so the user can change `scope`/`style`/etc. without recompiling), with defaults on first run. A Nix derivation packages the applet into home-manager so it installs permanently via `nixos-rebuild`. README documents install and config. Full suite green.

**Files:**
- Modify: `src/config.rs` (cosmic-config load/save), `src/main.rs` (load at init)
- Create (in dotfiles): `~/dotfiles/overlays/claude-usage/default.nix`
- Modify (in dotfiles): `~/dotfiles/flake.nix` (register overlay), `~/dotfiles/users/tux/packages/dev.nix` (add package)
- Modify: `README.md`

**Interfaces:**
- Consumes: `Config`
- Produces: `fn load() -> Config` (returns defaults if unset/missing)

- [ ] **Step 1: Add cosmic-config plumbing to `src/config.rs`**

```rust
use cosmic::cosmic_config::{self, CosmicConfigEntry};

pub const CONFIG_ID: &str = "co.osterberg.ClaudeUsage";
pub const CONFIG_VERSION: u64 = 1;

impl Config {
    pub fn load() -> Config {
        match cosmic_config::Config::new(CONFIG_ID, CONFIG_VERSION) {
            Ok(handler) => Config::get_entry(&handler).unwrap_or_else(|(_errs, cfg)| cfg),
            Err(_) => Config::default(),
        }
    }
}
```

Derive `CosmicConfigEntry` on `Config` (and ensure each field type implements the required traits). If the derive macro requires `#[version = 1]` or a different attribute in the pinned rev, follow the template's config example. If cosmic-config integration proves heavy, the acceptable fallback is reading a RON file at `~/.config/cosmic/co.osterberg.ClaudeUsage/config.ron` manually with `ron` + `serde` — Config already derives `Serialize`/`Deserialize`.

- [ ] **Step 2: Load config at startup in `src/main.rs`**

Change `init` to use `Config::load()` instead of `Config::default()`:

```rust
let config = Config::load();
```

- [ ] **Step 3: Write `README.md`**

```markdown
# cosmic-applet-claude-usage

A minimal COSMIC panel applet showing Claude Code usage as a quiet color-coded
indicator. Green → amber → red as you approach your session (5h) or weekly (7d)
limit. Hover for exact percentages; click for reset countdowns.

## Data source

Reads the last line of `~/.claude/usage-history.jsonl`, which Claude Code's
status line appends to while running. Values are "last known" between sessions;
the indicator dims when data is older than `stale_after`.

## Build & install (NixOS)

Dev iteration (no sudo, no /usr):

    nix develop -c just install-dev   # → ~/.local/bin + ~/.local/share/applications

Permanent install via home-manager (see `~/dotfiles/overlays/claude-usage`):

    sudo nixos-rebuild switch --flake ~/dotfiles#eclipse

Then add `co.osterberg.ClaudeUsage` to the panel via COSMIC Settings → Panel → Applets.

## Config

Stored via cosmic-config (`co.osterberg.ClaudeUsage` v1). Keys:

| Key            | Values                                   | Default     |
|----------------|------------------------------------------|-------------|
| scope          | session, weekly, worst, both             | worst       |
| style          | color-dot, fill-bar, fill-color          | color-dot   |
| show_percent   | true, false                              | false       |
| show_reset     | true, false (soonest reset text on bar)  | false       |
| thresholds     | { amber, red } fractions                 | 0.50 / 0.80 |
| stale_after    | seconds                                  | 600         |
| history_path   | path override (optional)                 | (unset)     |
```

- [ ] **Step 4: Run the full test suite**

Run: `nix develop -c cargo test`
Expected: PASS (all unit tests across config/usage/indicator/fill/view).

- [ ] **Step 5: Commit the applet source (before touching dotfiles)**

Commit a `Cargo.lock` (required for reproducible Nix builds):

```bash
nix develop -c cargo build --release   # ensures Cargo.lock is current
git add src/config.rs src/main.rs README.md Cargo.lock
git commit -m "feat: persisted cosmic-config + README; v0.1 feature complete"
```

- [ ] **Step 6: Write the Nix derivation in dotfiles — `~/dotfiles/overlays/claude-usage/default.nix`**

Follows the existing overlay pattern (`final: prev:`). Builds the local checkout with `rustPlatform.buildRustPackage` and installs the desktop entry, rewriting `Exec` to the store path.

```nix
final: prev:

{
  cosmic-applet-claude-usage = final.rustPlatform.buildRustPackage rec {
    pname = "cosmic-applet-claude-usage";
    version = "0.1.0";

    src = /home/tux/code/cosmic-applet-claude-usage;
    cargoLock = {
      lockFile = /home/tux/code/cosmic-applet-claude-usage/Cargo.lock;
      # libcosmic is a git dependency; pin its hash. Run the build once and
      # copy the "got:" hash nix prints into outputHashes below.
      outputHashes = {
        # "libcosmic-0.1.0" = "sha256-AAAA...";
      };
    };

    nativeBuildInputs = with final; [ pkg-config makeWrapper ];
    buildInputs = with final; [ wayland libxkbcommon vulkan-loader libGL fontconfig freetype expat ];

    postInstall = ''
      install -Dm0644 data/co.osterberg.ClaudeUsage.desktop \
        $out/share/applications/co.osterberg.ClaudeUsage.desktop
      substituteInPlace $out/share/applications/co.osterberg.ClaudeUsage.desktop \
        --replace-warn 'Exec=cosmic-applet-claude-usage' "Exec=$out/bin/cosmic-applet-claude-usage"
    '';

    postFixup = ''
      wrapProgram $out/bin/cosmic-applet-claude-usage \
        --prefix LD_LIBRARY_PATH : ${final.lib.makeLibraryPath buildInputs}
    '';

    meta = with final.lib; {
      description = "Minimal COSMIC panel applet showing Claude Code usage";
      mainProgram = "cosmic-applet-claude-usage";
      platforms = platforms.linux;
    };
  };
}
```

- [ ] **Step 7: Register the overlay and package in dotfiles**

In `~/dotfiles/flake.nix`, add to the `nixpkgs.overlays` list (alongside the existing `(import ./overlays/...)` lines):

```nix
            (import ./overlays/claude-usage)
```

In `~/dotfiles/users/tux/packages/dev.nix`, add to `home.packages`:

```nix
    cosmic-applet-claude-usage
```

- [ ] **Step 8: Build the package, fill in the libcosmic hash, rebuild**

First a dry build to learn the git-dependency hash:

```bash
nix build ~/dotfiles#nixosConfigurations.eclipse.config.home-manager.users.tux.home.packages 2>&1 | tee /tmp/build.log || true
```
If it fails with a hash mismatch for the libcosmic git dep, copy the `got: sha256-…` value into `outputHashes` in `overlays/claude-usage/default.nix` (key = `"libcosmic-<version>"` as nix reports it). Then apply:

```bash
sudo nixos-rebuild switch --flake ~/dotfiles#eclipse
```
Expected: `cosmic-applet-claude-usage` on PATH; desktop entry present in the system profile.

- [ ] **Step 9: Final manual smoke of all features**

Restart the panel and add the applet (COSMIC Settings → Panel → Applets). Confirm: indicator color matches usage; hover tooltip; click popup with countdowns; editing the config (e.g. `scope = both` via cosmic-config) and restarting the applet reflects the change; missing-file shows hollow grey + "No data" popup.

- [ ] **Step 10: Commit the dotfiles changes**

```bash
cd ~/dotfiles
git add overlays/claude-usage flake.nix users/tux/packages/dev.nix
git commit -m "feat: package cosmic-applet-claude-usage for tux"
```

---

## Notes for the implementer

- **libcosmic API drift is the main risk.** This plan's libcosmic code follows the `pop-os/cosmic-applet-template` and `pop-os/cosmic-applets` patterns, but exact symbol names (popup commands, `applet_button`, container `Style`) vary by rev. When in doubt, the cloned template and the real battery/network applets are ground truth — match them and keep the pinned `rev`.
- **Pure logic is fully tested** (config, usage, indicator, fill, tooltip text). Visual/panel behavior is verified by eye after each rendering task. Do not skip the manual smoke steps — they are the only test for the libcosmic wiring.
- **Staleness/countdowns use wall-clock `now`**; refreshed by the 30s tick, so popups stay current without reopening.
```
