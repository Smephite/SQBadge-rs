#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Error {
    StellarErr(StellarErr),
    Other(String),
    ProofErr(ProofErr),
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProofErr {
    ProofInvalidEncoding,
    ProofWrongVersion,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StellarErr {
    InvalidPublicKey,
    AccountNotFound,
    Unknown,
}
