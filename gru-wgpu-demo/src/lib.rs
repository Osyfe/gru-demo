mod root;

#[cfg(target_arch = "wasm32")]
use gru_wgpu::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main()
{
    root::start();
}
