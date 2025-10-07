#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use polkadot_sdk::pallet_nfts;
use polkadot_sdk::pallet_nfts::ItemConfig;
use polkadot_sdk::polkadot_sdk_frame as frame;
use polkadot_sdk::staging_xcm;

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
pub use xcm::*;
pub mod external_nfts_macros;
pub mod weights;
pub mod xcm;

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
            + PartialOrd
            + Debug
            + Clone;
        type AuthorId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + CheckedAdd
            + CheckedSub
            + PartialOrd
            + Debug
            + Clone;
        type EntityId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + CheckedAdd
            + CheckedSub
            + PartialOrd
            + Debug
            + Clone;

        type ForeignRequestId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + CheckedAdd
            + CheckedSub
            + PartialEq
            + Debug
            + Clone;
        type ForeignAuthorityId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Debug
            + Clone
            + PartialEq;
        type ForeignMover: ForeignMover<
            EntityDetailsFor<Self, I>,
            Self::AuthorityId,
            Self::ForeignAuthorityId,
            ForeignInfoFor<Self, I>,
        >;

        #[pallet::constant]
        type MaxShortStringLength: Get<u32> + Debug + Clone + PartialEq;

        #[pallet::constant]
        type MaxLongStringLength: Get<u32> + Debug + Clone + PartialEq;

        #[pallet::constant]
        type MaxEntityAuthors: Get<u32> + Debug + Clone + PartialEq;

        #[pallet::constant]
        type MaxRoyaltyParts: Get<u32> + Debug + Clone + PartialEq;

        #[pallet::constant]
        type MaxRelatedEntities: Get<u32> + Debug + Clone + PartialEq;

        #[pallet::constant]
        type MaxArrayLen: Get<u32> + Clone + PartialEq;

        type WhiteListChecker: Contains<Self::AccountId>;

        type CollectionId: Member
            + Parameter
            + MaxEncodedLen
            + Copy
            + Incrementable
            + Debug
            + Clone
            + PartialEq;
        type ItemId: Member + Parameter + MaxEncodedLen + Copy + Debug + Clone + PartialEq;

        type CollectionConfig: Default
            + MaxEncodedLen
            + TypeInfo
            + Decode
            + Encode
            + PartialEq
            + Debug
            + Clone
            + Debug;

        type Nfts: Mutate<Self::AccountId, ItemConfig, ItemId = Self::ItemId>
            + Create<Self::AccountId, Self::CollectionConfig, CollectionId = Self::CollectionId>;

        type Currency: Currency<Self::AccountId>;

        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;

        type WeightInfo: weights::WeightInfo;

        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelper<
            Self::CollectionId,
            Self::ItemId,
            Self::ForeignAuthorityId,
        >;
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

    #[pallet::storage]
    pub type NextForeignRequests<T: Config<I>, I: 'static = ()> =
        StorageValue<_, T::ForeignRequestId, OptionQuery>;

    #[pallet::storage]
    pub(super) type ForeignsRequests<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::ForeignRequestId, ForeignInfoFor<T, I>>;

    // source of foreign entities
    #[pallet::storage]
    pub(super) type EntitiesPointer<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, T::EntityId, (staging_xcm::v5::Location, T::EntityId)>;

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
        EntityWraped {
            entity_id: T::EntityId,
        },
        EntityMovedForeign {
            entity_id: T::EntityId,
        },

        /// ForeignRequests events
        ForeignRequestsAdded {
            request_id: T::ForeignRequestId,
        },
        ForeignRequestsDone {
            request_id: T::ForeignRequestId,
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
        AuthorityBadKind,
        /// Authority nft extension Error
        AuthorityNftCollectionIdAlreadyExist,

        /// AuthoritiesAccess
        AuthoritiesAccessNotFound,
        AuthoritiesAccessExist,
        AuthoritiesAccessNotExist,
        AuthoritiesAccessControlledForeign,

        /// Entity errors
        EntityAlreadyExists,
        EntityNotFound,
        EntityIdIncrementFailed,
        EntityAuthorNotFound,
        EntityRelatedEntityNotFound,
        EntityNftOwnerMustBeSpecified,
        EntityNftImmutable,

        /// Foreign errors
        ForeignAlreadyExists,
        ForeignNotFound,
        ForeignIdIncrementFailed,
        ForeignRequestNoPermission,
        ForeignRequestAlreadyDone,
        ForeignRequestNotApproved,

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

        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::foreign_authority_request())]
        pub fn foreign_authority_request(
            origin: OriginFor<T>,
            foreign_authority_id: T::ForeignAuthorityId,
            name: BoundedVec<u8, T::MaxShortStringLength>,
            entity_id: T::EntityId,
            foreign_location: staging_xcm::v5::Location,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let here_authority_id =
                Self::add_new_authority(who.clone(), name, AuthorityKind::Foreign, None)?;
            Self::set_foreign_authority_request(
                who,
                entity_id,
                here_authority_id,
                foreign_authority_id,
                foreign_location,
            )?;
            Ok(())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::foreign_authority_request_approve())]
        pub fn foreign_authority_request_approve(
            origin: OriginFor<T>,
            entity_id: T::EntityId,
            foreign_request_id: T::ForeignRequestId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::foreign_request_approve(who, entity_id, foreign_request_id)
        }

        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::foreign_authority_request_take())]
        pub fn foreign_authority_request_take(
            origin: OriginFor<T>,
            foreign_request_id: T::ForeignRequestId,
            src: staging_xcm::v5::Location,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?; // foreign origin is expected
            Self::foreign_request_take(who, foreign_request_id, src)
        }

        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::access_wrapped_entity())]
        pub fn access_wrapped_entity(
            origin: OriginFor<T>,
            from: staging_xcm::v5::Location,
            entity: EntityDetailsFor<T, I>,
            foreign_request: ForeignInfoFor<T, I>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::access_entity(who, from, entity, foreign_request)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    pub trait BenchmarkHelper<CollectionId, ItemId, ForeignAuthorityId> {
        fn collection_id(i: u32) -> CollectionId;
        fn item_id(i: u32) -> ItemId;
        fn foreign_authority_id(i: u32) -> ForeignAuthorityId;
    }
    #[cfg(feature = "runtime-benchmarks")]
    impl<CollectionId, ItemId, ForeignAuthorityId>
        BenchmarkHelper<CollectionId, ItemId, ForeignAuthorityId> for ()
    where
        CollectionId: From<u32>,
        ItemId: From<u32>,
        ForeignAuthorityId: From<u32>,
    {
        fn collection_id(i: u32) -> CollectionId {
            i.into()
        }

        fn item_id(i: u32) -> ItemId {
            i.into()
        }

        fn foreign_authority_id(i: u32) -> ForeignAuthorityId {
            i.into()
        }
    }
}
