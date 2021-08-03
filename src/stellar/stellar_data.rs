use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Account {
    pub account_id: String,
    pub sequence: String,
    pub balances: Vec<Balance>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: String,
    pub asset_issuer: String,
    pub last_modified_ledger: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct OperationPayment {
    pub id: String,
    pub source_account: String,
    pub created_at: String,
    pub transaction_hash: String,
    pub asset_type: String,
    pub asset_code: String,
    pub asset_issuer: String,
    pub from: String,
    pub to: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct TOMLCurrency {
    pub code: String,
    pub issuer: String,
    pub image: String,
    pub tag: String,
}
