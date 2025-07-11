use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Adds a new authority to the storage with a unique identifier.
    ///
    /// # It ensures
    /// - The `NextAuthorityId` is incremented and used as the unique identifier for the new authority.
    /// - Ensures that the authority ID does not already exist in the storage.
    /// - The `add_first_access` function is called to initialize access rights for the new authority.
    ///
    /// # Parameters
    /// - `name`: A bounded vector representing the name of the authority. This is a required field.
    /// - `owner`: The account ID of the owner associated with the authority.
    /// - `authority_kind`: The type or category of the authority being added.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthorityAlreadyExists` if the authority ID already exists in the storage.
    /// - Returns `Error::<T, I>::AuthorityIdIncrementFailed` if the `NextAuthorityId` cannot be incremented or initialized.
    ///
    /// # Events
    /// - Emits `Event::AuthorityAdded` with the newly created authority ID.
    pub(crate) fn add_new_authority(
        origin: T::AccountId,
        name: BoundedVec<u8, T::MaxShortStringLength>,
        authority_kind: AuthorityKind,
    ) -> DispatchResult {
        NextAuthorityId::<T, I>::try_mutate(|maybe_authority_id| -> DispatchResult {
            let authority_id = maybe_authority_id
                .map_or(T::AuthorityId::initial_value(), Some)
                .ok_or(Error::<T, I>::AuthorityIdIncrementFailed)?;

            ensure!(
                !Authorities::<T, I>::contains_key(authority_id),
                Error::<T, I>::AuthorityAlreadyExists
            );

            Authorities::<T, I>::insert(
                authority_id,
                AuthorityDetails {
                    authority_kind,
                    name,
                },
            );
            Self::deposit_event(Event::AuthorityAdded { authority_id });

            Self::add_first_access(authority_id, origin)?;

            let new_authority_id = authority_id
                .increment()
                .ok_or(Error::<T, I>::AuthorityIdIncrementFailed)?;

            *maybe_authority_id = Some(new_authority_id);

            Ok(())
        })
    }

    /// Edits an existing authority's details in the storage.
    ///
    /// # It ensures
    /// - The authority with the given `authority_id` exists in the storage before making any changes.
    /// - The caller (`origin`) has the necessary access rights to edit the authority.
    /// - Updates the `name` field if a new value is provided.
    /// - Updates the `authority_kind` field if a new value is provided.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller, which must have the required access rights for the authority.
    /// - `authority_id`: The unique identifier of the authority to be edited.
    /// - `name`: An optional bounded vector representing the new name of the authority. If `None`, the `name` field remains unchanged.
    /// - `authority_kind`: An optional value representing the new type or category of the authority. If `None`, the `authority_kind` field remains unchanged.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthorityNotFound` if the authority with the given `authority_id` does not exist in the storage.
    /// - Returns an access control error
    ///
    /// # Events
    /// - Emits `Event::AuthorityEdited` with the `authority_id` of the edited authority.
    pub(crate) fn set_authority(
        origin: T::AccountId,
        authority_id: T::AuthorityId,
        name: Option<BoundedVec<u8, T::MaxShortStringLength>>,
        authority_kind: Option<AuthorityKind>,
    ) -> DispatchResult {
        Self::ensure_access_right(
            &origin,
            &authority_id,
            AuthorityAccessSetting::EditAuthority.into(),
        )?;

        Authorities::<T, I>::try_mutate(authority_id, |maybe_authority| -> DispatchResult {
            let authority = maybe_authority
                .as_mut()
                .ok_or(Error::<T, I>::AuthorityNotFound)?;

            if let Some(new_name) = name {
                authority.name = new_name;
            }

            if let Some(new_authority_kind) = authority_kind {
                authority.authority_kind = new_authority_kind;
            }

            Ok(())
        })?;

        Self::deposit_event(Event::AuthorityEdited { authority_id });

        Ok(())
    }

    /// Fetches a paginated list of authorities from storage.
    ///
    /// # It ensures
    /// - The function performs a read-only operation and does not modify the storage.
    ///
    /// # Parameters
    /// - `from`: The starting authority ID to begin pagination.
    /// - `to`: The ending authority ID to stop pagination (exclusive).
    ///
    /// # Returns
    /// - A bounded vector containing tuples of authority IDs and their corresponding details.
    /// - Each tuple represents an authority ID and the associated `AuthorityDetails`.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::LimitExceeded` if the number of authorities exceeds the maximum array length.
    /// - Returns `Error::<T, I>::AuthorityIdIncrementFailed` if the `from` ID cannot be incremented.
    pub fn get_authorities(
        mut from: T::AuthorityId,
        to: T::AuthorityId,
    ) -> Result<
        BoundedVec<(T::AuthorityId, AuthorityDetailsFor<T, I>), T::MaxArrayLen>,
        DispatchError,
    > {
        ensure!(to >= from, Error::<T, I>::BadFormat);

        let mut authorities = BoundedVec::new();

        while from != to {
            if let Some(authority_details) = Authorities::<T, I>::get(from) {
                authorities
                    .try_push((from, authority_details))
                    .map_err(|_| Error::<T, I>::LimitExceeded)?;
            }

            from = from
                .increment()
                .ok_or(Error::<T, I>::AuthorityIdIncrementFailed)?;
        }

        Ok(authorities)
    }

    /// Fetches the details of a specific authority from storage.
    ///
    /// # It ensures
    /// - The function performs a read-only operation and does not modify the storage.
    ///
    /// # Parameters
    /// - `authority_id`: The unique identifier of the authority to retrieve.
    ///
    /// # Returns
    /// - `AuthorityDetails` containing the details of the authority if it exists.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthorityNotFound` if the authority with the given `authority_id` does not exist in the storage.
    pub fn get_authority(
        authority_id: T::AuthorityId,
    ) -> Result<AuthorityDetailsFor<T, I>, DispatchError> {
        Ok(Authorities::<T, I>::get(authority_id).ok_or(Error::<T, I>::AuthorityNotFound)?)
    }
}
