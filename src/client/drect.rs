use crate::client::common::CommonClient;
use crate::client::event::ClientEvent;
use crate::client::{Client, SyncClient};
use crate::dispatcher::event::DispatchEvent;
use crate::error::{IndexerError, IndexerResult};
use crate::event::{AddressType, BalanceType, IndexerEvent, TokenType, TxIdType};
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use crate::types::response::AllBalanceResponse;
use async_channel::Receiver;
use bitcoincore_rpc::bitcoin::{Transaction, Txid};
use bitcoincore_rpc::RpcApi;
use std::sync::Arc;
use tokio::runtime;
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct DirectClient<T: StorageProcessor + Clone> {
    rt: Arc<Runtime>,
    btc_client: Option<Arc<bitcoincore_rpc::Client>>,
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
            btc_client: None,
            storage: T::default(),
            base: CommonClient::default(),
        }
    }
}
impl<T: StorageProcessor + Clone> DirectClient<T> {
    pub fn new(
        rt: Arc<Runtime>,
        btc_client: Arc<bitcoincore_rpc::Client>,
        storage: T,
        base: CommonClient,
    ) -> Self {
        Self {
            rt,
            btc_client: Some(btc_client),
            storage,
            base,
        }
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor + Clone> Client for DirectClient<T> {
    async fn try_get_event(&self) -> IndexerResult<Option<ClientEvent>> {
        self.base.try_get_event().await
    }

    async fn push_event(&self, event: DispatchEvent) -> IndexerResult<()> {
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
    async fn report_reorg(&self, number: u32) -> IndexerResult<()> {
        self.base.report_reorg(number).await
    }

    async fn block_get_event(&self) -> IndexerResult<ClientEvent> {
        self.base.block_get_event().await
    }
}

impl<T: StorageProcessor + Clone> DirectClient<T> {
    pub fn get(&self) -> Vec<u8> {
        self.base.get()
    }
    pub fn block_get(&self) -> Vec<u8> {
        self.base.block_get()
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

    fn block_get_event(&self) -> IndexerResult<ClientEvent> {
        self.base.block_get_data()
    }

    fn report_height(&self, height: u32) -> IndexerResult<()> {
        self.base
            .tx
            .send_blocking(DispatchEvent::IndexerEvent(IndexerEvent::ReportHeight(
                height,
            )))?;
        Ok(())
    }

    fn report_reorg(&self, org_number: u32) -> IndexerResult<()> {
        self.base
            .tx
            .send_blocking(DispatchEvent::IndexerEvent(IndexerEvent::ReportReorg(
                org_number,
            )))?;
        Ok(())
    }

    fn push_event(&self, event: IndexerEvent) -> IndexerResult<()> {
        self.base
            .tx
            .send_blocking(DispatchEvent::IndexerEvent(event))?;
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
    ) -> IndexerResult<Vec<AllBalanceResponse>> {
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

    fn simple_set(&mut self, tx_id: &TxIdType, key: &[u8], value: Vec<u8>) -> IndexerResult<()> {
        Ok(self
            .rt
            .block_on(async { self.storage.simple_set(tx_id, key, value).await })?)
    }

    fn simple_get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        Ok(self
            .rt
            .block_on(async { self.storage.simple_get(key).await })?)
    }

    fn remove_tx_traces(&mut self, tx_id: Vec<TxIdType>) -> IndexerResult<()> {
        Ok(self
            .rt
            .block_on(async { self.storage.remove_tx_traces(tx_id).await })?)
    }

    fn get_btc_client(&self) -> Arc<bitcoincore_rpc::Client> {
        self.btc_client.clone().unwrap()
    }

    fn get_transaction_by_tx_id(&self, txid: Txid) -> IndexerResult<Option<Transaction>> {
        const NOT_EXIST_KEY: &str = "No such mempool or blockchain transaction";
        let client = self.btc_client.as_ref().unwrap();
        let ret = client.get_raw_transaction(&txid, None);
        match ret {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains(NOT_EXIST_KEY) {
                    Ok(None)
                } else {
                    Err(IndexerError::BitCoinClientError(e))
                }
            }
        }
    }
}
