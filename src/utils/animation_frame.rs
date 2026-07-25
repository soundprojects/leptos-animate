use futures::channel;
use leptos::prelude::request_animation_frame;

/// Resolves after two animation frames.
///
/// Both halves have to tolerate the caller going away. Navigating between
/// routes disposes owners, which cancels any `spawn_local` task awaiting this
/// - dropping the receiver while the browser still holds the queued
/// requestAnimationFrame callbacks. Those callbacks then run against a dead
/// channel.
///
/// `send` therefore ignores its error (the receiver being gone just means
/// nobody is waiting any more), and the await tolerates the sender being
/// dropped instead of unwrapping. `sleep()` in this module already got this
/// right; this did not, and panicked on every route change.
pub async fn animation_frame() {
    let (tx, rx) = channel::oneshot::channel();

    request_animation_frame(move || {
        request_animation_frame(move || {
            _ = tx.send(());
        });
    });

    _ = rx.await;
}
