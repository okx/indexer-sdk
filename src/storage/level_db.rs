// use crate::error::IndexerResult;
// use crate::event::{AddressType, BalanceType, TxIdType};
// use crate::storage::StorageProcessor;
// use crate::types::delta::TransactionDelta;
//
// pub struct LevelDBStorageProcessor<T: StorageProcessor> {
//     internal: T,
// }
//
// #[async_trait::async_trait]
// impl<T: StorageProcessor> StorageProcessor for LevelDBStorageProcessor<T> {
//     async fn get_balance(&self, address: &AddressType) -> IndexerResult<BalanceType> {
//         // TODO
//         self.internal.get_balance(address).await
//     }
//
//     async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
//         // TODO:
//         self.internal.add_transaction_delta(transaction).await
//     }
//
//     async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
//         // TODO:
//         self.internal.remove_transaction_delta(tx_id).await
//     }
//
//     async fn seen_and_store_txs(&mut self, tx_id: TxIdType) -> IndexerResult<()> {
//         todo!()
//     }
// }
