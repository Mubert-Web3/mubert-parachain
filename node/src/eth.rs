use futures::{future, prelude::*};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use polkadot_sdk::*;

// Substrate
use sc_client_api::backend::StateBackend;
use sc_client_api::{
    backend::{Backend, StorageProvider},
    client::BlockchainEvents,
};
use sc_network::service::traits::NetworkService;
use sc_network_sync::SyncingService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_service::TaskManager;
use sc_transaction_pool_api::TransactionPool;
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::H256;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};

// Frontier
use fc_db::kv::Backend as FrontierBackend;
use fc_rpc::{EthBlockDataCacheTask, EthTask};
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
use fc_storage::StorageOverride;
use fp_rpc::{ConvertTransaction, ConvertTransactionRuntimeApi, EthereumRuntimeRPCApi};

use crate::rpc::RpcExtension;
use crate::service::{ParachainBackend, ParachainClient};
use mubert_runtime::opaque::Block;

/// Extra dependencies for Ethereum compatibility.
pub struct EthDependencies<C, P, CT, CIDP> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Graph pool instance.
    pub graph: Arc<P>,
    /// Ethereum transaction converter.
    pub converter: Option<CT>,
    /// The Node authority flag
    pub is_authority: bool,
    /// Whether to enable dev signer
    pub enable_dev_signer: bool,
    /// Network service
    pub network: Arc<dyn NetworkService>,
    /// Chain syncing service
    pub sync: Arc<SyncingService<Block>>,
    /// Frontier Backend.
    pub frontier_backend: Arc<FrontierBackend<Block, C>>,
    /// Ethereum data access overrides.
    pub storage_override: Arc<dyn StorageOverride<Block>>,
    /// Cache for Ethereum block data.
    pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
    /// EthFilterApi pool.
    pub filter_pool: FilterPool,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
    /// Fee history cache.
    pub fee_history_cache: FeeHistoryCache,
    /// Maximum fee history cache size.
    pub fee_history_cache_limit: FeeHistoryCacheLimit,
    /// Maximum allowed gas limit will be ` block.gas_limit * execute_gas_limit_multiplier` when
    /// using eth_call/eth_estimateGas.
    pub execute_gas_limit_multiplier: u64,
    /// Mandated parent hashes for a given block hash.
    pub forced_parent_hashes: Option<BTreeMap<H256, H256>>,
    /// Something that can create the inherent data providers for pending state
    pub pending_create_inherent_data_providers: CIDP,
}
use sc_client_api::{AuxStore, UsageProvider};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
/// Instantiate Ethereum-compatible RPC extensions.
pub fn create<C, BE, P, CT, CIDP>(
    deps: EthDependencies<C, P, CT, CIDP>,
    subscription_task_executor: SubscriptionTaskExecutor,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<Block>,
        >,
    >,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C::Api: BlockBuilderApi<Block>
        + EthereumRuntimeRPCApi<Block>
        + ConvertTransactionRuntimeApi<Block>
        + sp_consensus_aura::AuraApi<Block, AuraId>,
    C: BlockchainEvents<Block> + AuxStore + UsageProvider<Block> + 'static,
    C: HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + StorageProvider<Block, BE>,
    C: CallApiAt<Block>,
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    P: TransactionPool<Block = Block, Hash = H256> + 'static,
    CT: ConvertTransaction<<Block as BlockT>::Extrinsic> + Send + Sync + 'static,
    CIDP: CreateInherentDataProviders<Block, ()> + Send + 'static,
{
    use fc_rpc::{
        pending::AuraConsensusDataProvider, Debug, DebugApiServer, Eth, EthApiServer, EthDevSigner,
        EthFilter, EthFilterApiServer, EthPubSub, EthPubSubApiServer, EthSigner, Net, NetApiServer,
        Web3, Web3ApiServer,
    };

    #[cfg(feature = "txpool")]
    use fc_rpc::{TxPool, TxPoolApiServer};

    let mut module = RpcExtension::new(());

    let EthDependencies {
        client,
        pool,
        graph,
        converter,
        is_authority,
        enable_dev_signer,
        network,
        sync,
        frontier_backend,
        storage_override,
        block_data_cache,
        filter_pool,
        max_past_logs,
        fee_history_cache,
        fee_history_cache_limit,
        execute_gas_limit_multiplier,
        forced_parent_hashes,
        pending_create_inherent_data_providers,
    } = deps;

    let mut signers = Vec::new();
    if enable_dev_signer {
        signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
    }

    module.merge(
        Eth::<Block, _, _, _, _, _, DefaultEthConfig<_, _>>::new(
            client.clone(),
            pool.clone(),
            graph.clone(),
            converter,
            sync.clone(),
            signers,
            storage_override.clone(),
            frontier_backend.clone(),
            is_authority,
            block_data_cache.clone(),
            fee_history_cache,
            fee_history_cache_limit,
            execute_gas_limit_multiplier,
            forced_parent_hashes,
            pending_create_inherent_data_providers,
            Some(Box::new(AuraConsensusDataProvider::new(client.clone()))),
        )
        .replace_config::<DefaultEthConfig<C, BE>>()
        .into_rpc(),
    )?;

    module.merge(
        EthFilter::new(
            client.clone(),
            frontier_backend.clone(),
            graph.clone(),
            filter_pool,
            500_usize, // max stored filters
            max_past_logs,
            block_data_cache.clone(),
        )
        .into_rpc(),
    )?;

    module.merge(
        EthPubSub::new(
            pool,
            client.clone(),
            sync,
            subscription_task_executor,
            storage_override.clone(),
            pubsub_notification_sinks,
        )
        .into_rpc(),
    )?;

    module.merge(
        Net::new(
            client.clone(),
            network,
            // Whether to format the `peer_count` response as Hex (default) or not.
            true,
        )
        .into_rpc(),
    )?;

    module.merge(Web3::new(client.clone()).into_rpc())?;

    module.merge(
        Debug::new(
            client.clone(),
            frontier_backend,
            storage_override,
            block_data_cache,
        )
        .into_rpc(),
    )?;

    #[cfg(feature = "txpool")]
    module.merge(TxPool::new(client, graph).into_rpc())?;

    Ok(module)
}

pub struct DefaultEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);

impl<C, BE> fc_rpc::EthConfig<Block, C> for DefaultEthConfig<C, BE>
where
    C: sc_client_api::StorageProvider<Block, BE> + Sync + Send + 'static,
    BE: Backend<Block> + 'static,
{
    type EstimateGasAdapter = ();
    type RuntimeStorageOverride =
        fc_rpc::frontier_backend_client::SystemAccountId20StorageOverride<Block, C, BE>;
}

pub async fn spawn_frontier_tasks(
    task_manager: &TaskManager,
    client: Arc<ParachainClient>,
    backend: Arc<ParachainBackend>,
    frontier_backend: Arc<FrontierBackend<Block, ParachainClient>>,
    filter_pool: FilterPool,
    storage_override: Arc<dyn StorageOverride<Block>>,
    fee_history_cache: FeeHistoryCache,
    fee_history_cache_limit: FeeHistoryCacheLimit,
    sync: Arc<SyncingService<Block>>,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<Block>,
        >,
    >,
) {
    // Spawn main mapping sync worker background task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-mapping-sync-worker",
        Some("frontier"),
        fc_mapping_sync::kv::MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::new(6, 0),
            client.clone(),
            backend,
            storage_override.clone(),
            frontier_backend.clone(),
            3,
            0u32.into(),
            fc_mapping_sync::SyncStrategy::Parachain,
            sync,
            pubsub_notification_sinks,
        )
        .for_each(|()| future::ready(())),
    );

    // Spawn Frontier EthFilterApi maintenance task.
    // Each filter is allowed to stay in the pool for 100 blocks.
    const FILTER_RETAIN_THRESHOLD: u64 = 100;
    task_manager.spawn_essential_handle().spawn(
        "frontier-filter-pool",
        Some("frontier"),
        EthTask::filter_pool_task(client.clone(), filter_pool, FILTER_RETAIN_THRESHOLD),
    );

    // Spawn Frontier FeeHistory cache maintenance task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("frontier"),
        EthTask::fee_history_task(
            client,
            storage_override,
            fee_history_cache,
            fee_history_cache_limit,
        ),
    );
}
