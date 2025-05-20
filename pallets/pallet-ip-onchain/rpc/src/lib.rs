use std::sync::Arc;

extern crate alloc;

use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::error::ErrorObject};
use scale_codec::Codec;

use polkadot_sdk::*;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

pub use pallet_ip_onchain_runtime_api::ApiIpOnchainRuntime;

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

#[rpc(server)]
pub trait IpOnchainRpcApi<
    BlockHash,
    EntityId,
    AuthorId,
    AuthorityId,
    EntityDetails,
    AuthorDetails,
    AuthorityDetails,
>
{
    /// Retrieves the details of an entity by its `entity_id`.
    #[method(name = "ipOnchain_entity")]
    fn entity(&self, entity_id: EntityId, at: Option<BlockHash>) -> RpcResult<EntityDetails>;

    #[method(name = "ipOnchain_entities")]
    fn entities(
        &self,
        from: EntityId,
        to: EntityId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(EntityId, EntityDetails)>>;

    #[method(name = "ipOnchain_author")]
    fn author(&self, author_id: AuthorId, at: Option<BlockHash>) -> RpcResult<AuthorDetails>;

    #[method(name = "ipOnchain_authors")]
    fn authors(
        &self,
        from: AuthorId,
        to: AuthorId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(AuthorId, AuthorDetails)>>;

    #[method(name = "ipOnchain_authority")]
    fn authority(
        &self,
        authority_id: AuthorityId,
        at: Option<BlockHash>,
    ) -> RpcResult<AuthorityDetails>;

    #[method(name = "ipOnchain_authorities")]
    fn authorities(
        &self,
        from: AuthorityId,
        to: AuthorityId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<(AuthorityId, AuthorityDetails)>>;
}

pub struct IpOnchainRpcHandler<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> IpOnchainRpcHandler<C, B> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, EntityId, AuthorId, AuthorityId, EntityDetails, AuthorDetails, AuthorityDetails>
    IpOnchainRpcApiServer<
        <Block as BlockT>::Hash,
        EntityId,
        AuthorId,
        AuthorityId,
        EntityDetails,
        AuthorDetails,
        AuthorityDetails,
    > for IpOnchainRpcHandler<C, Block>
where
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: pallet_ip_onchain_runtime_api::ApiIpOnchainRuntime<
        Block,
        EntityId,
        AuthorId,
        AuthorityId,
        EntityDetails,
        AuthorDetails,
        AuthorityDetails,
    >,
    Block: BlockT,
    EntityId: Codec + Send + Sync + 'static,
    AuthorId: Codec + Send + Sync + 'static,
    AuthorityId: Codec + Send + Sync + 'static,
    EntityDetails: Codec + Send + Sync + 'static,
    AuthorDetails: Codec + Send + Sync + 'static,
    AuthorityDetails: Codec + Send + Sync + 'static,
{
    fn entity(&self, entity_id: EntityId, at: Option<Block::Hash>) -> RpcResult<EntityDetails> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        let result = api.entity(at, entity_id).map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e.to_string()),
            )
        })?;

        Ok(result.map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e),
            )
        })?)
    }

    fn entities(
        &self,
        from: EntityId,
        to: EntityId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(EntityId, EntityDetails)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        let result = api.entities(at, from, to).map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e.to_string()),
            )
        })?;

        Ok(result.map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e),
            )
        })?)
    }

    fn author(&self, author_id: AuthorId, at: Option<Block::Hash>) -> RpcResult<AuthorDetails> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        let result = api.author(at, author_id).map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e.to_string()),
            )
        })?;

        Ok(result.map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e),
            )
        })?)
    }
    fn authors(
        &self,
        from: AuthorId,
        to: AuthorId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(AuthorId, AuthorDetails)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        let result = api.authors(at, from, to).map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e.to_string()),
            )
        })?;

        Ok(result.map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e),
            )
        })?)
    }

    fn authority(
        &self,
        authority_id: AuthorityId,
        at: Option<Block::Hash>,
    ) -> RpcResult<AuthorityDetails> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        let result = api.authority(at, authority_id).map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e.to_string()),
            )
        })?;

        Ok(result.map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e),
            )
        })?)
    }
    fn authorities(
        &self,
        from: AuthorityId,
        to: AuthorityId,
        at: Option<Block::Hash>,
    ) -> RpcResult<Vec<(AuthorityId, AuthorityDetails)>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        let result = api.authorities(at, from, to).map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e.to_string()),
            )
        })?;

        Ok(result.map_err(|e| {
            ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query details.",
                Some(e),
            )
        })?)
    }
}
