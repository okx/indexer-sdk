use std::sync::Arc;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TxIdType};
use crate::client::Client;
use crate::storage::StorageProcessor;
use crate::storage::thread_safe::ThreadSafeStorageProcessor;
use crate::types::delta::TransactionDelta;
use crate::types::response::GetDataResponse;

pub struct ConcurrentClient<T: Client> {
    internal: T,
    storage: ThreadSafeStorageProcessor<Box<dyn StorageProcessor>>,
}

impl<T: Client> ConcurrentClient<T> {
    pub fn new(internal: T, storage: ThreadSafeStorageProcessor<Box<dyn StorageProcessor>>) -> Self {
        Self { internal, storage }
    }
}

#[async_trait::async_trait]
impl<T: Client> Client for ConcurrentClient<T> {
    async fn get_data(&self) -> IndexerResult<Option<GetDataResponse>> {
        self.internal.get_data().await
    }

    async fn push_data(&self, data: Vec<u8>) -> IndexerResult<()> {
        self.internal.push_data(data).await
    }

    async fn get_balance(&self, address_type: AddressType) -> IndexerResult<BalanceType> {
        self.storage.get_balance(&address_type).await
    }

    async fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()> {
        self.storage.add_transaction_delta(&result).await
    }

    async fn tx_consumed(&mut self, tx_id: TxIdType) -> IndexerResult<()> {
        self.storage.remove_transaction_delta(&tx_id).await
    }
}