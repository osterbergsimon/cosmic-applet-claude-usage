// SPDX-License-Identifier: GPL-3.0-only

mod config;
mod indicator;
mod usage;

use cosmic::iced::Subscription;
use cosmic::prelude::*;

fn main() -> cosmic::iced::Result {
    // Start the applet's event loop with `()` as the application's flags.
    cosmic::applet::run::<Window>(())
}

/// The applet model. The COSMIC runtime manages the `core`.
struct Window {
    core: cosmic::Core,
}

/// Messages emitted by the applet. None yet — the static placeholder is inert.
#[derive(Debug, Clone)]
enum Message {}

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
        (Window { core }, Task::none())
    }

    fn update(&mut self, _message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// The applet's button in the panel is drawn here. Static placeholder for now.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("display-symbolic")
            .into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
