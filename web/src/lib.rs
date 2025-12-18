mod app;
mod components;
mod pages;
mod state;

use app::App;
use leptos::mount::mount_to_body;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    mount_to_body(App);
}
