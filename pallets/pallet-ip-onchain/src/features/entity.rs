use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Adds a new entity to the storage with a unique identifier.
    ///
    /// # It ensures
    /// - The `NextEntityId` is incremented and used as the unique identifier for the new entity.
    /// - Ensures that the entity ID does not already exist in the storage.
    /// - Validates that all provided authors exist in the `Authors` storage if the `authors` parameter is provided.
    /// - Validates that all related entities exist in the `Entities` storage if the `related_entities` parameter is provided.
    /// - Ensures the caller has the necessary access rights to create the entity.
    ///
    /// # Parameters
    /// - `entity_kind`: Specifies the type of the entity (e.g., `Loop`, `Music`, etc.).
    /// - `owner`: The unique authority ID of the owner associated with the entity.
    /// - `url`: A bounded vector containing the metadata URL for the entity.
    /// - `metadata_standard`: The standard format for the metadata.
    /// - `authors`: An optional bounded vector of author IDs linked to the entity.
    /// - `royalty_parts`: An optional bounded vector of wallets defining the royalty distribution for the entity.
    /// - `related_entities`: An optional bounded vector of entity IDs linked as related entities.
    /// - `nft_item_id`: An optional NFT item ID associated with the entity.
    /// - `nft_owner`: An optional account ID representing the owner of the NFT.
    /// - `nft_item_config`: An optional configuration for the NFT item.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::EntityAlreadyExists` if the entity ID already exists in the storage.
    /// - Returns `Error::<T, I>::EntityIdIncrementFailed` if the `NextEntityId` cannot be incremented or initialized.
    /// - Returns `Error::<T, I>::EntityAuthorNotFound` if any of the provided authors do not exist in the `Authors` storage.
    /// - Returns `Error::<T, I>::EntityRelatedEntityNotFound` if any of the provided related entities do not exist in the `Entities` storage.
    /// - Returns an access control error if the caller does not have the necessary rights to create the entity.
    ///
    /// # Events
    /// - Emits `Event::EntityAdded` with the newly created entity ID.
    pub(crate) fn add_new_entity(
        origin: T::AccountId,
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
        Self::ensure_access_right(&origin, &owner, AuthorityAccessSetting::CreateEntity.into())?;
        let collection_id =
            Self::mint_nft_for_entity(&owner, nft_item_id, nft_owner, nft_item_config)?;
        NextEntityId::<T, I>::try_mutate(|maybe_entity_id| -> DispatchResult {
            let entity_id = maybe_entity_id
                .map_or(T::EntityId::initial_value(), Some)
                .ok_or(Error::<T, I>::EntityIdIncrementFailed)?;

            ensure!(
                !Entities::<T, I>::contains_key(entity_id),
                Error::<T, I>::EntityAlreadyExists
            );

            let mut entity_details = EntityDetails {
                entity_kind,
                owner,
                authors: None,
                royalty_parts,
                related_to: None,
                metadata: Metadata {
                    url,
                    standard: metadata_standard,
                    features: metadata_features,
                },
                item_id: nft_item_id,
                collection_id,
            };

            if let Some(new_authors) = authors {
                ensure!(
                    new_authors.iter().all(Authors::<T, I>::contains_key),
                    Error::<T, I>::EntityAuthorNotFound
                );

                entity_details.authors = Some(new_authors);
            }

            if let Some(new_related_entities) = related_entities {
                ensure!(
                    new_related_entities
                        .iter()
                        .all(Entities::<T, I>::contains_key),
                    Error::<T, I>::EntityRelatedEntityNotFound
                );
                entity_details.related_to = Some(new_related_entities);
            }

            Entities::<T, I>::insert(entity_id, entity_details);

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
    /// - Ensures the caller has the necessary access rights to edit the entity.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller attempting to edit the entity.
    /// - `entity_id`: The unique identifier of the entity to be edited.
    /// - `url`: Optional metadata URL for the entity. If `None`, the `metadata` field remains unchanged.
    /// - `metadata_standard`: The metadata standard.
    /// - `owner`: An optional authority ID representing the new owner of the entity. If `None`, the `owner` field remains unchanged.
    /// - `authors`: An optional bounded vector of author IDs to update the entity's authors. If `None`, the `authors` field remains unchanged.
    /// - `royalty_parts`: An optional bounded vector of wallets representing the new royalty parts for the entity. If `None`, the `royalty_parts` field remains unchanged.
    /// - `related_entities`: An optional bounded vector of entity IDs representing the new related entities for the entity. If `None`, the `related_to` field remains unchanged.
    /// - `nft_item_id`: An optional NFT item ID for the entity.
    /// - `nft_owner`: An optional account ID for the NFT owner.
    /// - `nft_item_config`: An optional configuration for the NFT item.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::EntityNotFound` if the entity with the given `entity_id` does not exist in the storage.
    /// - Returns `Error::<T, I>::EntityAuthorNotFound` if any of the provided authors do not exist in the `Authors` storage.
    /// - Returns `Error::<T, I>::EntityRelatedEntityNotFound` if any of the provided related entities do not exist in the `Entities` storage.
    /// - Returns `Error::<T, I>::EntityNftImmutable` if caller try to rewrite item_id for entity.
    /// - Returns an access control error if the caller does not have the necessary rights to edit the entity.
    ///
    /// # Events
    /// - Emits `Event::EntityEdited` with the `entity_id` of the edited entity.
    pub(crate) fn set_entity(
        origin: T::AccountId,
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
        Entities::<T, I>::try_mutate(entity_id, |maybe_entity| -> DispatchResult {
            let entity = maybe_entity.as_mut().ok_or(Error::<T, I>::EntityNotFound)?;

            Self::ensure_access_right(
                &origin,
                &entity.owner,
                AuthorityAccessSetting::EditEntity.into(),
            )?;

            if !entity
                .metadata
                .features
                .has_feature(MetadataFeature::Immutable)
            {
                if let Some(new_url) = url {
                    entity.metadata.url = new_url;
                }

                if let Some(new_metadata_standard) = metadata_standard {
                    entity.metadata.standard = new_metadata_standard;
                }

                if let Some(new_metadata_features) = metadata_features {
                    entity.metadata.features = new_metadata_features;
                }
            }

            if let Some(new_owner) = owner {
                entity.owner = new_owner;
            }

            if let Some(new_authors) = authors {
                ensure!(
                    new_authors.iter().all(Authors::<T, I>::contains_key),
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
                        .all(Entities::<T, I>::contains_key),
                    Error::<T, I>::EntityRelatedEntityNotFound
                );
                entity.related_to = Some(new_related_entities);
            }

            if let Some(nft_item_id) = nft_item_id {
                ensure!(
                    entity.collection_id.is_none() && entity.item_id.is_none(),
                    Error::<T, I>::EntityNftImmutable
                );

                let collection_id = Self::mint_nft_for_entity(
                    &entity.owner,
                    Some(nft_item_id),
                    nft_owner,
                    nft_item_config,
                )?;

                if let Some(collection_id) = collection_id {
                    entity.collection_id = Some(collection_id);
                    entity.item_id = Some(nft_item_id);
                };
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
    /// - Returns `Error::<T, I>::BadFormat` if the `to` ID is less than the `from` ID.
    pub fn get_entities(
        mut from: T::EntityId,
        to: T::EntityId,
    ) -> Result<BoundedVec<(T::EntityId, EntityDetailsFor<T, I>), T::MaxArrayLen>, DispatchError>
    {
        ensure!(to >= from, Error::<T, I>::BadFormat);

        let mut entities = BoundedVec::new();

        while from != to {
            if let Some(entity_details) = Entities::<T, I>::get(from) {
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
