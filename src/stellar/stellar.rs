use crate::js::fetch;
use crate::stellar::stellar_data;
use crate::util::error::{Error, StellarErr};
use log::debug;
use serde_json::Value;
use toml::Value as tomlValue;
use wasm_bindgen::JsValue;
use web_sys::console::debug;
static HORIZONT_ENDPOINT: &str = "https://horizon.stellar.org/";

type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub async fn fetch_account(id: &String) -> Result<stellar_data::Account> {
    let mut url = String::from(HORIZONT_ENDPOINT);
    url.push_str("accounts/");
    url.push_str(&id);
    let json = fetch::get_json(&url).await;

    if json.is_err() {
        return Err(Error::Other(json.err().unwrap().as_string().unwrap()));
    }

    let acc: stellar_data::Account = json.unwrap().into_serde().unwrap();
    Ok(acc)
}

pub async fn fetch_account_payments(id: &String) -> Result<Vec<stellar_data::OperationPayment>> {
    let mut url = String::from(HORIZONT_ENDPOINT);
    url.push_str("accounts/");
    url.push_str(&id);
    url.push_str("/payments?limit=200");

    let mut next_url = url.clone();
    let mut all_payments = vec![];
    loop {
        let json = fetch::get_json(&next_url).await;
        if json.is_err() {
            break;
        }

        let data: Value = json.ok().unwrap().into_serde().unwrap();
        let next = data.pointer("/_links/next/href");

        if next.is_none() {
            let status_code = data.pointer("/status");
            if status_code.is_none() {
                return Err(Error::StellarErr(StellarErr::Unknown));
            }
            let status_code = status_code.unwrap();
            if status_code == 400 {
                // invalid public key?
                return Err(Error::StellarErr(StellarErr::InvalidPublicKey));
            } else if status_code == 404 {
                // account not funded
                return Err(Error::StellarErr(StellarErr::AccountNotFound));
            }
            return Err(Error::StellarErr(StellarErr::Unknown));
        }

        next_url = urldecode::decode(String::from(next.unwrap().as_str().unwrap()));
        let records = data.pointer("/_embedded/records").unwrap().clone();
        let mut payment_data: Vec<stellar_data::OperationPayment> =
            serde_json::from_value(records).unwrap();
        if payment_data.len() == 0 {
            break;
        }
        all_payments.append(&mut payment_data);
    }
    Ok(all_payments)
}

pub async fn search_created_claimed_balances(
    issuer: &String,
    asset: &String,
    needle_account: &String,
) -> Option<stellar_data::OperationClaimableBalance> {
    debug!("Searching for asset {} in claimable balances", asset);
    let mut url = String::from(HORIZONT_ENDPOINT);
    url.push_str("accounts/");
    url.push_str(&issuer);
    url.push_str("/operations?limit=200&order=desc");

    let mut next_url = url.clone();
    loop {
        let json = fetch::get_json(&next_url).await;
        if json.is_err() {
            break;
        }

        let data: Value = json.ok().unwrap().into_serde().unwrap();
        let next = data.pointer("/_links/next/href");

        if next.is_none() {
            return None;
        }

        next_url = urldecode::decode(String::from(next.unwrap().as_str().unwrap()));
        let records = data.pointer("/_embedded/records").unwrap().clone();
        let operation_data: Vec<stellar_data::OperationClaimableBalance> =
            serde_json::from_value(records).unwrap();
        if operation_data.len() == 0 {
            return None;
        }

        for operation in operation_data {
            let op_clone = operation.clone();
            if operation.type_i != 14 {
                // is not claimable balance
                continue;
            }

            if &operation.asset != asset {
                continue;
            }

            let claimants = operation.claimants;

            for c in claimants {
                if c.is_object() {
                    let c = c.as_object().unwrap();
                    if c.contains_key("destination")
                        && c.get("destination").unwrap().as_str().unwrap()
                            == needle_account.as_str()
                    {
                        return Some(op_clone);
                    }
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
pub async fn fetch_ledger_payments(
    id: &String,
) -> std::result::Result<Vec<stellar_data::OperationPayment>, JsValue> {
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
    let currencies: std::result::Result<Vec<stellar_data::TOMLCurrency>, toml::de::Error> =
        currencies.try_into();

    match currencies {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}
