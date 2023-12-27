use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use log::debug;
use tokio::sync::RwLock;

pub struct ThreadSafeStorageProcessor<T: StorageProcessor> {
    internal: T,
    rw_lock: RwLock<u64>,
}

impl<T: StorageProcessor> ThreadSafeStorageProcessor<T> {
    pub fn new(internal: T) -> Self {
        Self {
            internal,
            rw_lock: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor> StorageProcessor for ThreadSafeStorageProcessor<T> {
    async fn get_balance(
        &self,
        token_type: &TokenType,
        address: &AddressType,
    ) -> IndexerResult<BalanceType> {
        let count = self.rw_lock.read().await;
        let ret = self.internal.get_balance(token_type, address).await?;
        debug!("write count:{:?}", count);
        Ok(ret)
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        let mut write = self.rw_lock.write().await;
        self.internal.add_transaction_delta(transaction).await?;
        *write += 1;
        Ok(())
    }

    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        let mut write = self.rw_lock.write().await;
        self.internal.remove_transaction_delta(tx_id).await?;
        *write += 1;
        Ok(())
    }

    async fn seen_and_store_txs(&mut self, tx_id: TxIdType) -> IndexerResult<bool> {
        let write = self.rw_lock.write().await;
        let ret = self.internal.seen_and_store_txs(tx_id).await?;
        drop(write);
        Ok(ret)
    }

    async fn seen_tx(&self, tx_id: TxIdType) -> IndexerResult<bool> {
        let count = self.rw_lock.read().await;
        let ret = self.internal.seen_tx(tx_id).await?;
        drop(count);
        Ok(ret)
    }
}
