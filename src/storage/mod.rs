pub mod memory;
pub mod thread_safe;
mod level_db;

use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TxIdType};
use crate::types::delta::TransactionDelta;

#[async_trait::async_trait]
pub trait StorageProcessor: Send + Sync {
    async fn get_balance(&self, address: &AddressType) -> IndexerResult<BalanceType>;
    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()>;
    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()>;
    async fn seen_and_store_txs(&mut self,tx_id:TxIdType)->IndexerResult<bool>;
}


#[async_trait::async_trait]
impl StorageProcessor for Box<dyn StorageProcessor>{
    async fn get_balance(&self, address: &AddressType) -> IndexerResult<BalanceType> {
        self.as_ref().get_balance(address).await
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        self.as_mut().add_transaction_delta(transaction).await
    }

    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        self.as_mut().remove_transaction_delta(tx_id).await

    }

    async fn seen_and_store_txs(&mut self, tx_id: TxIdType) -> IndexerResult<bool> {
        self.as_mut().seen_and_store_txs(tx_id).await
    }
}