use arweave_rust::ar_substrate::extension::ArweaveSignerPtr;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use polkadot_sdk::*;
use std::sync::Arc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

#[rpc(client, server)]
pub trait ArweaveSignerApi<BlockHash> {
    #[method(name = "arweaveSigner_toggleEnable")]
    fn toggle_enable(&self, at: Option<BlockHash>) -> RpcResult<()>;
}

pub struct ArweaveSigner<C, B> {
    _client: Arc<C>,
    signer: ArweaveSignerPtr,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> ArweaveSigner<C, B> {
    pub fn new(_client: Arc<C>, signer: ArweaveSignerPtr) -> Self {
        ArweaveSigner {
            _client,
            signer,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> ArweaveSignerApiServer<<Block as BlockT>::Hash> for ArweaveSigner<C, Block>
where
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    Block: BlockT,
{
    fn toggle_enable(&self, _at: Option<Block::Hash>) -> RpcResult<()> {
        self.signer.toggle_enable();
        Ok(())
    }
}
