cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir export/web --out-name opengl ../target/wasm32-unknown-unknown/release/opengl.wasm --no-typescript
set RUST_LOG=debug
basic-http-server -x export/web