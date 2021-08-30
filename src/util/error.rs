#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    StellarErr(StellarErr),
    Other(String),
    ProofErr(ProofErr),
    Unknown,
}

#[derive(Debug)]
pub enum ProofErr {
    ProofInvalidEncoding,
    ProofWrongVersion,
}

#[derive(Debug)]
pub enum StellarErr {
    InvalidPublicKey,
    AccountNotFound,
    Unknown,
}
