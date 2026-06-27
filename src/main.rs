// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod indicator;
mod usage;
mod view;

use cosmic::iced::Subscription;
use cosmic::prelude::*;

use config::Config;
use indicator::{indicator_state, IndicatorState};
use usage::UsageSample;

fn main() -> cosmic::iced::Result {
    // Start the applet's event loop with `()` as the application's flags.
    cosmic::applet::run::<Window>(())
}

/// Current unix time in seconds (saturating to 0 on a pre-epoch clock).
fn unix_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// The applet model. The COSMIC runtime manages the `core`.
struct Window {
    core: cosmic::Core,
    config: Config,
    sample: Option<UsageSample>,
    now: i64,
}

/// Messages emitted by the applet.
#[derive(Debug, Clone)]
pub enum Message {
    /// Re-read the usage history file (wired to a watcher in Task 6).
    Reload,
    /// Toggle the details popup (implemented in Task 8).
    TogglePopup,
}

impl cosmic::Application for Window {
    /// The async executor used to run the application's commands.
    type Executor = cosmic::executor::Default;

    /// Data the application receives in its `init` method.
    type Flags = ();

    /// Messages the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "co.osterberg.ClaudeUsage";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // cosmic-config loading is Task 9; use defaults for now.
        let config = Config::default();
        let sample = usage::read_latest(&config.history_path_resolved());
        (
            Window {
                core,
                config,
                sample,
                now: unix_now(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Reload => {
                self.sample = usage::read_latest(&self.config.history_path_resolved());
                self.now = unix_now();
            }
            Message::TogglePopup => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // A file watcher is wired in here in Task 6.
        Subscription::none()
    }

    /// The applet's button in the panel renders the real indicator.
    fn view(&self) -> Element<'_, Self::Message> {
        let state: IndicatorState =
            indicator_state(self.sample.as_ref(), self.now, &self.config);
        let inner = view::indicator_view(&state, &self.config);
        // Wrap in a panel-sized press target.
        self.core
            .applet
            .button_from_element(inner, true)
            .on_press(Message::TogglePopup)
            .into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
