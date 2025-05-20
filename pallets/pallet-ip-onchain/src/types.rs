use super::*;

use polkadot_sdk::sp_debug_derive;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// Authority
pub type AuthorityDetailsFor<T, I = ()> = AuthorityDetails<
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::MaxShortStringLength,
>;
#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AuthorityKind {
    Musician,
    Label,
}

#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(ShortStringLimit))]
pub struct AuthorityDetails<AccountId, ShortStringLimit: Get<u32>> {
    pub authority_kind: AuthorityKind,
    pub owner: AccountId,
    pub name: BoundedVec<u8, ShortStringLimit>,
}

/// Author
pub type AuthorFor<T, I = ()> = AuthorDetails<
    <T as Config<I>>::AuthorityId,
    <T as Config<I>>::MaxShortStringLength,
    <T as Config<I>>::MaxLongStringLength,
>;

#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(ShortStringLimit, LongStringLimit))]
pub struct AuthorDetails<AuthorityId, ShortStringLimit: Get<u32>, LongStringLimit: Get<u32>> {
    pub nickname: BoundedVec<u8, ShortStringLimit>,
    pub real_name: Option<BoundedVec<u8, LongStringLimit>>,
    pub owner: AuthorityId,
}

/// Entity
pub type EntityDetailsFor<T, I = ()> = EntityDetails<
    <T as Config<I>>::AuthorityId,
    <T as Config<I>>::AuthorId,
    <T as Config<I>>::EntityId,
    Wallet<<T as frame_system::Config>::AccountId>,
    Metadata<<T as Config<I>>::MaxLongStringLength>,
    <T as Config<I>>::MaxEntityAuthors,
    <T as Config<I>>::MaxRoyaltyParts,
    <T as Config<I>>::MaxRelatedEntities,
>;

#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(MaxEntityAuthors, MaxRoyaltyParts, MaxRelatedEntities))]
pub struct EntityDetails<
    AuthorityId,
    AuthorId,
    EntityId,
    Wallet,
    Metadata,
    MaxEntityAuthors: Get<u32>,
    MaxRoyaltyParts: Get<u32>,
    MaxRelatedEntities: Get<u32>,
> {
    pub entity_kind: IPEntityKind,
    pub owner: AuthorityId,

    pub authors: Option<BoundedVec<AuthorId, MaxEntityAuthors>>,
    pub royalty_parts: Option<BoundedVec<Wallet, MaxRoyaltyParts>>,
    pub related_to: Option<BoundedVec<EntityId, MaxRelatedEntities>>,

    pub metadata: Option<Metadata>,
}

#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum IPEntityKind {
    Sample,
    Track,
    GenerativeTrack,
    GenerativeSample,
}

#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MetadataStandard {
    MM25,
}

/// Metadata
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(LongStringLimit))]
pub struct Metadata<LongStringLimit: Get<u32>> {
    pub url: BoundedVec<u8, LongStringLimit>,
    pub standard: MetadataStandard,
}

/// Wallet
#[derive(
    Clone, Encode, Decode, Eq, PartialEq, sp_debug_derive::RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Wallet<AccountId> {
    // pub name: Option<BoundedVec<u8, ShortStringLimit>>,
    pub address_id: AccountId,
    pub weight: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use polkadot_sdk::sp_debug_derive;

    use serde_json;

    // Mock implementation for the `Get` trait to define the maximum string length
    #[derive(
        Clone,
        Encode,
        Decode,
        Eq,
        PartialEq,
        sp_debug_derive::RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        Serialize,
        Deserialize,
    )]
    pub struct MaxStringLength;
    impl Get<u32> for MaxStringLength {
        fn get() -> u32 {
            256 // Example maximum length
        }
    }

    #[test]
    fn test_metadata_serialization() {
        // Create a sample Metadata instance
        let url_data = b"https://example.com/metadata".to_vec();
        let bounded_url: BoundedVec<u8, MaxStringLength> =
            BoundedVec::try_from(url_data.clone()).unwrap();
        let metadata = Metadata::<MaxStringLength> { url: bounded_url, standard: MetadataStandard::MM25 };

        // Serialize the Metadata instance to JSON
        let serialized = serde_json::to_string(&metadata).expect("Serialization should succeed");
        println!("Serialized Metadata: {}", serialized.to_string());

        // Deserialize the JSON back into a Metadata instance
        let deserialized: Metadata<MaxStringLength> =
            serde_json::from_str(&serialized).expect("Deserialization should succeed");

        // Ensure the deserialized data matches the original
        assert_eq!(metadata, deserialized);

        // Ensure the URL data is preserved correctly
        assert_eq!(deserialized.url.to_vec(), url_data);
    }
}
