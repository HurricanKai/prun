use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn main() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    body.set_inner_html("Hello, World!");
}
