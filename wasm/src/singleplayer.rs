use web_sys::console;
use wasm_bindgen::prelude::*;

/// Initialize the main lobby state
#[wasm_bindgen]
pub extern fn init() {
    console_error_panic_hook::set_once();
    console::log_1(&"Hello from Rust!".into());
}
