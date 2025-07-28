use polkadot_sdk::pallet_nfts::ItemConfig;
use polkadot_sdk::polkadot_sdk_frame as frame;

use frame::{
    deps::frame_system::GenesisConfig,
    prelude::*,
    runtime::prelude::*,
    testing_prelude::*,
    traits::tokens::nonfungibles_v2::{Create, Inspect, Mutate},
};

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_construct_runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;

    #[runtime::pallet_index(1)]
    pub type CustomPallet = crate;
}

// System pallet configuration
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

// Custom pallet configuration
parameter_types! {
    pub const MaxShortStringLength: u32 = 32;
    pub const MaxLongStringLength: u32 = 128;
    pub const MaxEntityAuthors: u32 = 10;
    pub const MaxRoyaltyParts: u32 = 10;
    pub const MaxRelatedEntities: u32 = 10;
    pub const MaxArrayLen: u32 = 10;
}

pub struct TestWhiteListChecker;

impl<AccountId32> Contains<AccountId32> for TestWhiteListChecker {
    fn contains(_account: &AccountId32) -> bool {
        true
    }
}

impl crate::Config for Test {
    type AuthorityId = u32;
    type AuthorId = u32;
    type EntityId = u32;
    type MaxShortStringLength = MaxShortStringLength;
    type MaxLongStringLength = MaxLongStringLength;
    type MaxEntityAuthors = MaxEntityAuthors;
    type MaxRoyaltyParts = MaxRoyaltyParts;
    type MaxRelatedEntities = MaxRelatedEntities;
    type MaxArrayLen = MaxArrayLen;
    type WhiteListChecker = TestWhiteListChecker;
    type CollectionId = u32;
    type ItemId = u32;
    type CollectionConfig = u8;
    type Nfts = NftsMock;
    type Currency = ();
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub struct NftsMock {}

impl Inspect<u64> for NftsMock {
    type ItemId = u32;
    type CollectionId = u32;

    fn owner(_collection: &Self::CollectionId, _item: &Self::ItemId) -> Option<u64> {
        Some(1)
    }
}

impl Mutate<u64, ItemConfig> for NftsMock {
    fn mint_into(
        _collection: &Self::CollectionId,
        _item: &Self::ItemId,
        _who: &u64,
        _config: &ItemConfig,
        _deposit_collection_owner: bool,
    ) -> DispatchResult {
        Ok(())
    }
}

impl Create<u64, u8> for NftsMock {
    fn create_collection(
        _who: &u64,
        _admin: &u64,
        config: &u8,
    ) -> Result<Self::CollectionId, DispatchError> {
        Ok(*config as u32)
    }

    fn create_collection_with_id(
        _collection: Self::CollectionId,
        _who: &u64,
        _admin: &u64,
        _config: &u8,
    ) -> Result<(), DispatchError> {
        Err(TokenError::Unsupported.into())
    }
}

// Test externalities initialization
pub fn new_test_ext() -> TestState {
    GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
