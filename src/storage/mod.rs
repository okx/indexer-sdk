pub mod db;
pub mod kv;
pub mod memory;
pub mod prefix;
pub mod thread_safe;

use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::prefix::{DeltaStatus, SeenStatus};
use crate::types::delta::TransactionDelta;
use crate::types::response::AllBalanceResponse;
use bitcoincore_rpc::bitcoin::Transaction;
use std::collections::HashMap;

#[async_trait::async_trait]
pub trait StorageProcessor: Send + Sync {
    async fn get_balance(
        &mut self,
        address: &AddressType,
        token_type: &TokenType,
    ) -> IndexerResult<BalanceType>;

    async fn get_all_balance(
        &mut self,
        address: &AddressType,
    ) -> IndexerResult<Vec<AllBalanceResponse>>;

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()>;
    async fn remove_transaction_delta(
        &mut self,
        tx_id: &TxIdType,
        status: DeltaStatus,
    ) -> IndexerResult<()>;

    async fn seen_and_store_txs(&mut self, tx: &Transaction) -> IndexerResult<SeenStatusResponse>;

    async fn seen_tx(&mut self, tx_id: TxIdType) -> IndexerResult<SeenStatusResponse>;

    async fn is_tx_executed(&mut self, tx_id: &TxIdType) -> IndexerResult<bool>;

    async fn get_all_un_consumed_txs(&mut self) -> IndexerResult<HashMap<TxIdType, i64>>;

    async fn simple_set(
        &mut self,
        tx_id: &TxIdType,
        key: &[u8],
        value: Vec<u8>,
    ) -> IndexerResult<()>;

    async fn simple_get(&mut self, tx_id: &TxIdType, key: &[u8]) -> IndexerResult<Option<Vec<u8>>>;

    async fn save_height_tx(&mut self, height: u32, tx_id: TxIdType) -> IndexerResult<()>;

    async fn remove_height_traces(&mut self, height: u32) -> IndexerResult<()>;
}

#[derive(Clone, Debug)]
pub struct SeenStatusResponse {
    seen: bool,
    status: SeenStatus,
}
impl SeenStatusResponse {
    pub fn is_seen(&self) -> bool {
        self.seen
    }
    pub fn is_executed(&self) -> bool {
        self.status == SeenStatus::Executed
    }
}

#[async_trait::async_trait]
impl StorageProcessor for Box<dyn StorageProcessor> {
    async fn get_balance(
        &mut self,
        address: &AddressType,
        token_type: &TokenType,
    ) -> IndexerResult<BalanceType> {
        self.as_mut().get_balance(address, token_type).await
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        self.as_mut().add_transaction_delta(transaction).await
    }

    async fn remove_transaction_delta(
        &mut self,
        tx_id: &TxIdType,
        status: DeltaStatus,
    ) -> IndexerResult<()> {
        self.as_mut().remove_transaction_delta(tx_id, status).await
    }

    async fn seen_and_store_txs(&mut self, tx: &Transaction) -> IndexerResult<SeenStatusResponse> {
        self.as_mut().seen_and_store_txs(tx).await
    }

    async fn seen_tx(&mut self, tx_id: TxIdType) -> IndexerResult<SeenStatusResponse> {
        self.as_mut().seen_tx(tx_id).await
    }

    async fn get_all_un_consumed_txs(&mut self) -> IndexerResult<HashMap<TxIdType, i64>> {
        self.as_mut().get_all_un_consumed_txs().await
    }

    async fn is_tx_executed(&mut self, tx_id: &TxIdType) -> IndexerResult<bool> {
        self.as_mut().is_tx_executed(tx_id).await
    }

    async fn get_all_balance(
        &mut self,
        address: &AddressType,
    ) -> IndexerResult<Vec<AllBalanceResponse>> {
        self.as_mut().get_all_balance(address).await
    }

    async fn simple_set(
        &mut self,
        tx_id: &TxIdType,
        key: &[u8],
        value: Vec<u8>,
    ) -> IndexerResult<()> {
        self.as_mut().simple_set(tx_id, key, value).await
    }

    async fn simple_get(&mut self, tx_id: &TxIdType, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        self.as_mut().simple_get(tx_id, key).await
    }

    async fn save_height_tx(&mut self, height: u32, tx_id: TxIdType) -> IndexerResult<()> {
        self.as_mut().save_height_tx(height, tx_id).await
    }

    async fn remove_height_traces(&mut self, height: u32) -> IndexerResult<()> {
        self.as_mut().remove_height_traces(height).await
    }
}
