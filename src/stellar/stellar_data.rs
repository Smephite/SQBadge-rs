use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Account {
    account_id: String,
    sequence: String,
    balances: Vec<Balance>
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Balance {
    balance: String,
    asset_type: String,
    asset_code: String,
    asset_issuer: String,
    last_modified_ledger: String
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct OperationPayment {
    id: String,
    source_account: String,
    created_at: String,
    transaction_hash: String,
    asset_type: String,
    asset_code: String,
    asset_issuer: String,
    from: String,
    to: String
}
