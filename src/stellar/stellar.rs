use crate::js::fetch;
use crate::stellar::stellar_data;
use wasm_bindgen::JsValue;
use serde_json::Value;
static HORIZONT_ENDPOINT : &str = "https://horizon.stellar.org/";

pub async fn fetch_account(id: String) -> Result<stellar_data::Account, JsValue>
{
    let mut url = String::from(HORIZONT_ENDPOINT);
    url.push_str("accounts/");
    url.push_str(&id);
    let json = fetch::get_json(url).await;
    let acc : stellar_data::Account = json?.into_serde().unwrap();
    Ok(acc)
}

pub async fn fetch_ledger_payments(id: String) -> Result<Vec<stellar_data::OperationPayment>, JsValue>
{
    let mut url = String::from(HORIZONT_ENDPOINT);
    url.push_str("ledgers/");
    url.push_str(&id);
    url.push_str("/payments");
    let json = fetch::get_json(url).await;
    let obj : Value = json?.into_serde().unwrap();
    let records = obj.pointer("/_embedded/records").unwrap().clone();
    let payments : Vec<stellar_data::OperationPayment> = serde_json::from_value(records).unwrap();
    Ok(payments)
}