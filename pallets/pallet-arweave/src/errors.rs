use polkadot_sdk::*;

extern crate alloc;
use alloc::vec::Vec;

/// Type alias for the result of an offchain worker operation.
pub type OffchainWorkerResult<T> = Result<T, OffchainWorkerError>;

/// Enumeration of possible errors that can occur in offchain workers.
///
/// # It ensures
/// - Provides detailed error types for offchain worker operations.
///
/// # Errors
/// - `HttpRequestError`: Represents an error during an HTTP request.
/// - `ArweaveRustError`: Represents an error from the Arweave Rust library.
/// - `ArweaveRustTransactionPending`: Indicates that an Arweave transaction is still pending.
/// - `BoundedVecError`: Represents an error with a bounded vector.
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
