use leptos::task::spawn_local;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    js_sys::Function,
    AnimationPlayState,
    GetAnimationsOptions,
    HtmlElement,
};

use super::{add_oneshot_event_listener, animation_frame};

pub trait OnAnimationsFinishedExt {
    fn on_animations_finished(&self, cb: impl Fn() + 'static, subtree: bool);
}

fn get_animations(element: &HtmlElement, subtree: bool) -> Vec<web_sys::Animation> {
    let animations = if subtree {
        let options = GetAnimationsOptions::new();
        options.set_subtree(true);
        element.get_animations_with_options(&options)
    } else {
        element.get_animations()
    };

    animations
        .into_iter()
        .filter_map(|animation| animation.dyn_into::<web_sys::Animation>().ok())
        // Only animations that can still fire `finish` are worth listening to.
        //
        // `getAnimations()` keeps returning CSS animations that have already
        // completed when they have `animation-fill-mode: forwards` (and
        // `Idle` ones that never started). Those will never fire `finish`
        // again, so a `once: true` listener attached to them never fires and
        // never removes itself.
        //
        // Because this is re-run on every parent mutation, listeners piled up
        // on those long-lived finished animations without bound: an app whose
        // list mutates continuously accumulated them until the main thread was
        // saturated purely by `addEventListener`. Symptom: fine on a fresh
        // load, unusable after the tab had been open a while, with no
        // corresponding growth in DOM size.
        .filter(|animation| {
            !matches!(
                animation.play_state(),
                AnimationPlayState::Finished | AnimationPlayState::Idle
            )
        })
        .collect()
}

impl OnAnimationsFinishedExt for HtmlElement {
    fn on_animations_finished(&self, cb: impl Fn() + 'static, subtree: bool) {
        // Leak the Closure up front and work with the plain JS Function from
        // here on.
        //
        // This used to register the listeners first and call
        // `closure.forget()` at the end of the spawn_local block below. That
        // is only sound if the block always runs to completion - and it
        // doesn't: leptos cancels spawned tasks when the owner is disposed,
        // which happens constantly for rows leaving a list. A cancelled task
        // dropped the Closure while `finish` listeners were still attached to
        // running animations, and firing one then panicked the app with
        // "closure invoked recursively or after being dropped".
        //
        // Forgetting first makes the ordering irrelevant: the Rust closure is
        // never dropped, so a listener firing at any point is safe.
        let closure = Closure::<dyn Fn()>::new(cb);
        let func = closure.as_ref().unchecked_ref::<Function>().clone();
        closure.forget();

        for animation in &get_animations(self, subtree) {
            add_oneshot_event_listener(animation, "finish", &func);
        }

        // sometimes animations appear in the next tick, so let's catch them too
        spawn_local({
            let element = self.clone();
            async move {
                animation_frame().await;

                for animation in get_animations(&element, subtree) {
                    add_oneshot_event_listener(&animation, "finish", &func);
                }
            }
        });
    }
}
