#![cfg_attr(not(feature = "std"), no_std)]

use polkadot_sdk::*;

use scale_codec::Codec;

extern crate alloc;
use alloc::vec::Vec;

sp_api::decl_runtime_apis! {
    pub trait ApiIpOnchainRuntime<EntityId, AuthorId, AuthorityId, EntityDetails, AuthorDetails, AuthorityDetails>
    where
        EntityId: Codec,
        AuthorId: Codec,
        AuthorityId: Codec,
        EntityDetails: Codec,
        AuthorDetails: Codec,
        AuthorityDetails: Codec,
    {
        fn entity(entity_id: EntityId) -> Result<EntityDetails, sp_runtime::DispatchError>;
        fn entities(from: EntityId, to: EntityId) -> Result<Vec<(EntityId, EntityDetails)>, sp_runtime::DispatchError>;

        fn author(author_id: AuthorId) -> Result<AuthorDetails, sp_runtime::DispatchError>;
        fn authors(from: AuthorId, to: AuthorId) -> Result<Vec<(AuthorId, AuthorDetails)>, sp_runtime::DispatchError>;

        fn authority(authority_id: AuthorityId) -> Result<AuthorityDetails, sp_runtime::DispatchError>;
        fn authorities(from: AuthorityId, to: AuthorityId) -> Result<Vec<(AuthorityId, AuthorityDetails)>, sp_runtime::DispatchError>;
    }
}
