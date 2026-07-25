use std::time::Duration;

use futures::channel;
use leptos::prelude::set_timeout;

pub async fn sleep(duration: Duration) {
    let (tx, rx) = channel::oneshot::channel();

    set_timeout(
        move || {
            _ = tx.send(());
        },
        duration,
    );

    // Same reasoning as animation_frame(): the sender can be dropped without
    // firing if the owner is disposed first, and that is not a panic.
    _ = rx.await;
}
