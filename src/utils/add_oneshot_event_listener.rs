use web_sys::{js_sys::Function, AddEventListenerOptions, EventTarget};

use crate::utils::log_error;

thread_local! {
    static OPTIONS: AddEventListenerOptions = {
        let options = AddEventListenerOptions::new();
        options.set_once(true);
        options
    };
}

/// Takes an already-leaked `Function` rather than a `&Closure`.
///
/// Callers must ensure the underlying Rust closure outlives the listener -
/// in practice by calling `Closure::forget()` *before* registering, not
/// afterwards. Registering first and forgetting later is unsafe whenever the
/// code in between can be cancelled: dropping the Closure while a listener is
/// still attached produces "closure invoked recursively or after being
/// dropped" when the event eventually fires.
pub fn add_oneshot_event_listener(
    target: &EventTarget,
    type_: &str,
    callback: &Function,
) {
    OPTIONS.with(|options| {
        if target
            .add_event_listener_with_callback_and_add_event_listener_options(
                type_, callback, options,
            )
            .is_err()
        {
            log_error!("Failed to add {type_} event listener");
        }
    });
}
