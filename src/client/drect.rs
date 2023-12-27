use crate::client::common::CommonClient;
use crate::client::Client;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType};
use crate::storage::db::level_db::LevelDB;
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use crate::types::response::GetDataResponse;
use bitcoincore_rpc::bitcoin::Transaction;
use crossbeam::channel::Receiver;

#[derive(Clone)]
pub struct DirectClient<T: StorageProcessor + Clone> {
    storage: T,
    pub(crate) base: CommonClient,
}
impl<T: StorageProcessor + Clone + Default> Default for DirectClient<T> {
    fn default() -> Self {
        Self {
            storage: T::default(),
            base: CommonClient::default(),
        }
    }
}
impl<T: StorageProcessor + Clone> DirectClient<T> {
    pub fn new(storage: T, base: CommonClient) -> Self {
        Self { storage, base }
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor + Clone> Client for DirectClient<T> {
    async fn get_data(&self) -> IndexerResult<Option<Transaction>> {
        self.base.get_data().await
    }

    async fn push_data(&self, data: Vec<u8>) -> IndexerResult<()> {
        self.base.push_data(data).await
    }

    async fn get_balance(
        &mut self,
        address_type: AddressType,
        token_type: TokenType,
    ) -> IndexerResult<BalanceType> {
        self.storage.get_balance(&address_type, &token_type).await
    }

    async fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()> {
        self.base.update_delta(result).await
    }
}

impl<T: StorageProcessor + Clone> DirectClient<T> {
    pub fn rx(&self) -> Receiver<Transaction> {
        self.base.rx.clone()
    }
    pub fn get(&self) -> Vec<u8> {
        self.base.get()
    }
}
