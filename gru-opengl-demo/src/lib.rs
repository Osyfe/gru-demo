mod prog;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() { gru_opengl::start::<prog::Demo>(()); }
