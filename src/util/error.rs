#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    StellarErr(StellarErr),
    Other(String),
    Unknown,
}

#[derive(Debug)]
pub enum StellarErr {
    InvalidPublicKey,
    AccountNotFound,
    Unknown,
}
