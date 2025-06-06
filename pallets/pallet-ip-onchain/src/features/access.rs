use crate::*;

use enumflags2::BitFlags;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Adds initial access settings for an authority and account.
    ///
    /// # It ensures
    /// - Ensures that the access settings are added only if they do not already exist.
    ///
    /// # Parameters
    /// - `authority_id`: The unique identifier of the authority.
    /// - `account_id`: The unique identifier of the account to which access is being granted.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthoritiesAccessExist` if access settings already exist for the given authority and account.
    ///
    /// # Events
    /// - Emits `Event::AuthoritiesAccessAdded` with the `authority_id` and `account_id`.
    pub fn add_first_access(
        authority_id: T::AuthorityId,
        account_id: T::AccountId,
    ) -> DispatchResult {
        ensure!(
            !AuthoritiesAccess::<T, I>::contains_key(authority_id, &account_id),
            Error::<T, I>::AuthoritiesAccessExist
        );

        AuthoritiesAccess::<T, I>::insert(
            authority_id,
            account_id.clone(),
            AuthorityAccessSettings::all(),
        );

        Self::deposit_event(Event::AuthoritiesAccessAdded {
            authority_id,
            account_id,
        });

        Ok(())
    }

    /// Adds specific access settings for an authority and account.
    ///
    /// # It ensures
    /// - Validates that the caller has the required access rights to edit access settings.
    /// - Ensures that the access settings are added only if they do not already exist.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller attempting to add access.
    /// - `authority_id`: The unique identifier of the authority.
    /// - `account_id`: The unique identifier of the account to which access is being granted.
    /// - `access`: The access settings to be added.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthoritiesAccessExist` if access settings already exist for the given authority and account.
    /// - Returns `Error::<T, I>::NotAuthorized` if the caller does not have the required access rights.
    ///
    /// # Events
    /// - Emits `Event::AuthoritiesAccessAdded` with the `authority_id` and `account_id`.
    pub(crate) fn add_access(
        origin: T::AccountId,
        authority_id: T::AuthorityId,
        account_id: T::AccountId,
        access: AuthorityAccessSettings,
    ) -> DispatchResult {
        Self::ensure_access_right(
            &origin,
            &authority_id,
            AuthorityAccessSetting::EditAccess.into(),
        )?;

        ensure!(
            !AuthoritiesAccess::<T, I>::contains_key(authority_id, &account_id),
            Error::<T, I>::AuthoritiesAccessExist
        );

        AuthoritiesAccess::<T, I>::insert(authority_id, account_id.clone(), access);

        Self::deposit_event(Event::AuthoritiesAccessAdded {
            authority_id,
            account_id,
        });

        Ok(())
    }

    /// Updates access settings for an authority and account.
    ///
    /// # It ensures
    /// - Validates that the caller has the required access rights to edit access settings.
    /// - Ensures that the access settings exist before attempting to update them.
    ///
    /// # Parameters
    /// - `origin`: The account ID of the caller attempting to update access.
    /// - `authority_id`: The unique identifier of the authority.
    /// - `account_id`: The unique identifier of the account whose access settings are being updated.
    /// - `new_access`: The new access settings to be applied.
    ///
    /// # Errors
    /// - Returns `Error::<T, I>::AuthoritiesAccessNotExist` if access settings do not exist for the given authority and account.
    /// - Returns `Error::<T, I>::NotAuthorized` if the caller does not have the required access rights.
    ///
    /// # Events
    /// - Emits `Event::AuthoritiesAccessChanged` with the `authority_id` and `account_id`.
    pub(crate) fn set_access(
        origin: T::AccountId,
        authority_id: T::AuthorityId,
        account_id: T::AccountId,
        new_access: AuthorityAccessSettings,
    ) -> DispatchResult {
        Self::ensure_access_right(
            &origin,
            &authority_id,
            AuthorityAccessSetting::EditAccess.into(),
        )?;

        AuthoritiesAccess::<T, I>::try_mutate(
            authority_id,
            account_id.clone(),
            |maybe_author| -> DispatchResult {
                ensure!(
                    maybe_author.is_some(),
                    Error::<T, I>::AuthoritiesAccessNotExist
                );
                *maybe_author = Some(new_access);

                Self::deposit_event(Event::AuthoritiesAccessChanged {
                    authority_id,
                    account_id,
                });

                Ok(())
            },
        )
    }

    /// Validates that an account has the required access rights for an authority.
    ///
    /// # It ensures
    /// - Ensures that the account has the specified access rights for the given authority.
    ///
    /// # Parameters
    /// - `who`: The account ID of the entity whose access rights are being validated.
    /// - `authority_id`: The unique identifier of the authority.
    /// - `access_flags`: The access flags required for the operation.
    ///
    /// # Errors
    /// (access control error)
    /// - Returns `Error::<T, I>::AuthoritiesAccessNotFound` if access settings do not exist for the given authority and account.
    /// - Returns `Error::<T, I>::NotAuthorized` if the account does not have the required access rights.
    ///
    /// # Events
    /// - ///
    pub fn ensure_access_right(
        who: &T::AccountId,
        authority_id: &T::AuthorityId,
        access_flags: BitFlags<AuthorityAccessSetting, u64>,
    ) -> DispatchResult {
        ensure!(
            AuthoritiesAccess::<T, I>::get(authority_id, who)
                .ok_or(Error::<T, I>::AuthoritiesAccessNotFound)?
                .has_access(access_flags),
            Error::<T, I>::NotAuthorized
        );
        Ok(())
    }
}
