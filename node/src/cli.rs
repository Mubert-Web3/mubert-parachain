use polkadot_sdk::*;
use std::path::PathBuf;

/// Sub-commands supported by the collator.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Remove the whole chain.
    PurgeChain(cumulus_client_cli::PurgeChainCmd),

    /// Export the genesis head data of the parachain.
    ///
    /// Head data is the encoded block header.
    #[command(alias = "export-genesis-state")]
    ExportGenesisHead(cumulus_client_cli::ExportGenesisHeadCommand),

    /// Export the genesis wasm of the parachain.
    ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),
    
    /// Key management cli utilities
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),

    /// Sub-commands concerned with benchmarking.
    /// The pallet benchmarking moved to the `pallet` sub-command.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}

const AFTER_HELP_EXAMPLE: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</></>
   <bold>parachain-template-node build-spec --disable-default-bootnode > plain-parachain-chainspec.json</>
           Export a chainspec for a local testnet in json format.
   <bold>parachain-template-node --chain plain-parachain-chainspec.json --tmp -- --chain rococo-local</>
           Launch a full node with chain specification loaded from plain-parachain-chainspec.json.
   <bold>parachain-template-node</>
           Launch a full node with default parachain <italic>local-testnet</> and relay chain <italic>rococo-local</>.
   <bold>parachain-template-node --collator</>
           Launch a collator with default parachain <italic>local-testnet</> and relay chain <italic>rococo-local</>.
 "#
);
#[derive(Debug, clap::Parser)]
#[command(
    propagate_version = true,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
#[command(after_help = AFTER_HELP_EXAMPLE)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[command(flatten)]
    pub run: cumulus_client_cli::RunCmd,

    /// Disable automatic hardware benchmarks.
    ///
    /// By default these benchmarks are automatically ran at startup and measure
    /// the CPU speed, the memory bandwidth and the disk speed.
    ///
    /// The results are then printed out in the logs, and also sent as part of
    /// telemetry, if telemetry is enabled.
    #[arg(long)]
    pub no_hardware_benchmarks: bool,

    /// Relay chain arguments
    #[arg(raw = true)]
    pub relay_chain_args: Vec<String>,

    #[clap(flatten)]
    pub ethereum_config: EthConfiguration,
}

#[derive(Debug)]
pub struct RelayChainCli {
    /// The actual relay chain cli object.
    pub base: polkadot_cli::RunCmd,

    /// Optional chain id that should be passed to the relay chain.
    pub chain_id: Option<String>,

    /// The base path that should be used by the relay chain.
    pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
    /// Parse the relay chain CLI parameters using the para chain `Configuration`.
    pub fn new<'a>(
        para_config: &sc_service::Configuration,
        relay_chain_args: impl Iterator<Item = &'a String>,
    ) -> Self {
        let extension = crate::chain_spec::Extensions::try_get(&*para_config.chain_spec);
        let chain_id = extension.map(|e| e.relay_chain.clone());
        let base_path = para_config.base_path.path().join("polkadot");
        Self {
            base_path: Some(base_path),
            chain_id,
            base: clap::Parser::parse_from(relay_chain_args),
        }
    }
}

/// The ethereum-compatibility configuration used to run a node.
#[derive(Clone, Debug, clap::Parser)]
pub struct EthConfiguration {
    /// Maximum number of logs in a query.
    #[arg(long, default_value = "10000")]
    pub max_past_logs: u32,

    /// Maximum fee history cache size.
    #[arg(long, default_value = "2048")]
    pub fee_history_limit: u64,

    #[arg(long)]
    pub enable_dev_signer: bool,

    /// The dynamic-fee pallet target gas price set by block author
    #[arg(long, default_value = "1")]
    pub target_gas_price: u64,

    /// Maximum allowed gas limit will be `block.gas_limit * execute_gas_limit_multiplier`
    /// when using eth_call/eth_estimateGas.
    #[arg(long, default_value = "10")]
    pub execute_gas_limit_multiplier: u64,

    /// Size in bytes of the LRU cache for block data.
    #[arg(long, default_value = "50")]
    pub eth_log_block_cache: usize,

    /// Size in bytes of the LRU cache for transactions statuses data.
    #[arg(long, default_value = "50")]
    pub eth_statuses_cache: usize,
}
