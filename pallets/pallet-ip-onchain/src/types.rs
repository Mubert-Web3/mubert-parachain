extern crate alloc;
use alloc::vec;

use super::*;
use external_nfts_macros::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use enumflags2::{bitflags, BitFlags};

use scale_codec::EncodeLike;
use scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};

use frame::traits::Currency;

pub type BalanceOf<T, I = ()> =
    <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Authority
pub type AuthorityDetailsFor<T, I = ()> =
    AuthorityDetails<<T as Config<I>>::MaxShortStringLength, <T as Config<I>>::CollectionId>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AuthorityKind {
    Musician,
    Label,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(ShortStringLimit))]
pub struct AuthorityDetails<ShortStringLimit: Get<u32>, CollectionId> {
    pub authority_kind: AuthorityKind,
    pub name: BoundedVec<u8, ShortStringLimit>,
    pub collection_id: Option<CollectionId>,
}

/// Author
pub type AuthorFor<T, I = ()> = AuthorDetails<
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::MaxShortStringLength,
    <T as Config<I>>::MaxLongStringLength,
>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[scale_info(skip_type_params(ShortStringLimit, LongStringLimit))]
pub struct AuthorDetails<AccountId, ShortStringLimit: Get<u32>, LongStringLimit: Get<u32>> {
    pub nickname: BoundedVec<u8, ShortStringLimit>,
    pub real_name: Option<BoundedVec<u8, LongStringLimit>>,
    pub owner: AccountId,
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
    <T as Config<I>>::CollectionId,
    <T as Config<I>>::ItemId,
>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
    CollectionId,
    ItemId,
> {
    pub entity_kind: IPEntityKind,
    pub owner: AuthorityId,

    pub authors: Option<BoundedVec<AuthorId, MaxEntityAuthors>>,
    pub royalty_parts: Option<BoundedVec<Wallet, MaxRoyaltyParts>>,
    pub related_to: Option<BoundedVec<EntityId, MaxRelatedEntities>>,

    pub metadata: Metadata,

    pub collection_id: Option<CollectionId>,
    pub item_id: Option<ItemId>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum IPEntityKind {
    Sample,
    Track,
    GenerativeTrack,
    GenerativeSample,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MetadataStandard {
    M25,
}

/// Metadata
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(LongStringLimit))]
pub struct Metadata<LongStringLimit: Get<u32>> {
    pub url: BoundedVec<u8, LongStringLimit>,
    pub standard: MetadataStandard,
    pub features: MetadataFeatures,
}

/// Wallet
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Wallet<AccountId> {
    // pub name: Option<BoundedVec<u8, ShortStringLimit>>,
    pub address_id: AccountId,
    pub weight: u32,
}

/// Flags
///
/// AuthorityAccessSetting - by default no flags
#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AuthorityAccessSetting {
    EditAccess,

    CreateEntity,
    EditEntity,

    EditAuthority,

    CreateAuthorityCollection,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AuthorityAccessSettings(pub BitFlags<AuthorityAccessSetting>);

impl AuthorityAccessSettings {
    pub fn none() -> Self {
        Self(BitFlags::EMPTY)
    }
    pub fn all() -> Self {
        Self(BitFlags::ALL)
    }
    pub fn has_access(&self, f: BitFlags<AuthorityAccessSetting, u64>) -> bool {
        self.0.contains(f)
    }
    pub fn add_access(&mut self, f: AuthorityAccessSetting) {
        self.0.insert(f);
    }
}
impl_codec_bitflags!(AuthorityAccessSettings, u64, AuthorityAccessSetting);

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MetadataFeature {
    Immutable,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MetadataFeatures(pub BitFlags<MetadataFeature>);

impl MetadataFeatures {
    pub fn none() -> Self {
        Self(BitFlags::EMPTY)
    }
    pub fn all() -> Self {
        Self(BitFlags::ALL)
    }
    pub fn has_feature(&self, f: MetadataFeature) -> bool {
        self.0.contains(f)
    }
    pub fn add_feature(&mut self, f: MetadataFeature) {
        self.0.insert(f);
    }
}
impl_codec_bitflags!(MetadataFeatures, u64, MetadataFeature);

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

    // Mock implementation for the `Get` trait to define the maximum string length
    #[derive(
        Clone,
        Encode,
        Decode,
        Eq,
        PartialEq,
        RuntimeDebug,
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
        let metadata = Metadata::<MaxStringLength> {
            url: bounded_url,
            standard: MetadataStandard::M25,
            features: Default::default(),
        };

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

    #[test]
    fn test_access_flags() {
        let all_access = AuthorityAccessSettings::all();

        assert!(all_access.has_access(AuthorityAccessSetting::CreateEntity.into()));
        assert!(all_access.has_access(
            AuthorityAccessSetting::EditAccess
                | AuthorityAccessSetting::CreateEntity
                | AuthorityAccessSetting::EditEntity
                | AuthorityAccessSetting::EditAuthority
        ));

        let none_access = AuthorityAccessSettings::none();

        assert_eq!(
            none_access.has_access(AuthorityAccessSetting::CreateEntity.into()),
            false
        );
        assert_eq!(
            none_access.has_access(
                AuthorityAccessSetting::EditAccess
                    | AuthorityAccessSetting::CreateEntity
                    | AuthorityAccessSetting::EditEntity
                    | AuthorityAccessSetting::EditAuthority
            ),
            false
        );
    }
}
