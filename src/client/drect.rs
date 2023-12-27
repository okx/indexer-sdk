use crate::client::common::CommonClient;
use crate::client::Client;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType};
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use crate::types::response::GetDataResponse;

pub struct DirectClient<T: StorageProcessor> {
    storage: T,
    base: CommonClient,
}
#[async_trait::async_trait]
impl<T: StorageProcessor> Client for DirectClient<T> {
    async fn get_data(&self) -> IndexerResult<Option<GetDataResponse>> {
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
