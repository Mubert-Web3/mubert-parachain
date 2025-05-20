//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
use polkadot_sdk::*;

use std::path::PathBuf;
use std::{sync::Arc, time::Duration};

use mubert_runtime::{
    apis::RuntimeApi,
    opaque::{Block, Hash},
    TransactionConverter,
};

// Cumulus Imports
use cumulus_client_cli::CollatorOptions;
use cumulus_client_collator::service::CollatorService;

#[docify::export(lookahead_collator)]
use cumulus_client_consensus_aura::collators::lookahead::{self as aura, Params as AuraParams};
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_service::{
    build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
    BuildNetworkParams, CollatorSybilResistance, DARecoveryProfile, ParachainHostFunctions,
    StartRelayChainTasksParams,
};

#[docify::export(cumulus_primitives)]
use cumulus_primitives_core::{
    relay_chain::{CollatorPair, ValidationCode},
    ParaId,
};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};

use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use prometheus_endpoint::Registry;
use sc_client_api::Backend;
use sc_consensus::ImportQueue;
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::NetworkBlock;
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_core::U256;
use sp_keystore::KeystorePtr;
use std::{collections::BTreeMap, sync::Mutex};

use fc_storage::{StorageOverride, StorageOverrideHandler};

use crate::cli::EthConfiguration;
use crate::eth::{spawn_frontier_tasks, EthDependencies};

#[docify::export(wasm_executor)]
type ParachainExecutor = WasmExecutor<ParachainHostFunctions>;

pub type ParachainClient = TFullClient<Block, RuntimeApi, ParachainExecutor>;

pub type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport = TParachainBlockImport<Block, Arc<ParachainClient>, ParachainBackend>;

/// Assembly of PartialComponents (enough to run chain ops subcommands)
pub type Service = PartialComponents<
    ParachainClient,
    ParachainBackend,
    (),
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::TransactionPoolHandle<Block, ParachainClient>,
    (
        ParachainBlockImport,
        Option<Telemetry>,
        Option<TelemetryWorkerHandle>,
        Arc<fc_db::kv::Backend<Block, ParachainClient>>,
        Arc<dyn StorageOverride<Block>>,
        FilterPool,
        FeeHistoryCache,
    ),
>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[docify::export(component_instantiation)]
pub fn new_partial(config: &Configuration) -> Result<Service, sc_service::Error> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let heap_pages = config
        .executor
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
            extra_pages: h as _,
        });

    let executor = ParachainExecutor::builder()
        .with_execution_method(config.executor.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.executor.max_runtime_instances)
        .with_runtime_cache_size(config.executor.runtime_cache_size)
        .build();

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
            true,
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

    let import_queue = build_import_queue(
        client.clone(),
        block_import.clone(),
        config,
        telemetry.as_ref().map(|telemetry| telemetry.handle()),
        &task_manager,
    );

    // ETH support
    let frontier_backend = Arc::new(fc_db::kv::Backend::open(
        Arc::clone(&client),
        &config.database,
        &db_config_dir(config),
    )?);
    let storage_override = Arc::new(StorageOverrideHandler::<Block, _, _>::new(client.clone()));
    let filter_pool: FilterPool = Arc::new(Mutex::new(BTreeMap::new()));
    let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));

    Ok(PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: (),
        other: (
            block_import,
            telemetry,
            telemetry_worker_handle,
            frontier_backend,
            storage_override,
            filter_pool,
            fee_history_cache,
        ),
    })
}

/// Build the import queue for the parachain runtime.
fn build_import_queue(
    client: Arc<ParachainClient>,
    block_import: ParachainBlockImport,
    config: &Configuration,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
) -> sc_consensus::DefaultImportQueue<Block> {
    cumulus_client_consensus_aura::equivocation_import_queue::fully_verifying_import_queue::<
        sp_consensus_aura::sr25519::AuthorityPair,
        _,
        _,
        _,
        _,
    >(
        client,
        block_import,
        move |_, _| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            Ok(timestamp)
        },
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        telemetry,
    )
}

#[allow(clippy::too_many_arguments)]
fn start_consensus(
    client: Arc<ParachainClient>,
    backend: Arc<ParachainBackend>,
    block_import: ParachainBlockImport,
    prometheus_registry: Option<&Registry>,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
    relay_chain_interface: Arc<dyn RelayChainInterface>,
    transaction_pool: Arc<sc_transaction_pool::TransactionPoolHandle<Block, ParachainClient>>,
    keystore: KeystorePtr,
    relay_chain_slot_duration: Duration,
    para_id: ParaId,
    collator_key: CollatorPair,
    overseer_handle: OverseerHandle,
    announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
) -> Result<(), sc_service::Error> {
    let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
        task_manager.spawn_handle(),
        client.clone(),
        transaction_pool,
        prometheus_registry,
        telemetry.clone(),
    );

    let proposer = Proposer::new(proposer_factory);

    let collator_service = CollatorService::new(
        client.clone(),
        Arc::new(task_manager.spawn_handle()),
        announce_block,
        client.clone(),
    );

    let params = AuraParams {
        create_inherent_data_providers: move |_, ()| async move { Ok(()) },
        block_import,
        para_client: client.clone(),
        para_backend: backend,
        relay_client: relay_chain_interface,
        code_hash_provider: move |block_hash| {
            client
                .code_at(block_hash)
                .ok()
                .map(|c| ValidationCode::from(c).hash())
        },
        keystore,
        collator_key,
        para_id,
        overseer_handle,
        relay_chain_slot_duration,
        proposer,
        collator_service,
        authoring_duration: Duration::from_millis(2000),
        reinitialize: false,
    };
    let fut = aura::run::<Block, sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _, _, _>(
        params,
    );
    task_manager
        .spawn_essential_handle()
        .spawn("aura", None, fut);

    Ok(())
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
pub async fn start_parachain_node(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    ethereum_config: EthConfiguration,
    collator_options: CollatorOptions,
    para_id: ParaId,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial(&parachain_config)?;
    let (
        block_import,
        mut telemetry,
        telemetry_worker_handle,
        frontier_backend,
        storage_override,
        filter_pool,
        fee_history_cache,
    ) = params.other;

    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let net_config = sc_network::config::FullNetworkConfiguration::<
        _,
        _,
        sc_network::NetworkWorker<Block, Hash>,
    >::new(&parachain_config.network, prometheus_registry.clone());

    let client = params.client.clone();
    let backend = params.backend.clone();
    let mut task_manager = params.task_manager;

    let (relay_chain_interface, collator_key) = build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options.clone(),
        hwbench.clone(),
    )
    .await
    .map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

    let validator = parachain_config.role.is_authority();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue_service = params.import_queue.service();

    // NOTE: because we use Aura here explicitly, we can use `CollatorSybilResistance::Resistant`
    // when starting the network.
    let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
        build_network(BuildNetworkParams {
            parachain_config: &parachain_config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            para_id,
            spawn_handle: task_manager.spawn_handle(),
            relay_chain_interface: relay_chain_interface.clone(),
            import_queue: params.import_queue,
            sybil_resistance_level: CollatorSybilResistance::Resistant, // because of Aura
        })
        .await?;

    if parachain_config.offchain_worker.enabled {
        use futures::FutureExt;

        let offchain_workers =
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(params.keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                is_validator: parachain_config.role.is_authority(),
                enable_http_requests: false,
                custom_extensions: move |_| vec![],
            })?;
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            offchain_workers
                .run(client.clone(), task_manager.spawn_handle())
                .boxed(),
        );
    }

    let role = parachain_config.role;
    let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
        fc_mapping_sync::EthereumBlockNotification<Block>,
    > = Default::default();
    let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();
        let network = network.clone();
        let sync = sync_service.clone();

        let is_authority = role.is_authority();
        let enable_dev_signer = ethereum_config.enable_dev_signer;
        let max_past_logs = ethereum_config.max_past_logs;
        let execute_gas_limit_multiplier = ethereum_config.execute_gas_limit_multiplier;
        let filter_pool = filter_pool.clone();
        let frontier_backend = frontier_backend.clone();
        let pubsub_notification_sinks = pubsub_notification_sinks.clone();
        let storage_override = storage_override.clone();
        let fee_history_cache = fee_history_cache.clone();
        let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
            task_manager.spawn_handle(),
            storage_override.clone(),
            ethereum_config.eth_log_block_cache,
            ethereum_config.eth_statuses_cache,
            prometheus_registry.clone(),
        ));

        let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
        let target_gas_price = ethereum_config.target_gas_price;
        let pending_create_inherent_data_providers = move |_, ()| async move {
            let current = sp_timestamp::InherentDataProvider::from_system_time();
            let next_slot = current.timestamp().as_millis() + slot_duration.as_millis();
            let timestamp = sp_timestamp::InherentDataProvider::new(next_slot.into());
            let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
            let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
            Ok((slot, timestamp, dynamic_fee))
        };

        Box::new(move |subscription_task_executor| {
            let eth_deps = EthDependencies {
                client: client.clone(),
                pool: transaction_pool.clone(),
                graph: transaction_pool.clone(),
                converter: Some::<TransactionConverter<Block>>(TransactionConverter::default()),
                is_authority,
                enable_dev_signer,
                network: network.clone(),
                sync: sync.clone(),
                frontier_backend: frontier_backend.clone(),
                storage_override: storage_override.clone(),
                block_data_cache: block_data_cache.clone(),
                filter_pool: filter_pool.clone(),
                max_past_logs: max_past_logs.clone(),
                fee_history_cache: fee_history_cache.clone(),
                fee_history_cache_limit: ethereum_config.fee_history_limit,
                execute_gas_limit_multiplier,
                forced_parent_hashes: None,
                pending_create_inherent_data_providers,
            };

            let m = crate::rpc::create_full(client.clone(), transaction_pool.clone())
                .map_err(Into::into);
            let m1 = crate::eth::create(
                eth_deps,
                subscription_task_executor,
                pubsub_notification_sinks.clone(),
            )
            .map_err(Into::into);

            match (m, m1) {
                (Ok(mut rpc), Ok(eth)) => {
                    rpc.merge(eth).unwrap();

                    Ok(rpc)
                }
                (Err(e), _) | (_, Err(e)) => Err(e),
            }
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        config: parachain_config,
        keystore: params.keystore_container.keystore(),
        backend: backend.clone(),
        network,
        sync_service: sync_service.clone(),
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    spawn_frontier_tasks(
        &task_manager,
        client.clone(),
        backend.clone(),
        frontier_backend,
        filter_pool,
        storage_override,
        fee_history_cache,
        ethereum_config.fee_history_limit,
        sync_service.clone(),
        pubsub_notification_sinks,
    )
    .await;

    if let Some(hwbench) = hwbench {
        sc_sysinfo::print_hwbench(&hwbench);
        // Here you can check whether the hardware meets your chains' requirements. Putting a link
        // in there and swapping out the requirements for your own are probably a good idea. The
        // requirements for a para-chain are dictated by its relay-chain.
        match SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench, false) {
            Err(err) if validator => {
                log::warn!(
				"⚠️  The hardware does not meet the minimal requirements {} for role 'Authority'.",
				err
			);
            }
            _ => {}
        }

        if let Some(ref mut telemetry) = telemetry {
            let telemetry_handle = telemetry.handle();
            task_manager.spawn_handle().spawn(
                "telemetry_hwbench",
                None,
                sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
            );
        }
    }

    let announce_block = {
        let sync_service = sync_service.clone();
        Arc::new(move |hash, data| sync_service.announce_block(hash, data))
    };

    let relay_chain_slot_duration = Duration::from_secs(6);

    let overseer_handle = relay_chain_interface
        .overseer_handle()
        .map_err(|e| sc_service::Error::Application(Box::new(e)))?;

    start_relay_chain_tasks(StartRelayChainTasksParams {
        client: client.clone(),
        announce_block: announce_block.clone(),
        para_id,
        relay_chain_interface: relay_chain_interface.clone(),
        task_manager: &mut task_manager,
        da_recovery_profile: if validator {
            DARecoveryProfile::Collator
        } else {
            DARecoveryProfile::FullNode
        },
        import_queue: import_queue_service,
        relay_chain_slot_duration,
        recovery_handle: Box::new(overseer_handle.clone()),
        sync_service: sync_service.clone(),
    })?;

    if validator {
        start_consensus(
            client.clone(),
            backend,
            block_import,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface,
            transaction_pool,
            params.keystore_container.keystore(),
            relay_chain_slot_duration,
            para_id,
            collator_key.expect("Command line arguments do not allow this. qed"),
            overseer_handle,
            announce_block,
        )?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}

pub fn db_config_dir(config: &Configuration) -> PathBuf {
    config.base_path.config_dir(config.chain_spec.id())
}
