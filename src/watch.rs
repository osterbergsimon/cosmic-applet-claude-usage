// SPDX-License-Identifier: GPL-3.0-only

//! inotify-backed file watch that emits [`Message::Reload`] when the usage
//! history file (or its parent directory) changes.

use cosmic::iced::futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use cosmic::iced::stream;
use notify::{Event, RecursiveMode, Watcher};
use std::path::PathBuf;

use crate::Message;

/// Emit [`Message::Reload`] whenever the watched file (or its parent dir)
/// changes.
///
/// We watch the *parent directory* non-recursively rather than the file
/// itself: appenders/editors frequently replace the inode (write-to-temp +
/// rename), which would silently break a watch bound to the original inode.
///
/// The `notify` callback runs on the watcher's own background thread, so we
/// hand events to the async task over a `futures` unbounded channel. This
/// avoids `tokio::task::block_in_place`, which would panic on COSMIC's
/// single-threaded applet executor.
pub fn file_stream(path: PathBuf) -> impl Stream<Item = Message> {
    stream::channel(16, move |mut output: mpsc::Sender<Message>| async move {
        let (tx, mut rx) = mpsc::unbounded::<()>();

        let watch_dir = path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let mut watcher = match notify::recommended_watcher(
            move |res: notify::Result<Event>| {
                if res.is_ok() {
                    // Non-blocking; ignore send errors once the receiver drops.
                    let _ = tx.unbounded_send(());
                }
            },
        ) {
            Ok(w) => w,
            Err(_) => return,
        };

        if watcher
            .watch(&watch_dir, RecursiveMode::NonRecursive)
            .is_err()
        {
            return;
        }

        // Drain change notifications for the lifetime of the subscription.
        while rx.next().await.is_some() {
            let _ = output.send(Message::Reload).await;
        }

        drop(watcher);
    })
}
