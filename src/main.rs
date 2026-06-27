// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod fill;
mod indicator;
mod usage;
mod view;
mod watch;

use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{time, Subscription};
use cosmic::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

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
    /// The open details popup, if any (set while the popup window is shown).
    popup: Option<Id>,
}

/// Messages emitted by the applet.
#[derive(Debug, Clone)]
pub enum Message {
    /// Re-read the usage history file (wired to a watcher in Task 6).
    Reload,
    /// Toggle the details popup (implemented in Task 8).
    TogglePopup,
    /// The popup window was closed by the compositor (e.g. click-away).
    PopupClosed(Id),
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
                popup: None,
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
            Message::TogglePopup => {
                // Refresh data/clock when opening so the popup is current.
                self.sample = usage::read_latest(&self.config.history_path_resolved());
                self.now = unix_now();
                return if let Some(id) = self.popup.take() {
                    destroy_popup(id)
                } else {
                    let new_id = Id::unique();
                    self.popup = Some(new_id);
                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // inotify watch on the usage file: emits Reload (near-instant) on change.
        // The path doubles as the subscription's identity (run_with hashes it).
        let path = self.config.history_path_resolved();
        let file = Subscription::run_with(path, |p: &PathBuf| {
            watch::file_stream(p.clone())
        });
        // 30s tick: re-reads the file and refreshes `now`, so staleness and
        // countdowns advance even when the file itself is idle.
        let tick = time::every(Duration::from_secs(30)).map(|_| Message::Reload);
        Subscription::batch([file, tick])
    }

    /// The applet's button in the panel renders the real indicator. Hovering
    /// shows a tooltip; clicking toggles the details popup.
    fn view(&self) -> Element<'_, Self::Message> {
        let state: IndicatorState =
            indicator_state(self.sample.as_ref(), self.now, &self.config);
        let inner = view::indicator_view(&state, &self.config);
        // Wrap in a panel-sized press target.
        let button: Element<'_, Self::Message> = self
            .core
            .applet
            .button_from_element(inner, true)
            .on_press(Message::TogglePopup)
            .into();

        match &self.sample {
            Some(s) => {
                let tip: Element<'_, Self::Message> = cosmic::widget::tooltip(
                    button,
                    cosmic::widget::text(view::tooltip_text(s, self.now)),
                    cosmic::widget::tooltip::Position::Bottom,
                )
                .into();
                if self.config.show_reset {
                    // Append the soonest reset countdown beside the indicator.
                    cosmic::widget::Row::new()
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

    /// The popup window: the details column, or a placeholder with no data.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        match &self.sample {
            Some(s) => self
                .core
                .applet
                .popup_container(view::popup_view(s, self.now, &self.config))
                .into(),
            None => self
                .core
                .applet
                .popup_container(cosmic::widget::text("No Claude usage data yet").size(14))
                .into(),
        }
    }

    /// Notified when the compositor closes the popup (e.g. click-away).
    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
