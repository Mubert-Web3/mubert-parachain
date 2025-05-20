use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Adds a new author to the storage with a unique identifier.
    ///
    /// # It ensures
    /// - Ensures that the `NextAuthorId` is incremented and used as the unique identifier for the new author.
    /// - Validates that the author ID does not already exist in the storage.
    ///
    /// # Parameters
    /// - `nickname`: A bounded vector representing the nickname of the author. This is a required field.
    /// - `real_name`: An optional bounded vector representing the real name of the author. This field is optional.
    /// - `owner`: The authority ID of the owner associated with the author.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthorAlreadyExists` if the author ID already exists in the storage.
    /// - Returns `Error::<T, I>::AuthorIdIncrementFailed` if the `NextAuthorId` cannot be incremented or initialized.
    ///
    /// # Events
    /// - Emits `Event::AuthorAdded` with the newly created author ID.
    pub(crate) fn add_new_author(
        nickname: BoundedVec<u8, T::MaxShortStringLength>,
        real_name: Option<BoundedVec<u8, T::MaxLongStringLength>>,
        owner: T::AuthorityId,
    ) -> DispatchResult {
        NextAuthorId::<T, I>::try_mutate(|maybe_author_id| -> DispatchResult {
            let author_id = maybe_author_id
                .map_or(T::AuthorId::initial_value(), |val| Some(val))
                .ok_or(Error::<T, I>::AuthorIdIncrementFailed)?;

            ensure!(
                !Authors::<T, I>::contains_key(&author_id),
                Error::<T, I>::AuthorAlreadyExists
            );

            Authors::<T, I>::insert(
                &author_id,
                AuthorDetails {
                    nickname,
                    real_name,
                    owner,
                },
            );
            Self::deposit_event(Event::AuthorAdded { author_id });

            let new_author_id = author_id
                .increment()
                .ok_or(Error::<T, I>::AuthorIdIncrementFailed)?;

            *maybe_author_id = Some(new_author_id);

            Ok(())
        })
    }

    /// Edits an existing author's details in the storage.
    ///
    /// # It ensures
    /// - Ensures the author with the given `author_id` exists in the storage before making any changes.
    /// - Updates the `real_name` field if a new value is provided.
    /// - Updates the `owner` field if a new value is provided.
    /// - Validates that the caller has the authority to modify the author's details.
    ///
    /// # Parameters
    /// - `author_id`: The unique identifier of the author to be edited.
    /// - `real_name`: An optional bounded vector representing the new real name of the author. If `None`, the `real_name` field remains unchanged.
    /// - `owner`: An optional authority ID representing the new owner of the author. If `None`, the `owner` field remains unchanged.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthorNotFound` if the author with the given `author_id` does not exist in the storage.
    ///
    /// # Events
    /// - Emits `Event::AuthorEdited` with the `author_id` of the edited author.
    pub(crate) fn set_author(
        origin: T::AccountId,
        author_id: T::AuthorId,
        real_name: Option<BoundedVec<u8, T::MaxLongStringLength>>,
        owner: Option<T::AuthorityId>,
    ) -> DispatchResult {
        // todo no change fast return
        Authors::<T, I>::try_mutate(&author_id, |maybe_author| -> DispatchResult {
            let author = maybe_author.as_mut().ok_or(Error::<T, I>::AuthorNotFound)?;

            Self::ensure_authority_owner(&origin, &author.owner)?;

            if let Some(new_real_name) = real_name {
                author.real_name = Some(new_real_name);
            }

            if let Some(new_owner) = owner {
                author.owner = new_owner;
            }

            Self::deposit_event(Event::AuthorEdited { author_id });

            Ok(())
        })
    }

    /// Fetches a paginated list of authors from storage.
    ///
    /// # Parameters
    /// - `from`: The starting author ID to begin pagination.
    /// - `to`: The ending author ID to stop pagination (exclusive).
    ///
    /// # Returns
    /// - A bounded vector containing tuples of author IDs and their corresponding details.
    /// - Each tuple represents an author ID and the associated `AuthorDetails`.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::LimitExceeded` if the number of authors exceeds the maximum array length.
    /// - Returns `Error::<T, I>::AuthorIdIncrementFailed` if the `from` ID cannot be incremented.
    pub fn get_authors(
        mut from: T::AuthorId,
        to: T::AuthorId,
    ) -> Result<BoundedVec<(T::AuthorId, AuthorFor<T, I>), T::MaxArrayLen>, DispatchError> {
        ensure!(to >= from, Error::<T, I>::BadFormat);

        let mut authors = BoundedVec::new();

        while from != to {
            if let Some(author_details) = Authors::<T, I>::get(&from) {
                authors
                    .try_push((from, author_details))
                    .map_err(|_| Error::<T, I>::LimitExceeded)?;
            }

            from = from
                .increment()
                .ok_or(Error::<T, I>::AuthorIdIncrementFailed)?;
        }

        Ok(authors)
    }

    /// Fetches the details of a specific author from storage.
    ///
    /// # Parameters
    /// - `author_id`: The unique identifier of the author to fetch.
    ///
    /// # Returns
    /// - The `AuthorDetails` associated with the given `author_id` if it exists.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthorNotFound` if no author with the given `author_id` exists in the storage.
    pub fn get_author(author_id: T::AuthorId) -> Result<AuthorFor<T, I>, DispatchError> {
        Ok(Authors::<T, I>::get(author_id).ok_or(Error::<T, I>::AuthorNotFound)?)
    }
}
