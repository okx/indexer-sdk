mod db;
mod kv;
pub mod memory;
mod prefix;
pub mod thread_safe;

use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::types::delta::TransactionDelta;
use bitcoincore_rpc::bitcoin::Transaction;

#[async_trait::async_trait]
pub trait StorageProcessor: Send + Sync {
    async fn get_balance(
        &mut self,
        token_type: &TokenType,
        address: &AddressType,
    ) -> IndexerResult<BalanceType>;
    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()>;
    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()>;
    async fn seen_and_store_txs(&mut self, tx: Transaction) -> IndexerResult<bool>;
    async fn seen_tx(&mut self, tx_id: TxIdType) -> IndexerResult<bool>;

    async fn get_all_un_consumed_txs(&mut self) -> IndexerResult<Vec<TxIdType>>;
}

#[async_trait::async_trait]
impl StorageProcessor for Box<dyn StorageProcessor> {
    async fn get_balance(
        &mut self,
        token_type: &TokenType,
        address: &AddressType,
    ) -> IndexerResult<BalanceType> {
        self.as_mut().get_balance(token_type, address).await
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        self.as_mut().add_transaction_delta(transaction).await
    }

    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        self.as_mut().remove_transaction_delta(tx_id).await
    }

    async fn seen_and_store_txs(&mut self, tx: Transaction) -> IndexerResult<bool> {
        self.as_mut().seen_and_store_txs(tx).await
    }

    async fn seen_tx(&mut self, tx_id: TxIdType) -> IndexerResult<bool> {
        self.as_mut().seen_tx(tx_id).await
    }

    async fn get_all_un_consumed_txs(&mut self) -> IndexerResult<Vec<TxIdType>> {
        self.as_mut().get_all_un_consumed_txs().await
    }
}
