use wasm_bindgen::prelude::*;
//use wasm_bindgen_futures::JsFuture;
use js_sys::JsString;

#[wasm_bindgen(module = "/src/js/albedo.js")]
extern "C" {
    #[wasm_bindgen(js_name = "publicKey", catch)]
    pub async fn public_key(token: JsString) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = "signMessage", catch)]
    pub async fn sign_message(message: JsString) -> Result<JsValue, JsValue>;
}
