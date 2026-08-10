cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/fea_beam_browser.wasm .
basic-http-server .