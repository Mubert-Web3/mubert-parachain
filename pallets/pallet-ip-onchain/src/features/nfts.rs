use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Creates a new NFT collection.
    ///
    /// # It ensures
    /// - A new collection is created if NFT support is enabled.
    ///
    /// # Parameters
    /// - `who`: The account ID of the creator of the collection.
    /// - `admin`: The account ID of the admin for the collection.
    /// - `config`: The configuration settings for the collection.
    ///
    /// # Returns
    /// - `Option<T::CollectionId>` containing the ID of the newly created collection if NFT support is enabled.
    ///
    /// # Errors
    /// - Returns an error if the collection creation fails.
    pub fn create_new_collection(
        who: T::AccountId,
        admin: T::AccountId,
        config: T::CollectionConfig,
    ) -> Result<Option<T::CollectionId>, DispatchError> {
        let enabled = NftsSupport::<T, I>::get().unwrap_or_default();

        if enabled {
            let collection_id = T::Nfts::create_collection(&who, &admin, &config)?;
            Ok(Some(collection_id))
        } else {
            Ok(None)
        }
    }

    /// Mints an NFT for a specific entity.
    ///
    /// # It ensures
    /// - An NFT is minted into the specified collection and assigned to the specified owner.
    ///
    /// # Parameters
    /// - `authority_id`: The unique identifier of the authority associated with the entity.
    /// - `item_id`: An optional item ID for the NFT.
    /// - `nft_owner`: An optional account ID representing the owner of the NFT.
    /// - `config`: An optional configuration for the NFT item.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::EntityNftOwnerMustBeSpecified` if the `nft_owner` is not provided.
    /// - Returns an error if the minting process fails.
    pub fn mint_nft_for_entity(
        authority_id: &T::AuthorityId,
        item_id: Option<T::ItemId>,
        nft_owner: Option<T::AccountId>,
        config: Option<pallet_nfts::ItemConfig>,
    ) -> Result<Option<T::CollectionId>, DispatchError> {
        if item_id.is_none() {
            return Ok(None);
        };

        ensure!(
            nft_owner.is_some(),
            Error::<T, I>::EntityNftOwnerMustBeSpecified
        );

        let authority = Self::get_authority(*authority_id)?;

        if let (Some(collection_id), Some(item_id), Some(nft_owner)) =
            (authority.collection_id, item_id, nft_owner)
        {
            T::Nfts::mint_into(
                &collection_id,
                &item_id,
                &nft_owner,
                &config.unwrap_or_default(),
                true,
            )?;

            Ok(Some(collection_id))
        } else {
            Ok(None)
        }
    }

    /// Init collection id for authority, if nft support is enabled.
    ///
    /// # It ensures
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller.
    /// - `config`: An optional collection configuration for initializing the authority's NFT collection.
    ///
    /// # Returns
    /// - `Option<T::CollectionId>` containing the initialized collection ID if successful.
    ///
    /// # Errors
    pub fn init_collection_id_checked(
        origin: T::AccountId,
        config: Option<T::CollectionConfig>,
    ) -> Result<Option<T::CollectionId>, DispatchError> {
        if let Some(config) = config {
            let collection_id =
                Self::create_new_collection(origin.clone(), origin.clone(), config)?;

            Ok(collection_id)
        } else {
            Ok(None)
        }
    }

    /// Toggles the support for NFTs in the system.
    ///
    /// # It ensures
    /// - The NFT support flag is toggled between enabled and disabled states.
    ///
    /// # Errors
    /// - Returns an error if the mutation of the NFT support flag fails.
    pub fn toggle_nfts_support() -> DispatchResult {
        NftsSupport::<T, I>::try_mutate(|support| -> DispatchResult {
            let current = support.unwrap_or_default();
            *support = Some(!current);
            Ok(())
        })
    }
}
