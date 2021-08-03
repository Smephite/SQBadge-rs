use crate::js::fetch;
use crate::stellar::stellar_data;
use serde_json::Value;
use toml::Value as tomlValue;
use wasm_bindgen::JsValue;
static HORIZONT_ENDPOINT: &str = "https://horizon.stellar.org/";

pub async fn fetch_account(id: &String) -> Result<stellar_data::Account, JsValue> {
    let mut url = String::from(HORIZONT_ENDPOINT);
    url.push_str("accounts/");
    url.push_str(&id);
    let json = fetch::get_json(&url).await?;
    let acc: stellar_data::Account = json.into_serde().unwrap();
    Ok(acc)
}

pub async fn fetch_ledger_payments(
    id: &String,
) -> Result<Vec<stellar_data::OperationPayment>, JsValue> {
    let mut url = String::from(HORIZONT_ENDPOINT);
    url.push_str("ledgers/");
    url.push_str(&id);
    url.push_str("/payments");
    let json = fetch::get_json(&url).await;
    let obj: Value = json?.into_serde().unwrap();
    let records = obj.pointer("/_embedded/records").unwrap().clone();
    let payments: Vec<stellar_data::OperationPayment> = serde_json::from_value(records).unwrap();
    Ok(payments)
}

pub async fn fetch_toml_currencies(toml_url: &String) -> Option<Vec<stellar_data::TOMLCurrency>> {
    let data = match fetch::get_text(toml_url).await {
        Ok(s) => s,
        Err(_) => {
            return None;
        }
    };

    let val: tomlValue = match toml::from_str(&data) {
        Ok(s) => s,
        Err(_) => {
            return None;
        }
    };

    let currencies = val.get("CURRENCIES")?.clone();
    let currencies: Result<Vec<stellar_data::TOMLCurrency>, toml::de::Error> =
        currencies.try_into();

    match currencies {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}
