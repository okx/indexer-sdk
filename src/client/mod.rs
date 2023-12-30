use crate::client::event::ClientEvent;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TokenType, TxIdType};
use crate::types::delta::TransactionDelta;
use crate::types::response::AllBalanceResponse;

pub mod common;

pub mod drect;
pub mod event;
pub mod ffi;

#[async_trait::async_trait]
pub trait Client: Send + Sync {
    async fn get_event(&self) -> IndexerResult<Option<ClientEvent>>;
    async fn report_height(&self, height: u32) -> IndexerResult<()>;
    async fn report_reorg(&self, txs: Vec<TxIdType>) -> IndexerResult<()>;
    async fn push_event(&self, event: IndexerEvent) -> IndexerResult<()>;
    async fn get_balance(
        &mut self,
        address_type: AddressType,
        token_type: TokenType,
    ) -> IndexerResult<BalanceType>;
    async fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()>;

    fn rx(&self) -> async_channel::Receiver<ClientEvent>;
}

#[async_trait::async_trait]
pub trait SyncClient {
    fn get_event(&self) -> IndexerResult<Option<ClientEvent>>;
    fn report_height(&self, height: u32) -> IndexerResult<()>;
    fn report_reorg(&self, txs: Vec<TxIdType>) -> IndexerResult<()>;
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
}
