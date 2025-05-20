#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use polkadot_sdk::polkadot_sdk_frame as frame;

use frame::traits::Contains;

use frame::prelude::*;
use frame::traits::Incrementable;

mod features;
mod types;

pub use pallet::*;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const LOG_TARGET: &'static str = "runtime::ip-onchain";

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

        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as polkadot_sdk::frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type Authorities<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        T::AuthorityId,
        AuthorityDetails<T::AccountId, T::MaxShortStringLength>,
    >;

    #[pallet::storage]
    pub(super) type Authors<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        T::AuthorId,
        AuthorDetails<T::AuthorityId, T::MaxShortStringLength, T::MaxLongStringLength>,
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

        /// Entity errors
        EntityAlreadyExists,
        EntityNotFound,
        EntityIdIncrementFailed,
        EntityTooManyAuthors,
        EntityAuthorNotFound,
        EntityTooManyRoyaltyParts,
        EntityTooManyRelatedEntities,
        EntityRelatedEntityNotFound,

        /// General Errors
        Overflow, // checked_add failed
        LimitExceeded,
        BadFormat,

        /// Permissions Errors
        NoPermission,
        /// Whitelist Errors
        NotWhitelisted,
    }

    /// Calls
    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Authors calls
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_author(
            origin: OriginFor<T>,
            nickname: BoundedVec<u8, T::MaxShortStringLength>,
            real_name: Option<BoundedVec<u8, T::MaxLongStringLength>>,
            owner: T::AuthorityId,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::add_new_author(nickname, real_name, owner)?;
            Ok(())
        }
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn edit_author(
            origin: OriginFor<T>,
            author_id: T::AuthorId,
            real_name: Option<BoundedVec<u8, T::MaxLongStringLength>>,
            owner: Option<T::AuthorityId>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::set_author(origin, author_id, real_name, owner)?;
            Ok(())
        }

        /// Authority calls
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn create_authority(
            origin: OriginFor<T>,
            name: BoundedVec<u8, T::MaxShortStringLength>,
            owner: T::AccountId,
            authority_kind: AuthorityKind,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::add_new_authority(name, owner, authority_kind)?;
            Ok(())
        }
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn edit_authority(
            origin: OriginFor<T>,
            authority_id: T::AuthorityId,
            name: Option<BoundedVec<u8, T::MaxShortStringLength>>,
            owner: Option<T::AccountId>,
            authority_kind: Option<AuthorityKind>,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::set_authority(origin, authority_id, name, owner, authority_kind)?;
            Ok(())
        }

        /// Entity calls
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn create_entity(
            origin: OriginFor<T>,
            entity_kind: IPEntityKind,
            owner: T::AuthorityId,
            url: BoundedVec<u8, T::MaxLongStringLength>,
            metadata_standard: MetadataStandard,
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            ensure!(
                T::WhiteListChecker::contains(&origin),
                Error::<T, I>::NotWhitelisted
            );
            Self::add_new_entity(entity_kind, owner, url, metadata_standard)?;
            Ok(())
        }
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn edit_entity(
            origin: OriginFor<T>,
            entity_id: T::EntityId,
            url: Option<BoundedVec<u8, T::MaxLongStringLength>>,
            metadata_standard: MetadataStandard,
            owner: Option<T::AuthorityId>,
            authors: Option<BoundedVec<T::AuthorId, T::MaxEntityAuthors>>,
            royalty_parts: Option<BoundedVec<Wallet<T::AccountId>, T::MaxRoyaltyParts>>,
            related_entities: Option<BoundedVec<T::EntityId, T::MaxRelatedEntities>>,
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
                owner,
                authors,
                royalty_parts,
                related_entities,
            )?;
            Ok(())
        }
    }
}
