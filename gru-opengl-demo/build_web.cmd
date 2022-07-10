cargo build --target wasm32-unknown-unknown --release
set RUST_LOG=info
wasm-bindgen --target web --out-dir export --out-name opengl ../target/wasm32-unknown-unknown/release/gru_opengl_demo.wasm --no-typescript
basic-http-server -x export
