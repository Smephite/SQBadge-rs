use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbedoError {
    code: i64,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbedoPublicKey {
    pubkey: String,
    signed_message: String,
    signature: String,
    intent: String,
}
