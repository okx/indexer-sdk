pub mod memory;
pub mod thread_safe;

use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TxIdType};
use crate::types::delta::TransactionDelta;

#[async_trait::async_trait]
pub trait StorageProcessor: Send + Sync {
    async fn get_balance(&self, address: &AddressType) -> IndexerResult<BalanceType>;

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()>;
    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()>;
}