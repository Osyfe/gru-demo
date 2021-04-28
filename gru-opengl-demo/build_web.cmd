cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir export/web --out-name opengl ../target/wasm32-unknown-unknown/release/gru_opengl_demo.wasm --no-typescript
set RUST_LOG=debug
basic-http-server -x export/web