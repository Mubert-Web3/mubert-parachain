//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use mubert_runtime::{
    opaque::Block, AccountId, AuthorDetails, AuthorId, AuthorityDetails, AuthorityId, Balance,
    EntityDetails, EntityId, Nonce,
};

use polkadot_sdk::*;

use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Instantiate all RPC extensions.
pub fn create_full<C, P>(
    client: Arc<C>,
    pool: Arc<P>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + Send
        + Sync
        + 'static,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: BlockBuilder<Block>,
    C::Api: pallet_ip_onchain_runtime_api::ApiIpOnchainRuntime<
        Block,
        EntityId,
        AuthorId,
        AuthorityId,
        EntityDetails,
        AuthorDetails,
        AuthorityDetails,
    >,
    P: TransactionPool + Sync + Send + 'static,
{
    use pallet_ip_onchain_rpc::{IpOnchainRpcApiServer, IpOnchainRpcHandler};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcExtension::new(());

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    module.merge(IpOnchainRpcHandler::new(client.clone()).into_rpc())?;

    Ok(module)
}
