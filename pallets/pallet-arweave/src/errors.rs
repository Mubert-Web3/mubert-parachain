use polkadot_sdk::*;

extern crate alloc;
use alloc::vec::Vec;

pub type OffchainWorkerResult<T> = Result<T, OffchainWorkerError>;

#[derive(Debug, thiserror::Error)]
pub enum OffchainWorkerError {
    #[error("Http request: {0:?}")]
    HttpRequestError(sp_runtime::offchain::http::Error),

    #[error("Arweave rust: {0:?}")]
    ArweaveRustError(arweave_rust::ar_substrate::Error),

    #[error("Arweave rust transaction pending")]
    ArweaveRustTransactionPending,

    #[error("Bounded vec: {0:?}")]
    BoundedVecError(Vec<u8>),
}
