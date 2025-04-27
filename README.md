to compile for web like cloudflare servers wasm-pack build --target web

to compile for normal javascript use case wasm-pack build --target nodejs

use "npx wrangler dev"

wasm-pack test --firefox 

cargo add wasm-bindgen

cargo install wasm-pack

cargo add web-sys

    use web_sys::console;
    console::log_1(&format!("{:?}", pairs).into());

