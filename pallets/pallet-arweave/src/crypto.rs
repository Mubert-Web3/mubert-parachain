use polkadot_sdk::*;

use sp_core::crypto::KeyTypeId;

pub const KEY_ARWEAVE_SIGNER: KeyTypeId = KeyTypeId(*b"ar_s");

pub mod arweave {
    use super::KEY_ARWEAVE_SIGNER;

    use polkadot_sdk::*;

    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };
    app_crypto!(sr25519, KEY_ARWEAVE_SIGNER);

    pub struct AuthId;

    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for AuthId {
        type RuntimeAppPublic = Public;
        type GenericPublic = sp_core::sr25519::Public;
        type GenericSignature = sp_core::sr25519::Signature;
    }

    // implemented for mock runtime in test
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
