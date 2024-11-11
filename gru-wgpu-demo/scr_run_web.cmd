cargo build --target wasm32-unknown-unknown
set RUST_LOG=info
wasm-bindgen --target web --out-dir export --out-name gru_wgpu_demo ../target/wasm32-unknown-unknown/debug/gru_wgpu_demo.wasm --no-typescript
basic-http-server -x export
