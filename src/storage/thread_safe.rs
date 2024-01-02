use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::prefix::DeltaStatus;
use crate::storage::{SeenStatusResponse, StorageProcessor};
use crate::types::delta::TransactionDelta;
use crate::types::response::AllBalanceResponse;
use bitcoincore_rpc::bitcoin::Transaction;
use log::debug;
use std::collections::HashMap;
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
        &mut self,
        address: &AddressType,
        token_type: &TokenType,
    ) -> IndexerResult<BalanceType> {
        let count = self.rw_lock.read().await;
        let ret = self.internal.get_balance(address, token_type).await?;
        debug!("write count:{:?}", count);
        Ok(ret)
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        let mut write = self.rw_lock.write().await;
        self.internal.add_transaction_delta(transaction).await?;
        *write += 1;
        Ok(())
    }

    async fn remove_transaction_delta(
        &mut self,
        tx_id: &TxIdType,
        status: DeltaStatus,
    ) -> IndexerResult<()> {
        let mut write = self.rw_lock.write().await;
        self.internal
            .remove_transaction_delta(tx_id, status)
            .await?;
        *write += 1;
        Ok(())
    }

    async fn seen_and_store_txs(&mut self, tx: &Transaction) -> IndexerResult<SeenStatusResponse> {
        let write = self.rw_lock.write().await;
        let ret = self.internal.seen_and_store_txs(tx).await?;
        drop(write);
        Ok(ret)
    }

    async fn seen_tx(&mut self, tx_id: TxIdType) -> IndexerResult<SeenStatusResponse> {
        let count = self.rw_lock.read().await;
        let ret = self.internal.seen_tx(tx_id).await?;
        drop(count);
        Ok(ret)
    }

    async fn get_all_un_consumed_txs(&mut self) -> IndexerResult<HashMap<TxIdType, i64>> {
        let read = self.rw_lock.write().await;
        let ret = self.internal.get_all_un_consumed_txs().await;
        drop(read);
        ret
    }

    async fn is_tx_executed(&mut self, tx_id: &TxIdType) -> IndexerResult<bool> {
        let read = self.rw_lock.write().await;
        let ret = self.internal.is_tx_executed(tx_id).await;
        drop(read);
        ret
    }

    async fn get_all_balance(
        &mut self,
        address: &AddressType,
    ) -> IndexerResult<Vec<AllBalanceResponse>> {
        let read = self.rw_lock.write().await;
        let ret = self.internal.get_all_balance(address).await;
        drop(read);
        ret
    }

    async fn simple_set(
        &mut self,
        tx_id: &TxIdType,
        key: &[u8],
        value: Vec<u8>,
    ) -> IndexerResult<()> {
        let mut write = self.rw_lock.write().await;
        self.internal.simple_set(tx_id, key, value).await?;
        *write += 1;
        Ok(())
    }

    async fn simple_get(&mut self, tx_id: &TxIdType, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        let read = self.rw_lock.write().await;
        let ret = self.internal.simple_get(tx_id, key).await;
        drop(read);
        ret
    }

    async fn save_height_tx(&mut self, height: u32, tx_id: TxIdType) -> IndexerResult<()> {
        let mut write = self.rw_lock.write().await;
        self.internal.save_height_tx(height, tx_id).await?;
        *write += 1;
        Ok(())
    }

    async fn remove_height_traces(&mut self, height: u32) -> IndexerResult<()> {
        let mut write = self.rw_lock.write().await;
        self.internal.remove_height_traces(height).await?;
        *write += 1;
        Ok(())
    }
}
