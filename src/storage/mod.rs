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


#[async_trait::async_trait]
impl<T:StorageProcessor> StorageProcessor for Box<T>{
    async fn get_balance(&self, address: &AddressType) -> IndexerResult<BalanceType> {
        (**self).get_balance(address).await
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        (**self).add_transaction_delta(transaction).await
    }

    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        (**self).remove_transaction_delta(tx_id).await
    }
}


#[async_trait::async_trait]
impl StorageProcessor for Box<dyn StorageProcessor>{
    async fn get_balance(&self, address: &AddressType) -> IndexerResult<BalanceType> {
        (**self).get_balance(address).await
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        (**self).add_transaction_delta(transaction).await
    }

    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        (**self).remove_transaction_delta(tx_id).await
    }
}

