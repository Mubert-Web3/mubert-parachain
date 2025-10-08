use super::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use polkadot_sdk::staging_xcm;

pub type ForeignInfoFor<T, I = ()> = ForeignInfo<
    <T as frame_system::Config>::AccountId,
    <T as Config<I>>::EntityId,
    <T as Config<I>>::AuthorityId,
    staging_xcm::v5::Location,
    <T as Config<I>>::ForeignAuthorityId,
>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ForeignInfo<AccountId, EntityId, AuthorityId, Location, ForeignAuthorityId> {
    pub who: AccountId,
    pub here_authority_id: AuthorityId,
    pub entity_id: EntityId,
    pub foreign_location: Location,
    pub foreign_authority_id: ForeignAuthorityId,
    pub approval: bool,
    pub done: bool,
}

pub trait ForeignMover<EntityDetails, AuthorityId, ForeignAuthorityId, ForeignRequest> {
    fn send_wrapped(
        from: staging_xcm::v5::Location,
        entity_details: EntityDetails,
        foreign_request: ForeignRequest,
    ) -> DispatchResult;
    fn covert_authority_id(id: ForeignAuthorityId) -> AuthorityId;
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Sets a foreign authority request for a given entity.
    ///
    /// # It ensures
    /// - A new foreign authority request is created and stored if it does not already exist.
    ///
    /// # Parameters
    /// - `who`: The account ID of the entity making the request.
    /// - `entity_id`: The ID of the entity associated with the request.
    /// - `here_authority_id`: The authority ID within the current chain.
    /// - `foreign_authority_id`: The authority ID in the foreign chain.
    /// - `foreign_location`: The location of the foreign chain.
    ///
    /// # Errors
    /// - `ForeignIdIncrementFailed`: If the foreign request ID cannot be incremented.
    /// - `ForeignAlreadyExists`: If a foreign request with the same ID already exists.
    ///
    /// # Events
    /// - `ForeignRequestsAdded`: Emitted when a new foreign request is successfully added.
    pub fn set_foreign_authority_request(
        who: T::AccountId,
        entity_id: T::EntityId,
        here_authority_id: T::AuthorityId,
        foreign_authority_id: T::ForeignAuthorityId,
        foreign_location: staging_xcm::v5::Location,
    ) -> DispatchResult {
        NextForeignRequests::<T, I>::try_mutate(|maybe_request_id| -> DispatchResult {
            let request_id = maybe_request_id
                .map_or(T::ForeignRequestId::initial_value(), Some)
                .ok_or(Error::<T, I>::ForeignIdIncrementFailed)?;

            ensure!(
                !ForeignsRequests::<T, I>::contains_key(request_id),
                Error::<T, I>::ForeignAlreadyExists
            );

            ForeignsRequests::<T, I>::insert(
                request_id,
                ForeignInfo {
                    who,
                    here_authority_id,
                    entity_id,
                    foreign_location,
                    foreign_authority_id,
                    approval: false,
                    done: false,
                },
            );

            Self::deposit_event(Event::ForeignRequestsAdded { request_id });

            let new_request_id = request_id
                .increment()
                .ok_or(Error::<T, I>::ForeignIdIncrementFailed)?;

            *maybe_request_id = Some(new_request_id);

            Ok(())
        })
    }

    /// Approves a foreign request for an entity.
    ///
    /// # It ensures
    /// - The foreign request is marked as approved.
    ///
    /// # Parameters
    /// - `who`: The account ID of the user approving the request.
    /// - `entity_id`: The ID of the entity associated with the foreign request.
    /// - `foreign_request_id`: The ID of the foreign request to approve.
    ///
    /// # Errors
    /// - `EntityNotFound`: If the specified entity does not exist.
    /// - `ForeignNotFound`: If the specified foreign request does not exist.
    /// - `AccessDenied`: If the user does not have the required access rights.
    ///
    /// # Events
    /// - `EntityWraped`: Emitted when the entity is successfully wrapped.
    pub fn foreign_request_approve(
        who: T::AccountId,
        entity_id: T::EntityId,
        foreign_request_id: T::ForeignRequestId,
    ) -> Result<(), DispatchError> {
        Entities::<T, I>::try_mutate(entity_id, |maybe_entity| -> DispatchResult {
            let entity = maybe_entity.as_mut().ok_or(Error::<T, I>::EntityNotFound)?;

            Self::ensure_access_right(
                &who,
                &entity.owner,
                AuthorityAccessSetting::WrapEntity.into(),
            )?;

            ForeignsRequests::<T, I>::try_mutate(
                foreign_request_id,
                |maybe_entity_request| -> DispatchResult {
                    let foreign_request = maybe_entity_request
                        .as_mut()
                        .ok_or(Error::<T, I>::ForeignNotFound)?;

                    foreign_request.approval = true;

                    Ok(())
                },
            )?;

            Self::deposit_event(Event::EntityWraped { entity_id });

            Ok(())
        })
    }

    /// Handles the process of taking a foreign request.
    ///
    /// # It ensures
    /// - The caller has permission to take the foreign request.
    /// - The foreign request is approved and not already completed.
    /// - The entity associated with the foreign request is moved to the foreign authority.
    ///
    /// # Parameters
    /// - `who`: The account ID of the caller attempting to take the foreign request.
    /// - `foreign_request_id`: The unique identifier of the foreign request to be taken.
    /// - `src`: The location from which the foreign request originates.
    ///
    /// # Errors
    /// - `ForeignNotFound`: If the foreign request does not exist.
    /// - `ForeignRequestNoPermission`: If the caller does not have permission to take the foreign request.
    /// - `ForeignRequestNotApproved`: If the foreign request has not been approved.
    /// - `ForeignRequestAlreadyDone`: If the foreign request has already been completed.
    /// - `EntityNotFound`: If the entity associated with the foreign request does not exist.
    ///
    /// # Events
    /// - `EntityMovedForeign`: Emitted when the entity is successfully moved to the foreign authority.
    /// - `ForeignRequestsDone`: Emitted when the foreign request is marked as completed.
    pub fn foreign_request_take(
        who: T::AccountId,
        foreign_request_id: T::ForeignRequestId,
        src: staging_xcm::v5::Location,
    ) -> Result<(), DispatchError> {
        let foreign_info = ForeignsRequests::<T, I>::get(&foreign_request_id)
            .ok_or(Error::<T, I>::ForeignNotFound)?; // take a foreign authority

        ensure!(
            foreign_info.who == who,
            Error::<T, I>::ForeignRequestNoPermission
        );
        ensure!(
            foreign_info.approval == true,
            Error::<T, I>::ForeignRequestNotApproved
        );
        ensure!(
            foreign_info.done != true,
            Error::<T, I>::ForeignRequestAlreadyDone
        );

        Entities::<T, I>::try_mutate(&foreign_info.entity_id, |maybe_entity| -> DispatchResult {
            let entity = maybe_entity.as_mut().ok_or(Error::<T, I>::EntityNotFound)?;

            entity.owner = foreign_info.here_authority_id;

            Self::deposit_event(Event::EntityMovedForeign {
                entity_id: foreign_info.entity_id,
            });

            Ok(())
        })?;

        let entity =
            Entities::<T, I>::get(foreign_info.entity_id).ok_or(Error::<T, I>::EntityNotFound)?;

        T::ForeignMover::send_wrapped(src, entity, foreign_info)?;

        ForeignsRequests::<T, I>::try_mutate(
            &foreign_request_id,
            |maybe_foreign_request| -> DispatchResult {
                let foreign_request = maybe_foreign_request
                    .as_mut()
                    .ok_or(Error::<T, I>::ForeignNotFound)?;

                foreign_request.done = true;

                Self::deposit_event(Event::ForeignRequestsDone {
                    request_id: foreign_request_id,
                });

                Ok(())
            },
        )
    }

    /// Accesses an entity and updates its ownership based on a foreign request.
    ///
    /// # It ensures
    /// - The entity's ownership is updated to the foreign authority ID.
    /// - A new entity ID is generated and associated with the entity.
    /// - The entity is stored in the `Entities` storage map.
    /// - An event is emitted to signal the addition of the entity.
    ///
    /// # Parameters
    /// - `_who`: The account ID of the caller (unused in this function).
    /// - `from`: The foreign location from which the entity is accessed.
    /// - `entity`: The details of the entity being accessed.
    /// - `foreign_request`: The foreign request containing information about the foreign authority and entity.
    ///
    /// # Errors
    /// - `EntityIdIncrementFailed`: If the entity ID cannot be incremented.
    /// - `EntityAlreadyExists`: If the entity ID already exists in the `Entities` storage map.
    ///
    /// # Events
    /// - `EntityAdded`: Emitted when a new entity is successfully added.
    pub fn access_entity(
        _who: T::AccountId,
        from: staging_xcm::v5::Location,
        mut entity: EntityDetailsFor<T, I>,
        foreign_request: ForeignInfoFor<T, I>,
    ) -> Result<(), DispatchError> {
        entity.owner = T::ForeignMover::covert_authority_id(foreign_request.foreign_authority_id);
        NextEntityId::<T, I>::try_mutate(|maybe_entity_id| -> DispatchResult {
            let entity_id = maybe_entity_id
                .map_or(T::EntityId::initial_value(), Some)
                .ok_or(Error::<T, I>::EntityIdIncrementFailed)?;

            ensure!(
                !Entities::<T, I>::contains_key(entity_id),
                Error::<T, I>::EntityAlreadyExists
            );

            Entities::<T, I>::insert(entity_id, entity);
            EntitiesPointer::<T, I>::insert(entity_id, (from, foreign_request.entity_id));

            Self::deposit_event(Event::EntityAdded { entity_id });

            let new_entity_id = entity_id
                .increment()
                .ok_or(Error::<T, I>::EntityIdIncrementFailed)?;

            *maybe_entity_id = Some(new_entity_id);

            Ok(())
        })
    }
}
