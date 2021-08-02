use wasm_bindgen::prelude::*;
//use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/src/albedo.js")]
extern "C" {
    #[wasm_bindgen(js_name = "publicKey", catch)]
    pub async fn public_key(token: JsValue) -> Result<JsValue, JsValue>;
}