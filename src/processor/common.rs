use std::sync::{Arc};
use std::time::Duration;
use bitcoincore_rpc::bitcoin::consensus::deserialize;
use bitcoincore_rpc::bitcoin::Transaction;
use bitcoincore_rpc::RpcApi;
use log::{error, info};
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TxIdType};
use crate::{Component, IndexProcessor};
use crate::configuration::base::IndexerConfiguration;
use crate::storage::{StorageProcessor};
use crate::types::delta::TransactionDelta;
use crate::types::response::{DataEnum, GetDataResponse};

#[derive(Clone)]
pub struct IndexerProcessorImpl<T: StorageProcessor> {
    tx: crossbeam::channel::Sender<GetDataResponse>,
    storage: T,
    btc_client: Arc<bitcoincore_rpc::Client>,
}

unsafe impl<T: StorageProcessor> Send for IndexerProcessorImpl<T> {}

unsafe impl<T: StorageProcessor> Sync for IndexerProcessorImpl<T> {}

impl<T: StorageProcessor> IndexerProcessorImpl<T> {
    pub fn new(tx: crossbeam::channel::Sender<GetDataResponse>, storage: T, client: bitcoincore_rpc::Client) -> Self {
        Self { tx, storage, btc_client: Arc::new(client) }
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor> Component for IndexerProcessorImpl<T> {
    type Event = IndexerEvent;
    type Configuration = IndexerConfiguration;
    type Inner = Self;

    fn inner(&mut self) -> &mut Self::Inner {
        unreachable!()
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(10)
    }

    async fn handle_tick_event(&mut self) -> IndexerResult<()> {
        self.do_handle_sync_mempool().await?;
        Ok(())
    }

    async fn handle_event(&mut self, event: &Self::Event) -> IndexerResult<()> {
        if let Err(e) = self.do_handle_event(event).await {
            error!("handle_event error:{:?}",e)
        }
        Ok(())
    }
}

impl<T: StorageProcessor> IndexerProcessorImpl<T> {
    async fn do_handle_sync_mempool(&mut self) -> IndexerResult<()> {
        let txs = self.btc_client.get_raw_mempool()?;
        for tx in txs {
            // FIXME : to_string maybe is not right
            let tx_id: TxIdType = tx.to_string();
            // TODO: notify other component to load raw tx
        }
        Ok(())
    }
    async fn do_handle_event(&mut self, event: &IndexerEvent) -> IndexerResult<()> {
        info!("do_handle_event,event:{:?}",event);
        match event {
            IndexerEvent::NewTxComing(data, sequence) => {
                self.do_handle_new_tx_coming(data).await?;
            }
            IndexerEvent::GetBalance(address, tx) => {
                self.do_handle_get_balance(address, tx).await?;
            }
            IndexerEvent::UpdateDelta(data) => {
                self.do_handle_update_delta(data).await?;
            }
            IndexerEvent::TxConsumed(tx_id) => {
                self.do_handle_tx_consumed(tx_id).await?;
            }
            IndexerEvent::RawBlockComing(_, _) => {}
        }
        Ok(())
    }
    pub(crate) async fn do_handle_new_tx_coming(&mut self, data: &Vec<u8>) -> IndexerResult<()> {
        let data = self.parse_zmq_data(&data);
        if let Some((tx_id, data)) = data {
            if self.storage.seen_and_store_txs(tx_id.clone()).await? {
                info!("tx_id:{:?} has been seen",tx_id);
                return Ok(());
            }
            self.tx.send(data).unwrap();
        }
        Ok(())
    }
    fn parse_zmq_data(&self, data: &Vec<u8>) -> Option<(TxIdType, GetDataResponse)> {
        let tx: Transaction = deserialize(&data).expect("Failed to deserialize transaction");
        // TODO:
        Some((tx.txid().to_string(), GetDataResponse {
            data_type: DataEnum::NewTx,
            data: data.clone(),
        }))
    }

    pub(crate) async fn do_handle_get_balance(&self, address: &AddressType, tx: &crossbeam::channel::Sender<BalanceType>) -> IndexerResult<()> {
        let ret = self.storage.get_balance(address).await?;
        let _ = tx.send(ret);
        Ok(())
    }
    async fn do_handle_update_delta(&mut self, data: &TransactionDelta) -> IndexerResult<()> {
        self.storage.add_transaction_delta(data).await?;
        Ok(())
    }
    async fn do_handle_tx_consumed(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        self.storage.remove_transaction_delta(tx_id).await?;
        Ok(())
    }
}


#[async_trait::async_trait]
impl<T: StorageProcessor> IndexProcessor for IndexerProcessorImpl<T> {}
