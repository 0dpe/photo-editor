// Native desktop entry point. On wasm, `photo_editor::run` is the `#[wasm_bindgen(start)]` entry.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    photo_editor::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {}
