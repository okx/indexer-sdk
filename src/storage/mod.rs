pub mod db;
pub mod kv;
pub mod memory;
pub mod prefix;
pub mod thread_safe;

use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::prefix::{DeltaStatus, SeenStatus};
use crate::types::delta::TransactionDelta;
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
    ) -> IndexerResult<Vec<(TokenType, BalanceType)>>;

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
    ) -> IndexerResult<Vec<(TokenType, BalanceType)>> {
        self.as_mut().get_all_balance(address).await
    }
}
