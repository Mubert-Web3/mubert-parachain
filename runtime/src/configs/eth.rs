mod precompiles;

use polkadot_sdk::*;

use frame_support::{
    parameter_types,
    traits::{ConstU32, FindAuthor},
    weights::Weight,
};

use core::marker::PhantomData;

use sp_core::{crypto::ByteArray, H160, U256};
use sp_runtime::{ConsensusEngineId, Permill};

// EVM
use fp_evm::weight_per_gas;
use pallet_ethereum::PostLogContent;
use pallet_evm::EnsureAddressTruncated;
use precompiles::FrontierPrecompiles;

// Local module imports
use super::{
    AccountId, Aura, Balances, BaseFee, EVMChainId, Runtime, RuntimeEvent, Timestamp,
    NORMAL_DISPATCH_RATIO,
};
impl pallet_evm_chain_id::Config for Runtime {}

pub struct FindAuthorTruncated<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        if let Some(author_index) = F::find_author(digests) {
            let authority_id =
                pallet_aura::Authorities::<Runtime>::get()[author_index as usize].clone();
            return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
        }
        None
    }
}

parameter_types! {
    pub BoundDivision: U256 = U256::from(1024);
}

impl pallet_dynamic_fee::Config for Runtime {
    type MinGasPriceBoundDivisor = BoundDivision;
}

parameter_types! {
    pub DefaultBaseFeePerGas: U256 = U256::from(1_000_000_000);
    pub DefaultElasticity: Permill = Permill::from_parts(125_000);
}
pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
    fn lower() -> Permill {
        Permill::zero()
    }
    fn ideal() -> Permill {
        Permill::from_parts(500_000)
    }
    fn upper() -> Permill {
        Permill::from_parts(1_000_000)
    }
}
impl pallet_base_fee::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Threshold = BaseFeeThreshold;
    type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
    type DefaultElasticity = DefaultElasticity;
}

const BLOCK_GAS_LIMIT: u64 = 75_000_000;
const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// The maximum storage growth per block in bytes.
const MAX_STORAGE_GROWTH: u64 = 400 * 1024;
pub const WEIGHT_MILLISECS_PER_BLOCK: u64 = 2000;

parameter_types! {
    pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
    pub const GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(MAX_POV_SIZE);
    pub const GasLimitStorageGrowthRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(MAX_STORAGE_GROWTH);
    pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::new();
    pub WeightPerGas: Weight = Weight::from_parts(weight_per_gas(BLOCK_GAS_LIMIT, NORMAL_DISPATCH_RATIO, WEIGHT_MILLISECS_PER_BLOCK), 0);
}

pub struct HashedAddressMapping;

impl pallet_evm::AddressMapping<AccountId> for HashedAddressMapping {
    fn into_account_id(address: H160) -> AccountId {
        let mut data = [0u8; 32];
        data[0..20].copy_from_slice(address.as_bytes());
        data.into()
    }
}

impl pallet_evm::Config for Runtime {
    type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
    type FeeCalculator = BaseFee;
    type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
    type WeightPerGas = WeightPerGas;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressTruncated;
    type WithdrawOrigin = EnsureAddressTruncated;
    type AddressMapping = HashedAddressMapping;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type PrecompilesType = FrontierPrecompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = EVMChainId;
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type OnChargeTransaction = ();
    type OnCreate = ();
    type FindAuthor = FindAuthorTruncated<Aura>;
    type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
    type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
    type Timestamp = Timestamp;
    type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

parameter_types! {
    pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}
impl pallet_ethereum::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self::Version>;
    type PostLogContent = PostBlockAndTxnHashes;
    type ExtraDataLength = ConstU32<30>;
}
