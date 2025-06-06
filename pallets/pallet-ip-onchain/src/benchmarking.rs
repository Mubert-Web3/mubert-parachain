//! Benchmarking setup for pallet-ip-onchain
#![cfg(feature = "runtime-benchmarks")]
use super::*;

extern crate alloc;
use alloc::vec;

use polkadot_sdk::*;

use frame_benchmarking::v2::*;

#[benchmarks]
mod benchmarks {
    use super::*;
    #[cfg(test)]
    use crate::pallet::Pallet as IpOnchain;
    use frame_system::RawOrigin;

    #[benchmark]
    fn create_author() {
        let author_id: T::AuthorId = T::AuthorId::initial_value().unwrap();
        let caller: T::AccountId = whitelisted_caller();
        let nickname: BoundedVec<u8, T::MaxShortStringLength> = vec![1, 2, 3].try_into().unwrap();
        let real_name: Option<BoundedVec<u8, T::MaxLongStringLength>> =
            Some(vec![4, 5, 6].try_into().unwrap());

        #[extrinsic_call]
        create_author(RawOrigin::Signed(caller), nickname, real_name, None);

        assert!(Authors::<T>::contains_key(author_id));
    }

    #[benchmark]
    fn edit_author() {
        let caller: T::AccountId = whitelisted_caller();
        let author_id: T::AuthorId = T::AuthorId::initial_value().unwrap();

        Authors::<T>::insert(
            author_id,
            AuthorDetails {
                nickname: vec![0].try_into().unwrap(),
                real_name: Some(vec![1].try_into().unwrap()),
                owner: caller.clone(),
            },
        );

        let new_real_name: Option<BoundedVec<u8, T::MaxLongStringLength>> =
            Some(vec![7, 8, 9].try_into().unwrap());

        #[extrinsic_call]
        edit_author(
            RawOrigin::Signed(caller),
            author_id,
            new_real_name,
            Some(caller.clone()),
        );

        let updated_author = Authors::<T>::get(author_id).unwrap();
        assert_eq!(updated_author.real_name.unwrap().to_vec(), vec![7, 8, 9]);
    }

    #[benchmark]
    fn create_authority() {
        let caller: T::AccountId = whitelisted_caller();
        let name: BoundedVec<u8, T::MaxShortStringLength> = vec![1, 2, 3].try_into().unwrap();

        #[extrinsic_call]
        create_authority(RawOrigin::Signed(caller), name, AuthorityKind::Label);

        let authority_id = T::AuthorityId::initial_value().unwrap();
        assert!(Authorities::<T>::contains_key(authority_id));
    }

    #[benchmark]
    fn edit_authority() {
        let caller: T::AccountId = whitelisted_caller();
        let authority_id: T::AuthorityId = T::AuthorityId::initial_value().unwrap();

        Authorities::<T>::insert(
            authority_id,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![0].try_into().unwrap(),
            },
        );

        AuthoritiesAccess::<T>::insert(
            authority_id,
            caller.clone(),
            AuthorityAccessSettings::all(),
        );

        let new_name: Option<BoundedVec<u8, T::MaxShortStringLength>> =
            Some(vec![4, 5, 6].try_into().unwrap());

        #[extrinsic_call]
        edit_authority(
            RawOrigin::Signed(caller),
            authority_id,
            new_name,
            Some(AuthorityKind::Musician),
        );

        let updated_authority = Authorities::<T>::get(authority_id).unwrap();
        assert_eq!(updated_authority.name.to_vec(), vec![4, 5, 6]);
    }

    #[benchmark]
    fn create_entity() {
        let caller: T::AccountId = whitelisted_caller();
        let authority_id: T::AuthorityId = T::AuthorityId::initial_value().unwrap();
        let url: BoundedVec<u8, T::MaxLongStringLength> = vec![4, 5, 6].try_into().unwrap();

        Authorities::<T>::insert(
            authority_id,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![0].try_into().unwrap(),
            },
        );

        AuthoritiesAccess::<T>::insert(
            authority_id,
            caller.clone(),
            AuthorityAccessSettings::all(),
        );

        #[extrinsic_call]
        create_entity(
            RawOrigin::Signed(caller),
            IPEntityKind::Track,
            authority_id,
            url,
            MetadataStandard::M25,
            MetadataFeatures::default(),
            None,
            None,
            None,
        );

        let entity_id = T::EntityId::initial_value().unwrap();
        assert!(Entities::<T>::contains_key(entity_id));
    }

    #[benchmark]
    fn edit_entity() {
        let caller: T::AccountId = whitelisted_caller();
        let entity_id: T::EntityId = T::EntityId::initial_value().unwrap();
        let authority_id: T::AuthorityId = T::AuthorityId::initial_value().unwrap();

        AuthoritiesAccess::<T>::insert(
            authority_id,
            caller.clone(),
            AuthorityAccessSettings::all(),
        );

        Entities::<T>::insert(
            entity_id,
            EntityDetails {
                entity_kind: IPEntityKind::Track,
                owner: authority_id.clone(),
                authors: None,
                royalty_parts: None,
                related_to: None,
                metadata: Metadata {
                    url: vec![4, 5, 6].try_into().unwrap(),
                    standard: MetadataStandard::M25,
                    features: Default::default(),
                },
            },
        );

        Authorities::<T>::insert(
            authority_id.clone(),
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![0].try_into().unwrap(),
            },
        );

        let new_url: Option<BoundedVec<u8, T::MaxLongStringLength>> =
            Some(vec![7, 8, 9].try_into().unwrap());

        #[extrinsic_call]
        edit_entity(
            RawOrigin::Signed(caller),
            entity_id,
            new_url,
            Some(MetadataStandard::M25),
            None,
            None,
            None,
            None,
            None,
        );

        let updated_entity = Entities::<T>::get(entity_id).unwrap();
        assert_eq!(updated_entity.metadata.url.to_vec(), vec![7, 8, 9]);
    }

    #[benchmark]
    fn create_account_access() {
        let caller: T::AccountId = whitelisted_caller();
        let authority_id: T::AuthorityId = T::AuthorityId::initial_value().unwrap();
        let account_id: T::AccountId = account("other", 1, 1);
        let access: AuthorityAccessSettings = AuthorityAccessSettings::all();

        Authorities::<T>::insert(
            authority_id,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![0].try_into().unwrap(),
            },
        );

        AuthoritiesAccess::<T>::insert(
            authority_id,
            caller.clone(),
            AuthorityAccessSettings::all(),
        );

        #[extrinsic_call]
        create_account_access(
            RawOrigin::Signed(caller),
            authority_id,
            account_id.clone(),
            access,
        );

        assert!(AuthoritiesAccess::<T>::contains_key(
            authority_id,
            account_id
        ));
    }

    #[benchmark]
    fn edit_account_access() {
        let caller: T::AccountId = whitelisted_caller();
        let authority_id: T::AuthorityId = T::AuthorityId::initial_value().unwrap();
        let account_id: T::AccountId = whitelisted_caller();
        let new_access: AuthorityAccessSettings = AuthorityAccessSettings::none();

        Authorities::<T>::insert(
            authority_id,
            AuthorityDetails {
                authority_kind: AuthorityKind::Label,
                name: vec![0].try_into().unwrap(),
            },
        );

        AuthoritiesAccess::<T>::insert(
            authority_id,
            account_id.clone(),
            AuthorityAccessSettings::all(),
        );

        #[extrinsic_call]
        edit_account_access(
            RawOrigin::Signed(caller),
            authority_id,
            account_id.clone(),
            new_access,
        );

        let updated_access = AuthoritiesAccess::<T>::get(authority_id, account_id).unwrap();
        assert_eq!(updated_access, new_access);
    }

    impl_benchmark_test_suite!(IpOnchain, mock::new_test_ext(), mock::Test);
}
