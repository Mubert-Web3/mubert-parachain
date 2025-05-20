use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Adds a new entity to the storage with a unique identifier.
    ///
    /// # It ensures
    /// - The `NextEntityId` is incremented and used as the unique identifier for the new entity.
    /// - Ensures that the entity ID does not already exist in the storage.
    ///
    /// # Parameters
    /// - `entity_kind`: Specifies the type of the entity (e.g., `Loop`, `Music`).
    /// - `owner`: The authority ID of the owner associated with the entity.
    /// - `metadata`: Optional metadata for the entity.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::EntityAlreadyExists` if the entity ID already exists in the storage.
    /// - Returns `Error::<T, I>::EntityIdIncrementFailed` if the `NextEntityId` cannot be incremented or initialized.
    ///
    /// # Events
    /// - Emits `Event::EntityAdded` with the newly created entity ID.
    pub(crate) fn add_new_entity(
        entity_kind: IPEntityKind,
        owner: T::AuthorityId,
        url: BoundedVec<u8, T::MaxLongStringLength>,
        metadata_standard: MetadataStandard,
    ) -> DispatchResult {
        NextEntityId::<T, I>::try_mutate(|maybe_entity_id| -> DispatchResult {
            let entity_id = maybe_entity_id
                .map_or(T::EntityId::initial_value(), |val| Some(val))
                .ok_or(Error::<T, I>::EntityIdIncrementFailed)?;

            ensure!(
                !Entities::<T, I>::contains_key(&entity_id),
                Error::<T, I>::EntityAlreadyExists
            );

            Entities::<T, I>::insert(
                &entity_id,
                EntityDetails {
                    entity_kind,
                    owner,
                    authors: None,
                    royalty_parts: None,
                    related_to: None,
                    metadata: Some(Metadata { url, standard: metadata_standard }),
                },
            );
            Self::deposit_event(Event::EntityAdded { entity_id });

            let new_entity_id = entity_id
                .increment()
                .ok_or(Error::<T, I>::EntityIdIncrementFailed)?;

            *maybe_entity_id = Some(new_entity_id);

            Ok(())
        })
    }

    /// Updates the details of an existing entity in the storage.
    ///
    /// # It ensures
    /// - The entity with the given `entity_id` exists in the storage before making any changes.
    /// - Validates that the caller has the authority to modify the entity details.
    /// - Updates the `metadata` field if a new value is provided.
    /// - Updates the `owner` field if a new value is provided.
    /// - Updates the `authors` field if a new value is provided, ensuring all provided authors exist in the `Authors` storage.
    /// - Updates the `royalty_parts` field if a new value is provided.
    /// - Updates the `related_to` field if a new value is provided, ensuring all related entities exist in the `Entities` storage.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller attempting to edit the entity.
    /// - `entity_id`: The unique identifier of the entity to be edited.
    /// - `url`: Optional metadata URL for the entity. If `None`, the `metadata` field remains unchanged.
    /// - `owner`: An optional authority ID representing the new owner of the entity. If `None`, the `owner` field remains unchanged.
    /// - `authors`: An optional bounded vector of author IDs to update the entity's authors. If `None`, the `authors` field remains unchanged.
    /// - `royalty_parts`: An optional bounded vector of wallets representing the new royalty parts for the entity. If `None`, the `royalty_parts` field remains unchanged.
    /// - `related_entities`: An optional bounded vector of entity IDs representing the new related entities for the entity. If `None`, the `related_to` field remains unchanged.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::EntityNotFound` if the entity with the given `entity_id` does not exist in the storage.
    /// - Returns `Error::<T, I>::EntityAuthorNotFound` if any of the provided authors do not exist in the `Authors` storage.
    /// - Returns `Error::<T, I>::EntityNotFound` if any of the provided related entities do not exist in the `Entities` storage.
    ///
    /// # Events
    /// - Emits `Event::EntityEdited` with the `entity_id` of the edited entity.
    pub(crate) fn set_entity(
        origin: T::AccountId,
        entity_id: T::EntityId,
        url: Option<BoundedVec<u8, T::MaxLongStringLength>>,
        metadata_standard: MetadataStandard,
        owner: Option<T::AuthorityId>,
        authors: Option<BoundedVec<T::AuthorId, T::MaxEntityAuthors>>,
        royalty_parts: Option<BoundedVec<Wallet<T::AccountId>, T::MaxRoyaltyParts>>,
        related_entities: Option<BoundedVec<T::EntityId, T::MaxRelatedEntities>>,
    ) -> DispatchResult {
        Entities::<T, I>::try_mutate(&entity_id, |maybe_entity| -> DispatchResult {
            let entity = maybe_entity.as_mut().ok_or(Error::<T, I>::EntityNotFound)?;

            Self::ensure_authority_owner(&origin, &entity.owner)?;

            if let Some(new_url) = url {
                entity.metadata = Some(Metadata { url: new_url, standard: metadata_standard });
            }

            if let Some(new_owner) = owner {
                entity.owner = new_owner;
            }

            if let Some(new_authors) = authors {
                ensure!(
                    new_authors
                        .iter()
                        .all(|author_id| Authors::<T, I>::contains_key(author_id)),
                    Error::<T, I>::EntityAuthorNotFound
                );

                entity.authors = Some(new_authors);
            }

            if let Some(new_royalty_parts) = royalty_parts {
                entity.royalty_parts = Some(new_royalty_parts);
            }

            if let Some(new_related_entities) = related_entities {
                ensure!(
                    new_related_entities
                        .iter()
                        .all(|entity_id| Entities::<T, I>::contains_key(entity_id)),
                    Error::<T, I>::EntityNotFound
                );
                entity.related_to = Some(new_related_entities);
            }

            Self::deposit_event(Event::EntityEdited { entity_id });

            Ok(())
        })
    }

    /// Retrieves the details of an entity from the storage.
    ///
    /// # It ensures
    /// - The function performs a read-only operation and does not modify the storage.
    ///
    /// # Parameters
    /// - `entity_id`: The unique identifier of the entity to retrieve.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::EntityNotFound` if the entity with the given `entity_id` does not exist in the storage.
    ///
    /// # Returns
    /// - `EntityDetailsFor<T, I>` containing the details of the entity if it exists.
    pub fn get_entity(entity_id: T::EntityId) -> Result<EntityDetailsFor<T, I>, DispatchError> {
        Ok(Entities::<T, I>::get(entity_id).ok_or(Error::<T, I>::EntityNotFound)?)
    }

    /// Fetches a paginated list of entities from storage.
    ///
    /// # Parameters
    /// - `from`: The starting entity ID to begin pagination.
    /// - `to`: The ending entity ID to stop pagination (exclusive).
    ///
    /// # Returns
    /// - A bounded vector containing tuples of entity IDs and their corresponding details.
    /// - Each tuple represents an entity ID and the associated `EntityDetails`.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::LimitExceeded` if the number of entities exceeds the maximum array length.
    /// - Returns `Error::<T, I>::EntityIdIncrementFailed` if the `from` ID cannot be incremented.
    pub fn get_entities(
        mut from: T::EntityId,
        to: T::EntityId,
    ) -> Result<BoundedVec<(T::EntityId, EntityDetailsFor<T, I>), T::MaxArrayLen>, DispatchError>
    {
        ensure!(to >= from, Error::<T, I>::BadFormat);

        let mut entities = BoundedVec::new();

        while from != to {
            if let Some(entity_details) = Entities::<T, I>::get(&from) {
                entities
                    .try_push((from, entity_details))
                    .map_err(|_| Error::<T, I>::LimitExceeded)?;
            }

            from = from
                .increment()
                .ok_or(Error::<T, I>::EntityIdIncrementFailed)?;
        }

        Ok(entities)
    }
}
