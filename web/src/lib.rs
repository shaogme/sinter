mod app;
mod components;
mod pages;
mod router;

use crate::app::app;
use sinter_ui::dom::view::{IntoAnyView, View};
use sinter_ui::prelude::*;

use lite_alloc::single_threaded::FreeListAllocator;

#[global_allocator]
static ALLOCATOR: FreeListAllocator = FreeListAllocator::new();

#[cfg(debug_assertions)]
use std::panic;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = "export function get_stack() { return new Error().stack; }")]
extern "C" {
    fn get_stack() -> String;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg: &str);
}

/// A panic hook for use with
/// [`std::panic::set_hook`](https://doc.rust-lang.org/nightly/std/panic/fn.set_hook.html)
/// that logs panics into
/// [`console.error`](https://developer.mozilla.org/en-US/docs/Web/API/Console/error).
///
/// On non-wasm targets, prints the panic to `stderr`.
#[allow(dead_code)]
#[cfg(debug_assertions)]
fn hook(info: &panic::PanicHookInfo) {
    #[cfg(target_arch = "wasm32")]
    {
        let msg = format!("{}\n\nStack:\n\n{}\n\n", info, get_stack());
        error(&msg);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::io::{self, Write};
        let _ = writeln!(io::stderr(), "{}", info);
    }
}

#[cfg(debug_assertions)]
fn register() {
    panic::set_hook(Box::new(hook));
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    #[cfg(debug_assertions)]
    register();

    // Create a scope to hold the application state
    create_scope(move || {
        let window = web_sys::window().expect("No global window");
        let document = window.document().expect("No document");
        let body = document.body().expect("No body");

        let app_view = app();
        app_view.into_any().mount(&body);
    });
}
