cargo build --target wasm32-unknown-unknown --release
set RUST_LOG=info
wasm-bindgen --target web --out-dir export --out-name gru_wgpu_demo ../target/wasm32-unknown-unknown/release/gru_wgpu_demo.wasm --no-typescript
wasm-opt -O4 --output export/gru_wgpu_demo_bg.wasm export/gru_wgpu_demo_bg.wasm
