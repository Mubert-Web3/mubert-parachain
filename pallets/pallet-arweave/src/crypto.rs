use polkadot_sdk::*;

use sp_core::crypto::KeyTypeId;

/// Key type identifier for the Arweave signer.
pub const KEY_ARWEAVE_SIGNER: KeyTypeId = KeyTypeId(*b"ar_s");

/// Module for Arweave-specific cryptographic utilities.
pub mod arweave {
    use super::KEY_ARWEAVE_SIGNER;

    use polkadot_sdk::*;

    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_ARWEAVE_SIGNER);

    /// Authentication identifier for Arweave operations.
    pub struct AuthId;

    /// Implementation of the `AppCrypto` trait for `AuthId`.
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
        type RuntimeAppPublic = Public;
        type GenericPublic = sp_core::sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
    }

    /// Implementation of the `AppCrypto` trait for mock runtime in tests.
    impl
        frame_system::offchain::AppCrypto<
            <sp_core::sr25519::Signature as Verify>::Signer,
            sp_core::sr25519::Signature,
        > for AuthId
    {
        type RuntimeAppPublic = Public;
        type GenericPublic = sp_core::sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
    }
}
