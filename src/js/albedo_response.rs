use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbedoError {
    pub code: i64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbedoPublicKey {
    pub pubkey: String,
    pub signed_message: String,
    pub signature: String,
    pub intent: String,
}
