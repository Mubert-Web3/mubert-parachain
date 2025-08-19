#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use polkadot_sdk::pallet_nfts;
use polkadot_sdk::pallet_nfts::ItemConfig;
use polkadot_sdk::polkadot_sdk_frame as frame;

use frame::traits::Contains;

use frame::prelude::*;

use frame::traits::{
    tokens::nonfungibles_v2::{Create, Mutate},
    Currency, Get, Incrementable,
};

use scale_codec::{Decode, Encode, MaxEncodedLen};

mod features;
mod types;

pub use pallet::*;
pub use types::*;
pub mod external_nfts_macros;
pub mod weights;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub use weights::WeightInfo;

pub const LOG_TARGET: &str = "runtime::ip-onchain";

/// TODO benchmarking for all calls, and remove dev_mode for pallet
#[frame::pallet]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    #[pallet::config]
    pub trait Config<I: 'static = ()>: polkadot_sdk::frame_system::Config {
        type AuthorityId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + CheckedAdd
            + CheckedSub
            + PartialOrd;
        type AuthorId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + CheckedAdd
            + CheckedSub
            + PartialOrd;
        type EntityId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + CheckedAdd
            + CheckedSub
            + PartialOrd;

        #[pallet::constant]
        type MaxShortStringLength: Get<u32>;

        #[pallet::constant]
        type MaxLongStringLength: Get<u32>;

        #[pallet::constant]
        type MaxEntityAuthors: Get<u32>;

        #[pallet::constant]
        type MaxRoyaltyParts: Get<u32>;

        #[pallet::constant]
        type MaxRelatedEntities: Get<u32>;

        #[pallet::constant]
        type MaxArrayLen: Get<u32>;

        type WhiteListChecker: Contains<Self::AccountId>;

        type CollectionId: Member + Parameter + MaxEncodedLen + Copy + Incrementable;
        type ItemId: Member + Parameter + MaxEncodedLen + Copy;

        type CollectionConfig: Default
            + MaxEncodedLen
            + TypeInfo
            + Decode
            + Encode
            + PartialEq
            + Debug
            + Clone;

        type Nfts: Mutate<Self::AccountId, ItemConfig, ItemId = Self::ItemId>
            + Create<Self::AccountId, Self::CollectionConfig, CollectionId = Self::CollectionId>;

        type Currency: Currency<Self::AccountId>;

        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;

        type WeightInfo: weights::WeightInfo;

        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelper<Self::CollectionId, Self::ItemId>;
    }

    #[pallet::storage]
    pub(super) type Authorities<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::AuthorityId, AuthorityDetailsFor<T, I>>;

    #[pallet::storage]
    pub(super) type AuthoritiesAccess<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AuthorityId,
        Blake2_128Concat,
        T::AccountId,
        AuthorityAccessSettings,
        OptionQuery,
    >;

    #[pallet::storage]
    pub(super) type Authors<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        T::AuthorId,
        AuthorDetails<T::AccountId, T::MaxShortStringLength, T::MaxLongStringLength>,
    >;

    #[pallet::storage]
    pub(super) type Entities<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::EntityId, EntityDetailsFor<T, I>>;

    /// Incrementable storages
    ///
    #[pallet::storage]
    pub type NextAuthorityId<T: Config<I>, I: 'static = ()> =
        StorageValue<_, T::AuthorityId, OptionQuery>;

    #[pallet::storage]
    pub type NextAuthorId<T: Config<I>, I: 'static = ()> =
        StorageValue<_, T::AuthorId, OptionQuery>;

    #[pallet::storage]
    pub type NextEntityId<T: Config<I>, I: 'static = ()> =
        StorageValue<_, T::EntityId, OptionQuery>;

    #[pallet::storage]
    pub type NftsSupport<T: Config<I>, I: 'static = ()> = StorageValue<_, bool>;

    /// Events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// Author events
        AuthorAdded {
            author_id: T::AuthorId,
        },
        AuthorEdited {
            author_id: T::AuthorId,
        },

        /// Authority events
        AuthorityAdded {
            authority_id: T::AuthorityId,
        },
        AuthorityEdited {
            authority_id: T::AuthorityId,
        },
        AuthoritiesAccessAdded {
            authority_id: T::AuthorityId,
            account_id: T::AccountId,
        },
        AuthoritiesAccessChanged {
            authority_id: T::AuthorityId,
            account_id: T::AccountId,
        },

        /// Entity events
        EntityAdded {
            entity_id: T::EntityId,
        },
        EntityEdited {
            entity_id: T::EntityId,
        },
    }

    /// Errors
    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Author errors
        AuthorAlreadyExists,
        AuthorNotFound,
        AuthorIdIncrementFailed,

        /// Authority errors
        AuthorityAlreadyExists,
        AuthorityNotFound,
        AuthorityIdIncrementFailed,
        /// Authority nft extension Error
        AuthorityNftCollectionIdAlreadyExist,

        /// AuthoritiesAccess
        AuthoritiesAccessNotFound,
        AuthoritiesAccessExist,
        AuthoritiesAccessNotExist,

        /// Entity errors
        EntityAlreadyExists,
        EntityNotFound,
        EntityIdIncrementFailed,
        EntityAuthorNotFound,
        EntityRelatedEntityNotFound,
        EntityNftOwnerMustBeSpecified,
        EntityNftImmutable,

        /// General Errors
        Overflow, // checked_add failed
        LimitExceeded,
        BadFormat,

        /// Permissions Errors
        NoPermission,
        NotAuthorized,
        /// Whitelist Errors
        NotWhitelisted,
    }

    /// Calls
    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Authors calls
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_author())]
        pub fn create_author(
            origin: OriginFor<T>,
            nickname: BoundedVec<u8, T::MaxShortStringLength>,
            real_name: Option<BoundedVec<u8, T::MaxLongStringLength>>,
            owner: Option<T::AccountId>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::add_new_author(origin, nickname, real_name, owner)?;
            Ok(())
        }
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::edit_author())]
        pub fn edit_author(
            origin: OriginFor<T>,
            author_id: T::AuthorId,
            real_name: Option<BoundedVec<u8, T::MaxLongStringLength>>,
            new_owner: Option<T::AccountId>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::set_author(origin, author_id, real_name, new_owner)?;
            Ok(())
        }

        /// Authority calls
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::create_authority())]
        pub fn create_authority(
            origin: OriginFor<T>,
            name: BoundedVec<u8, T::MaxShortStringLength>,
            authority_kind: AuthorityKind,
            collection_cfg: Option<T::CollectionConfig>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::add_new_authority(origin, name, authority_kind, collection_cfg)?;
            Ok(())
        }
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::edit_authority())]
        pub fn edit_authority(
            origin: OriginFor<T>,
            authority_id: T::AuthorityId,
            name: Option<BoundedVec<u8, T::MaxShortStringLength>>,
            authority_kind: Option<AuthorityKind>,
            init_collection_id: Option<T::CollectionConfig>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::set_authority(
                origin,
                authority_id,
                name,
                authority_kind,
                init_collection_id,
            )?;
            Ok(())
        }

        /// Entity calls
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::create_entity())]
        pub fn create_entity(
            origin: OriginFor<T>,
            entity_kind: IPEntityKind,
            owner: T::AuthorityId,
            url: BoundedVec<u8, T::MaxLongStringLength>,
            metadata_standard: MetadataStandard,
            metadata_features: MetadataFeatures,
            authors: Option<BoundedVec<T::AuthorId, T::MaxEntityAuthors>>,
            royalty_parts: Option<BoundedVec<Wallet<T::AccountId>, T::MaxRoyaltyParts>>,
            related_entities: Option<BoundedVec<T::EntityId, T::MaxRelatedEntities>>,
            nft_item_id: Option<T::ItemId>,
            nft_owner: Option<T::AccountId>,
            nft_item_config: Option<pallet_nfts::ItemConfig>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::add_new_entity(
                origin,
                entity_kind,
                owner,
                url,
                metadata_standard,
                metadata_features,
                authors,
                royalty_parts,
                related_entities,
                nft_item_id,
                nft_owner,
                nft_item_config,
            )?;
            Ok(())
        }
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::edit_entity())]
        pub fn edit_entity(
            origin: OriginFor<T>,
            entity_id: T::EntityId,
            url: Option<BoundedVec<u8, T::MaxLongStringLength>>,
            metadata_standard: Option<MetadataStandard>,
            metadata_features: Option<MetadataFeatures>,
            owner: Option<T::AuthorityId>,
            authors: Option<BoundedVec<T::AuthorId, T::MaxEntityAuthors>>,
            royalty_parts: Option<BoundedVec<Wallet<T::AccountId>, T::MaxRoyaltyParts>>,
            related_entities: Option<BoundedVec<T::EntityId, T::MaxRelatedEntities>>,
            nft_item_id: Option<T::ItemId>,
            nft_owner: Option<T::AccountId>,
            nft_item_config: Option<pallet_nfts::ItemConfig>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::set_entity(
                origin,
                entity_id,
                url,
                metadata_standard,
                metadata_features,
                owner,
                authors,
                royalty_parts,
                related_entities,
                nft_item_id,
                nft_owner,
                nft_item_config,
            )?;
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::create_account_access())]
        pub fn create_account_access(
            origin: OriginFor<T>,
            authority_id: T::AuthorityId,
            account_id: T::AccountId,
            access: AuthorityAccessSettings,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::add_access(origin, authority_id, account_id, access)?;
            Ok(())
        }

        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::edit_account_access())]
        pub fn edit_account_access(
            origin: OriginFor<T>,
            authority_id: T::AuthorityId,
            account_id: T::AccountId,
            access: AuthorityAccessSettings,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::set_access(origin, authority_id, account_id, access)?;
            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::call_toggle_nfts_support())]
        pub fn call_toggle_nfts_support(origin: OriginFor<T>) -> DispatchResult {
            ensure_root(origin)?;
            Self::toggle_nfts_support()
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    pub trait BenchmarkHelper<CollectionId, ItemId> {
        fn collection_id(i: u32) -> CollectionId;
        fn item_id(i: u32) -> ItemId;
    }
    #[cfg(feature = "runtime-benchmarks")]
    impl<CollectionId, ItemId> BenchmarkHelper<CollectionId, ItemId> for ()
    where
        CollectionId: From<u32>,
        ItemId: From<u32>,
    {
        fn collection_id(i: u32) -> CollectionId {
            i.into()
        }

        fn item_id(i: u32) -> ItemId {
            i.into()
        }
    }
}
