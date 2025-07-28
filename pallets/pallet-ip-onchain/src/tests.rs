use polkadot_sdk::polkadot_sdk_frame as frame;

use crate::{
    mock::*, AuthorDetails, Authorities, AuthorityDetails, Authors, Entities, EntityDetails,
    IPEntityKind, Metadata, *,
};

use frame::testing_prelude::*;

#[test]
fn test_get_authors() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Case 1: No authors in storage
        let authors = CustomPallet::get_authors(0, 2).unwrap();
        assert_eq!(authors.len(), 0);

        // Case 2: Insert authors into storage
        Authors::<Test>::insert(
            0,
            AuthorDetails {
                nickname: vec![0].try_into().unwrap(),
                real_name: Some(vec![1].try_into().unwrap()),
                owner: 0,
            },
        );
        Authors::<Test>::insert(
            1,
            AuthorDetails {
                nickname: vec![2].try_into().unwrap(),
                real_name: Some(vec![3].try_into().unwrap()),
                owner: 1,
            },
        );

        // Call get_authors with a valid range
        let authors = CustomPallet::get_authors(0, 2).unwrap();
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].0, 0);
        assert_eq!(authors[0].1.nickname.to_vec(), vec![0]);
        assert_eq!(authors[0].1.real_name.clone().unwrap().to_vec(), vec![1]);
        assert_eq!(authors[0].1.owner, 0);

        assert_eq!(authors[1].0, 1);
        assert_eq!(authors[1].1.nickname.to_vec(), vec![2]);
        assert_eq!(authors[1].1.real_name.clone().unwrap().to_vec(), vec![3]);
        assert_eq!(authors[1].1.owner, 1);

        // Case 3: Invalid range (from > to)
        assert!(CustomPallet::get_authors(2, 0).is_err());

        // Case 4: Partial range
        let authors = CustomPallet::get_authors(1, 2).unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].0, 1);
        assert_eq!(authors[0].1.nickname.to_vec(), vec![2]);
        assert_eq!(authors[0].1.real_name.clone().unwrap().to_vec(), vec![3]);
        assert_eq!(authors[0].1.owner, 1);
    });
}

#[test]
fn test_add_new_author() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        let caller: _ = 0;

        // Successfully add a new author
        let nickname: BoundedVec<u8, MaxShortStringLength> = vec![1, 2, 3].try_into().unwrap();
        let real_name: Option<BoundedVec<u8, MaxLongStringLength>> =
            Some(vec![4, 5, 6].try_into().unwrap());

        assert!(CustomPallet::add_new_author(
            caller.clone(),
            nickname.clone(),
            real_name.clone(),
            Some(caller.clone())
        )
        .is_ok());

        // Verify the author is added
        let author_details = Authors::<Test>::get(0).unwrap();
        assert_eq!(author_details.nickname.to_vec(), vec![1, 2, 3]);
        assert_eq!(
            author_details.real_name.clone().unwrap().to_vec(),
            vec![4, 5, 6]
        );
        assert_eq!(author_details.owner, 0);
    });
}

#[test]
fn test_set_author() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        add_author_for_test(0, 0);

        // Case 1: Successfully update the author's real name and owner
        let new_real_name: Option<BoundedVec<u8, MaxLongStringLength>> =
            Some(vec![7, 8, 9].try_into().unwrap());
        let new_owner: Option<_> = Some(1);

        assert_ok!(CustomPallet::set_author(
            0,
            0,
            new_real_name.clone(),
            new_owner
        ));

        // Verify the updates
        let updated_author = Authors::<Test>::get(0).unwrap();
        assert_eq!(
            updated_author.real_name.clone().unwrap().to_vec(),
            vec![7, 8, 9]
        );
        assert_eq!(updated_author.owner, 1);

        // Case 2: Attempt to update a non-existent author
        assert_err!(
            CustomPallet::set_author(0, 1, new_real_name.clone(), new_owner),
            Error::<Test, _>::AuthorNotFound,
        );

        // Case 3: No changes provided (real_name and owner are None)
        assert_ok!(CustomPallet::set_author(1, 0, None, None));

        // Verify no changes were made
        let unchanged_author = Authors::<Test>::get(0).unwrap();
        assert_eq!(
            unchanged_author.real_name.clone().unwrap().to_vec(),
            vec![7, 8, 9]
        );
        assert_eq!(unchanged_author.owner, 1);

        assert_err!(
            CustomPallet::set_author(2, 0, new_real_name.clone(), new_owner),
            Error::<Test, _>::NoPermission,
        )
    });
}

#[test]
fn test_add_new_authority() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        NftsSupport::<Test>::set(Some(true));

        // Case 1: Successfully add a new authority
        let name: BoundedVec<u8, MaxShortStringLength> = vec![1, 2, 3].try_into().unwrap();
        let collection_config = Some(0);

        assert_ok!(CustomPallet::add_new_authority(
            0,
            name.clone(),
            AuthorityKind::Label,
            collection_config
        ));

        // Verify the authority is added
        let authority_details = Authorities::<Test>::get(0).unwrap();
        assert_eq!(authority_details.name.to_vec(), vec![1, 2, 3]);
        assert_eq!(authority_details.authority_kind, AuthorityKind::Label);
        assert_eq!(authority_details.collection_id, Some(0));
    });
}

#[test]
fn test_set_authority() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        NftsSupport::<Test>::set(Some(true));

        add_authority_access_for_test(0, 0, None);

        // Case 1: Successfully update the authority's name, owner, and kind
        let new_name: Option<BoundedVec<u8, MaxShortStringLength>> =
            Some(vec![4, 5, 6].try_into().unwrap());
        let collection_config = Some(0u8);

        assert_ok!(CustomPallet::set_authority(
            0,
            0,
            new_name.clone(),
            Some(AuthorityKind::Musician),
            collection_config,
        ));

        // Verify the updates
        let updated_authority = Authorities::<Test>::get(0).unwrap();
        assert_eq!(updated_authority.name.to_vec(), vec![4, 5, 6]);
        assert_eq!(updated_authority.authority_kind, AuthorityKind::Musician);
        assert_eq!(updated_authority.collection_id, Some(0));

        // Case 2: Attempt to update a non-existent authority
        assert_err!(
            CustomPallet::set_authority(
                0,
                1,
                new_name.clone(),
                Some(AuthorityKind::Musician),
                None
            ),
            Error::<Test, _>::AuthoritiesAccessNotFound,
        );

        add_authority_access_for_test(1, 1, None);

        assert_err!(
            CustomPallet::set_authority(
                0,
                1,
                new_name.clone(),
                Some(AuthorityKind::Musician),
                None
            ),
            Error::<Test, _>::AuthoritiesAccessNotFound,
        );

        // test AuthorityNftCollectionIdAlreadyExist error
        assert_err!(
            CustomPallet::set_authority(
                0,
                0,
                new_name.clone(),
                Some(AuthorityKind::Musician),
                Some(1),
            ),
            Error::<Test, _>::AuthorityNftCollectionIdAlreadyExist,
        );

        // Case 3: No changes provided (name, owner, and kind are None)
        assert_ok!(CustomPallet::set_authority(0, 0, None, None, None));

        // Verify no changes were made
        let unchanged_authority = Authorities::<Test>::get(0).unwrap();
        assert_eq!(unchanged_authority.name.to_vec(), vec![4, 5, 6]);
        assert_eq!(unchanged_authority.authority_kind, AuthorityKind::Musician);
        assert_eq!(unchanged_authority.collection_id, Some(0));
    });
}

#[test]
fn test_add_new_entity() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        add_authority_access_for_test(0, 0, Some(0));
        add_author_for_test(0, 0);

        // Case 1: Successfully add a new entity
        let entity_kind = IPEntityKind::Track;
        let owner: u32 = 0;
        let url: BoundedVec<u8, MaxLongStringLength> = vec![4, 5, 6].try_into().unwrap();

        assert_ok!(CustomPallet::add_new_entity(
            0,
            entity_kind,
            owner,
            url.clone(),
            MetadataStandard::M25,
            MetadataFeatures::default(),
            Some(vec![0].try_into().unwrap()),
            None,
            None,
            Some(0u32),
            Some(0),
            None,
        ));

        // Verify the entity is added
        let entity_details = Entities::<Test>::get(0).unwrap();
        assert_eq!(entity_details.entity_kind, IPEntityKind::Track);
        assert_eq!(entity_details.owner, 0);
        assert_eq!(entity_details.metadata.url.to_vec(), vec![4, 5, 6]);
        assert_eq!(entity_details.collection_id, Some(0));
        assert_eq!(entity_details.item_id, Some(0));
    });
}

#[test]
fn test_set_entity() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        NftsSupport::<Test>::set(Some(true));

        add_authority_access_for_test(0, 0, Some(0));
        add_authority_access_for_test(1, 1, Some(1));

        // Insert an entity into storage
        Entities::<Test>::insert(
            0,
            EntityDetails {
                entity_kind: IPEntityKind::Track,
                owner: 0,
                authors: None,
                royalty_parts: None,
                related_to: None,
                metadata: Metadata {
                    url: vec![4, 5, 6].try_into().unwrap(),
                    standard: MetadataStandard::M25,
                    features: Default::default(),
                },
                collection_id: None,
                item_id: None,
            },
        );

        // Insert authors into storage
        Authors::<Test>::insert(
            0,
            AuthorDetails {
                nickname: vec![1].try_into().unwrap(),
                real_name: Some(vec![2].try_into().unwrap()),
                owner: 0,
            },
        );

        // Case 1: Successfully update the entity's metadata, owner, and authors
        let new_url: Option<BoundedVec<u8, MaxLongStringLength>> =
            Some(vec![7, 8, 9].try_into().unwrap());
        let new_owner: Option<u32> = Some(1);

        assert_ok!(CustomPallet::set_entity(
            0,
            0,
            new_url.clone(),
            Some(MetadataStandard::M25),
            None,
            new_owner,
            Some(vec![0].try_into().unwrap()),
            None,
            None,
            Some(1u32),
            Some(0),
            None,
        ));

        // Verify the updates
        let updated_entity = Entities::<Test>::get(0).unwrap();
        assert_eq!(updated_entity.metadata.url.to_vec(), vec![7, 8, 9]);
        assert_eq!(updated_entity.owner, 1);
        assert_eq!(updated_entity.authors.unwrap(), vec![0]);
        assert_eq!(updated_entity.collection_id, Some(1));
        assert_eq!(updated_entity.item_id, Some(1));

        // Case 2: Attempt to update a non-existent entity
        assert_err!(
            CustomPallet::set_entity(
                0,
                1,
                new_url.clone(),
                Some(MetadataStandard::M25),
                None,
                new_owner,
                Some(vec![0].try_into().unwrap()),
                None,
                None,
                None,
                None,
                None,
            ),
            Error::<Test, _>::EntityNotFound
        );

        // Case 3: Attempt to set authors that do not exist
        let invalid_authors: Option<BoundedVec<u32, MaxEntityAuthors>> =
            Some(vec![999].try_into().unwrap());
        assert_err!(
            CustomPallet::set_entity(
                1,
                0,
                None,
                Some(MetadataStandard::M25),
                None,
                None,
                invalid_authors,
                None,
                None,
                None,
                None,
                None,
            ),
            Error::<Test, _>::EntityAuthorNotFound
        );

        assert_err!(
            CustomPallet::set_entity(
                1,
                0,
                None,
                Some(MetadataStandard::M25),
                None,
                None,
                None,
                None,
                None,
                Some(5),
                Some(0),
                None,
            ),
            Error::<Test, _>::EntityNftImmutable
        );

        // Case 4: No changes provided
        assert_ok!(CustomPallet::set_entity(
            1,
            0,
            None,
            Some(MetadataStandard::M25),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        // Verify no changes were made
        let unchanged_entity = Entities::<Test>::get(0).unwrap();
        assert_eq!(unchanged_entity.metadata.url.to_vec(), vec![7, 8, 9]);
        assert_eq!(unchanged_entity.owner, 1);
        assert_eq!(unchanged_entity.authors.unwrap(), vec![0]);
        assert_eq!(unchanged_entity.collection_id, Some(1));
        assert_eq!(unchanged_entity.item_id, Some(1));
    });
}

#[test]
fn test_set_entity_nft_owner_must_be_specified() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        NftsSupport::<Test>::set(Some(true));

        add_authority_access_for_test(0, 0, Some(0));
        add_authority_access_for_test(1, 1, Some(1));

        // Insert an entity into storage
        Entities::<Test>::insert(
            0,
            EntityDetails {
                entity_kind: IPEntityKind::Track,
                owner: 0,
                authors: None,
                royalty_parts: None,
                related_to: None,
                metadata: Metadata {
                    url: vec![4, 5, 6].try_into().unwrap(),
                    standard: MetadataStandard::M25,
                    features: Default::default(),
                },
                collection_id: None,
                item_id: None,
            },
        );

        // Insert authors into storage
        Authors::<Test>::insert(
            0,
            AuthorDetails {
                nickname: vec![1].try_into().unwrap(),
                real_name: Some(vec![2].try_into().unwrap()),
                owner: 0,
            },
        );

        assert_err!(
            CustomPallet::set_entity(
                0,
                0,
                None,
                Some(MetadataStandard::M25),
                None,
                None,
                None,
                None,
                None,
                Some(5),
                None,
                None,
            ),
            Error::<Test, _>::EntityNftOwnerMustBeSpecified
        );
    });
}

#[test]
fn test_add_first_access() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let authority_id: u32 = 0;
        let account_id: _ = 0;

        assert_ok!(CustomPallet::add_first_access(authority_id, account_id));

        let access_settings = AuthoritiesAccess::<Test>::get(authority_id, account_id).unwrap();
        assert!(access_settings.has_access(AuthorityAccessSetting::EditAccess.into()));
    });
}

#[test]
fn test_add_access() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let origin: _ = 0;
        let authority_id: _ = 1;
        let account_id: _ = 2;
        let access = AuthorityAccessSettings::all();

        add_authority_access_for_test(origin, authority_id, None);

        assert_ok!(CustomPallet::add_access(
            origin,
            authority_id,
            account_id,
            access
        ));

        let access_settings = AuthoritiesAccess::<Test>::get(authority_id, account_id).unwrap();
        assert!(access_settings.has_access(AuthorityAccessSetting::EditAccess.into()));
    });
}

#[test]
fn test_set_access() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let account_id: _ = 0;
        let authority_id: _ = 1;
        let new_access = AuthorityAccessSettings::none();

        add_authority_access_for_test(account_id, authority_id, None);
        AuthoritiesAccess::<Test>::insert(authority_id, account_id, AuthorityAccessSettings::all());

        assert_ok!(CustomPallet::set_access(
            account_id,
            authority_id,
            account_id,
            new_access
        ));

        let updated_access = AuthoritiesAccess::<Test>::get(authority_id, account_id).unwrap();
        assert_eq!(updated_access, new_access);

        assert_err!(
            CustomPallet::set_access(account_id, 2, account_id, new_access),
            Error::<Test, _>::AuthoritiesAccessNotFound
        );
    });
}

#[test]
fn test_ensure_access_right() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let authority_id: _ = 0;
        let account_id: _ = 1;

        add_authority_access_for_test(account_id, authority_id, None);

        assert_ok!(CustomPallet::ensure_access_right(
            &account_id,
            &authority_id,
            AuthorityAccessSetting::EditAccess.into()
        ));

        let invalid_account_id: _ = 2;
        assert_err!(
            CustomPallet::ensure_access_right(
                &invalid_account_id,
                &authority_id,
                AuthorityAccessSetting::EditAccess.into()
            ),
            Error::<Test, _>::AuthoritiesAccessNotFound
        );
    });
}

#[test]
fn test_toggle_nfts_support() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        NftsSupport::<Test>::set(None);

        assert_ok!(CustomPallet::toggle_nfts_support());
        assert_eq!(NftsSupport::<Test>::get(), Some(true));

        assert_ok!(CustomPallet::toggle_nfts_support());
        assert_eq!(NftsSupport::<Test>::get(), Some(false));
    });
}

fn add_authority_access_for_test(
    account_id: <Test as frame_system::Config>::AccountId,
    authority_id: <Test as Config>::AuthorityId,
    collection_id: Option<<Test as Config>::CollectionId>,
) {
    Authorities::<Test>::insert(
        authority_id,
        AuthorityDetails {
            authority_kind: AuthorityKind::Label,
            name: vec![1, 2, 3].try_into().unwrap(),
            collection_id,
        },
    );

    AuthoritiesAccess::<Test>::insert(authority_id, account_id, AuthorityAccessSettings::all());
}

fn add_author_for_test(
    owner: <Test as frame_system::Config>::AccountId,
    author_id: <Test as Config>::AuthorId,
) {
    Authors::<Test>::insert(
        author_id,
        AuthorDetails {
            nickname: vec![0].try_into().unwrap(),
            real_name: Some(vec![1].try_into().unwrap()),
            owner,
        },
    );
}
