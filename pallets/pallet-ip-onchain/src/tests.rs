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

        // Successfully add a new author
        let nickname: BoundedVec<u8, MaxShortStringLength> = vec![1, 2, 3].try_into().unwrap();
        let real_name: Option<BoundedVec<u8, MaxLongStringLength>> =
            Some(vec![4, 5, 6].try_into().unwrap());
        let owner: u32 = 0;

        assert!(CustomPallet::add_new_author(nickname.clone(), real_name.clone(), owner).is_ok());

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

        // Insert an author into storage
        Authorities::<Test>::insert(
            0,
            AuthorityDetails {
                authority_kind: crate::AuthorityKind::Label,
                name: vec![0].try_into().unwrap(),
                owner: 0,
            },
        );

        Authorities::<Test>::insert(
            1,
            AuthorityDetails {
                authority_kind: crate::AuthorityKind::Label,
                name: vec![0].try_into().unwrap(),
                owner: 1,
            },
        );

        // Insert an author into storage
        Authors::<Test>::insert(
            0,
            AuthorDetails {
                nickname: vec![0].try_into().unwrap(),
                real_name: Some(vec![1].try_into().unwrap()),
                owner: 0,
            },
        );

        // Case 1: Successfully update the author's real name and owner
        let new_real_name: Option<BoundedVec<u8, MaxLongStringLength>> =
            Some(vec![7, 8, 9].try_into().unwrap());
        let new_owner: Option<u32> = Some(1);

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
    });
}

#[test]
fn test_add_new_authority() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Case 1: Successfully add a new authority
        let name: BoundedVec<u8, MaxShortStringLength> = vec![1, 2, 3].try_into().unwrap();
        let owner = 0;

        assert_ok!(CustomPallet::add_new_authority(
            name.clone(),
            owner,
            AuthorityKind::Label
        ));

        // Verify the authority is added
        let authority_details = Authorities::<Test>::get(0).unwrap();
        assert_eq!(authority_details.name.to_vec(), vec![1, 2, 3]);
        assert_eq!(authority_details.owner, 0);
        assert_eq!(authority_details.authority_kind, AuthorityKind::Label);
    });
}

#[test]
fn test_set_authority() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Insert an authority into storage
        Authorities::<Test>::insert(
            0,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![1, 2, 3].try_into().unwrap(),
                owner: 0,
            },
        );

        // Case 1: Successfully update the authority's name, owner, and kind
        let new_name: Option<BoundedVec<u8, MaxShortStringLength>> =
            Some(vec![4, 5, 6].try_into().unwrap());
        let new_owner: Option<_> = Some(1);

        assert_ok!(CustomPallet::set_authority(
            0,
            0,
            new_name.clone(),
            new_owner,
            Some(AuthorityKind::Musician)
        ));

        // Verify the updates
        let updated_authority = Authorities::<Test>::get(0).unwrap();
        assert_eq!(updated_authority.name.to_vec(), vec![4, 5, 6]);
        assert_eq!(updated_authority.owner, 1);
        assert_eq!(
            updated_authority.authority_kind,
            AuthorityKind::Musician
        );

        // Case 2: Attempt to update a non-existent authority
        assert_err!(
            CustomPallet::set_authority(
                0,
                1,
                new_name.clone(),
                new_owner,
                Some(AuthorityKind::Musician)
            ),
            Error::<Test, _>::AuthorityNotFound,
        );

        // Case 3: No changes provided (name, owner, and kind are None)
        assert_ok!(CustomPallet::set_authority(1, 0, None, None, None));

        // Verify no changes were made
        let unchanged_authority = Authorities::<Test>::get(0).unwrap();
        assert_eq!(unchanged_authority.name.to_vec(), vec![4, 5, 6]);
        assert_eq!(unchanged_authority.owner, 1);
        assert_eq!(
            unchanged_authority.authority_kind,
            AuthorityKind::Musician
        );
    });
}

#[test]
fn test_ensure_authority_owner() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Insert an authority into storage
        Authorities::<Test>::insert(
            0,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![1, 2, 3].try_into().unwrap(),
                owner: 0,
            },
        );

        // Case 1: Successfully verify the owner of the authority
        assert_ok!(CustomPallet::ensure_authority_owner(&0, &0));

        // Case 2: Attempt to verify ownership with a non-owner account
        assert_err!(
            CustomPallet::ensure_authority_owner(&1, &0),
            Error::<Test, _>::NoPermission
        );

        // Case 3: Attempt to verify ownership of a non-existent authority
        assert_err!(
            CustomPallet::ensure_authority_owner(&0, &1),
            Error::<Test, _>::AuthorityNotFound
        );
    });
}

#[test]
fn test_add_new_entity() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Insert an authority into storage to act as the owner
        Authorities::<Test>::insert(
            0,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![1, 2, 3].try_into().unwrap(),
                owner: 0,
            },
        );

        // Case 1: Successfully add a new entity
        let entity_kind = IPEntityKind::Track;
        let owner: u32 = 0;
        let url: BoundedVec<u8, MaxLongStringLength> = vec![4, 5, 6].try_into().unwrap();

        assert_ok!(CustomPallet::add_new_entity(
            entity_kind,
            owner,
            url.clone(),
            MetadataStandard::M25,
        ));

        // Verify the entity is added
        let entity_details = Entities::<Test>::get(0).unwrap();
        assert_eq!(entity_details.entity_kind, IPEntityKind::Track);
        assert_eq!(entity_details.owner, 0);
        assert_eq!(entity_details.metadata.unwrap().url.to_vec(), vec![4, 5, 6]);
    });
}

#[test]
fn test_set_entity() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // Insert an authority into storage to act as the owner
        Authorities::<Test>::insert(
            0,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![1, 2, 3].try_into().unwrap(),
                owner: 0,
            },
        );
        Authorities::<Test>::insert(
            1,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![1, 2, 3].try_into().unwrap(),
                owner: 1,
            },
        );

        // Insert an entity into storage
        Entities::<Test>::insert(
            0,
            EntityDetails {
                entity_kind: IPEntityKind::Track,
                owner: 0,
                authors: None,
                royalty_parts: None,
                related_to: None,
                metadata: Some(Metadata {
                    url: vec![4, 5, 6].try_into().unwrap(),
                    standard: MetadataStandard::M25,
                }),
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
            MetadataStandard::M25,
            new_owner,
            Some(vec![0].try_into().unwrap()),
            None,
            None
        ));

        // Verify the updates
        let updated_entity = Entities::<Test>::get(0).unwrap();
        assert_eq!(updated_entity.metadata.unwrap().url.to_vec(), vec![7, 8, 9]);
        assert_eq!(updated_entity.owner, 1);
        assert_eq!(updated_entity.authors.unwrap(), vec![0]);

        // Case 2: Attempt to update a non-existent entity
        assert_err!(
            CustomPallet::set_entity(
                0,
                1,
                new_url.clone(),
                MetadataStandard::M25,
                new_owner,
                Some(vec![0].try_into().unwrap()),
                None,
                None
            ),
            Error::<Test, _>::EntityNotFound
        );

        // Case 3: Attempt to set authors that do not exist
        let invalid_authors: Option<BoundedVec<u32, MaxEntityAuthors>> =
            Some(vec![999].try_into().unwrap());
        assert_err!(
            CustomPallet::set_entity(1, 0, None, MetadataStandard::M25, None, invalid_authors, None, None),
            Error::<Test, _>::EntityAuthorNotFound
        );

        // Case 4: No changes provided
        assert_ok!(CustomPallet::set_entity(1, 0, None, MetadataStandard::M25, None, None, None, None));

        // Verify no changes were made
        let unchanged_entity = Entities::<Test>::get(0).unwrap();
        assert_eq!(
            unchanged_entity.metadata.unwrap().url.to_vec(),
            vec![7, 8, 9]
        );
        assert_eq!(unchanged_entity.owner, 1);
        assert_eq!(unchanged_entity.authors.unwrap(), vec![0]);
    });
}
