pub mod app;
pub mod components;

use app::App;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run_radio() {
    dioxus::launch(App);
}
