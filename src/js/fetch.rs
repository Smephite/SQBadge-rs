use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
pub async fn get(url: String) -> Result<Response, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts).unwrap();

    let window = web_sys::window().unwrap();
    let request_promise = window.fetch_with_request(&request);

    let future = JsFuture::from(request_promise).await?;
    let resp: Response = future.dyn_into().unwrap();
    Ok(resp)
}

pub async fn get_text(url: &String) -> Result<String, JsValue> {
    let resp = get(url.to_string()).await?;
    let text = JsFuture::from(resp.text()?)
        .await
        .unwrap()
        .as_string()
        .unwrap();
    Ok(text)
}

pub async fn get_json(url: &String) -> Result<JsValue, JsValue> {
    let resp = get(url.to_string()).await?;
    let value = JsFuture::from(resp.json()?).await.unwrap();

    Ok(value)
}
