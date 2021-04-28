mod app;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn run() { gru_opengl::start::<app::Test>(); }
