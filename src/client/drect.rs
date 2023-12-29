use crate::client::common::CommonClient;
use crate::client::event::ClientEvent;
use crate::client::{Client, SyncClient};
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TokenType, TxIdType};
use crate::storage::db::level_db::LevelDB;
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use crate::types::response::GetDataResponse;
use async_channel::Receiver;
use bitcoincore_rpc::bitcoin::Transaction;
use std::sync::Arc;
use tokio::runtime;
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct DirectClient<T: StorageProcessor + Clone> {
    rt: Arc<Runtime>,
    storage: T,
    pub(crate) base: CommonClient,
}
impl<T: StorageProcessor + Clone + Default> Default for DirectClient<T> {
    fn default() -> Self {
        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        Self {
            rt: Arc::new(rt),
            storage: T::default(),
            base: CommonClient::default(),
        }
    }
}
impl<T: StorageProcessor + Clone> DirectClient<T> {
    pub fn new(rt: Arc<Runtime>, storage: T, base: CommonClient) -> Self {
        Self { rt, storage, base }
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor + Clone> Client for DirectClient<T> {
    async fn get_event(&self) -> IndexerResult<Option<ClientEvent>> {
        self.base.get_event().await
    }

    async fn push_event(&self, event: IndexerEvent) -> IndexerResult<()> {
        self.base.push_event(event).await
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
    fn rx(&self) -> async_channel::Receiver<ClientEvent> {
        self.base.rx()
    }

    async fn report_height(&self, height: u32) -> IndexerResult<()> {
        self.base.report_height(height).await
    }
    async fn report_reorg(&self, txs: Vec<TxIdType>) -> IndexerResult<()> {
        self.base.report_reorg(txs).await
    }
}

impl<T: StorageProcessor + Clone> DirectClient<T> {
    pub fn get(&self) -> Vec<u8> {
        self.base.get()
    }
}
impl<T: StorageProcessor + Clone> DirectClient<T> {
    pub fn sync_push_event(&self, event: IndexerEvent) {
        self.base.sync_push_event(event);
    }
}

impl<T: StorageProcessor + Clone> SyncClient for DirectClient<T> {
    fn get_event(&self) -> IndexerResult<Option<ClientEvent>> {
        self.base.do_get_data()
    }

    fn report_height(&self, height: u32) -> IndexerResult<()> {
        self.base
            .tx
            .send_blocking(IndexerEvent::ReportHeight(height))
            .unwrap();
        Ok(())
    }

    fn report_reorg(&self, txs: Vec<TxIdType>) -> IndexerResult<()> {
        self.base
            .tx
            .send_blocking(IndexerEvent::ReportReorg(txs))
            .unwrap();
        Ok(())
    }

    fn push_event(&self, event: IndexerEvent) -> IndexerResult<()> {
        self.base.tx.send_blocking(event).unwrap();
        Ok(())
    }

    fn get_balance(
        &mut self,
        address_type: AddressType,
        token_type: TokenType,
    ) -> IndexerResult<BalanceType> {
        self.rt
            .block_on(async { self.storage.get_balance(&address_type, &token_type).await })
    }

    fn get_all_balance(
        &mut self,
        address_type: AddressType,
    ) -> IndexerResult<Vec<(TokenType, BalanceType)>> {
        Ok(self
            .rt
            .block_on(async { self.storage.get_all_balance(&address_type).await })?)
    }

    fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()> {
        self.base.do_update_delta(result)
    }

    fn rx(&self) -> Receiver<ClientEvent> {
        self.base.rx.clone()
    }
}
