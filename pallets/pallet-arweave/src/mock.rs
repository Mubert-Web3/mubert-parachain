use polkadot_sdk::polkadot_sdk_frame as frame;
use polkadot_sdk::*;

use crate::*;

use frame::{
    deps::frame_system::GenesisConfig, prelude::*, runtime::prelude::*, testing_prelude::*,
};

use sp_core::{
    offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
    sr25519::Signature,
    H256,
};

use sp_runtime::{
    testing::TestXt,
    traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
    RuntimeAppPublic,
};

// Configure a mock runtime to test the pallet.
#[frame_construct_runtime]
mod test_runtime {
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
    pub type Balances = pallet_balances;
    #[runtime::pallet_index(2)]
    pub type CustomPallet = crate;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Nonce = u64;
    type AccountId = AccountId;
    type AccountData = pallet_balances::AccountData<u64>;
    type Lookup = IdentityLookup<Self::AccountId>;

    type Block = MockBlock<Test>;
    type BlockHashCount = ConstU64<250>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

type Extrinsic = TestXt<RuntimeCall, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::CreateTransactionBase<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    type Extrinsic = Extrinsic;

    type RuntimeCall = RuntimeCall;
}

impl<LocalCall> frame_system::offchain::CreateTransaction<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    type Extension = ();

    fn create_transaction(call: RuntimeCall, _extension: Self::Extension) -> Extrinsic {
        Extrinsic::new_transaction(call, ())
    }
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    fn create_signed_transaction<
        C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>,
    >(
        call: RuntimeCall,
        _public: <Signature as Verify>::Signer,
        _account: AccountId,
        nonce: u64,
    ) -> Option<Extrinsic> {
        Some(Extrinsic::new_signed(call, nonce, (), ()))
    }
}

impl<LocalCall> frame_system::offchain::CreateInherent<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    fn create_inherent(call: Self::RuntimeCall) -> Self::Extrinsic {
        Extrinsic::new_bare(call)
    }
}

parameter_types! {
    pub const MaxDataLength: u32 = 1024;
    pub const MaxTxHashLength: u32 = 1024;
    pub const MaxSignedDataLength: u32 = 1024;
}

impl crate::Config for Test {
    type AuthorityId = crypto::arweave::AuthId;
    type TaskId = u32;
    type Currency = Balances;
    type MaxDataLength = MaxDataLength;
    type MaxTxHashLength = MaxTxHashLength;
    type MaxSignedDataLength = MaxSignedDataLength;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> TestState {
    GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
