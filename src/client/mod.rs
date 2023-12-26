use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TxIdType};
use crate::types::delta::TransactionDelta;
use crate::types::response::GetDataResponse;

pub mod common;

mod concurrent_client;
pub mod event;
pub mod ffi;

#[async_trait::async_trait]
pub trait Client: Send + Sync {
    async fn get_data(&self) -> IndexerResult<Option<GetDataResponse>>;
    async fn push_data(&self, data: Vec<u8>) -> IndexerResult<()>;
    async fn get_balance(&self, address_type: AddressType) -> IndexerResult<BalanceType>;
    async fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()>;
    async fn tx_consumed(&mut self, tx_id: TxIdType) -> IndexerResult<()>;
}
