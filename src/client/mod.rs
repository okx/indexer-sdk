use crate::client::event::ClientEvent;
use crate::dispatcher::event::DispatchEvent;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TokenType, TxIdType};
use crate::types::delta::TransactionDelta;
use crate::types::response::AllBalanceResponse;
use bitcoincore_rpc::bitcoin::{Transaction, Txid};
use std::sync::Arc;

pub mod common;

pub mod drect;
pub mod event;
pub mod ffi;

#[async_trait::async_trait]
pub trait Client: Send + Sync {
    async fn get_event(&self) -> IndexerResult<Option<ClientEvent>>;
    async fn report_height(&self, height: u32) -> IndexerResult<()>;
    async fn report_reorg(&self, number: u32) -> IndexerResult<()>;
    async fn push_event(&self, event: DispatchEvent) -> IndexerResult<()>;
    async fn get_balance(
        &mut self,
        address_type: AddressType,
        token_type: TokenType,
    ) -> IndexerResult<BalanceType>;
    async fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()>;

    fn rx(&self) -> async_channel::Receiver<ClientEvent>;
}

#[async_trait::async_trait]
pub trait SyncClient: Send + Sync {
    fn get_event(&self) -> IndexerResult<Option<ClientEvent>>;
    fn block_get_event(&self) -> IndexerResult<ClientEvent>;
    fn report_height(&self, height: u32) -> IndexerResult<()>;
    fn report_reorg(&self, org_number: u32) -> IndexerResult<()>;
    fn push_event(&self, event: IndexerEvent) -> IndexerResult<()>;
    fn get_balance(
        &mut self,
        address_type: AddressType,
        token_type: TokenType,
    ) -> IndexerResult<BalanceType>;

    fn get_all_balance(
        &mut self,
        address_type: AddressType,
    ) -> IndexerResult<Vec<AllBalanceResponse>>;

    fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()>;

    fn rx(&self) -> async_channel::Receiver<ClientEvent>;

    fn simple_set(&mut self, tx_id: &TxIdType, key: &[u8], value: Vec<u8>) -> IndexerResult<()>;

    fn simple_get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>>;

    fn remove_tx_traces(&mut self, tx_id: Vec<TxIdType>) -> IndexerResult<()>;

    fn get_btc_client(&self) -> Arc<bitcoincore_rpc::Client>;

    fn get_transaction_by_tx_id(&self, txid: Txid) -> IndexerResult<Option<Transaction>>;
}
