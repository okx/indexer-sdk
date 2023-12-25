use std::sync::Arc;
use std::time::Duration;
use log::{error, info};
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TxIdType};
use crate::{Component, IndexProcessor};
use crate::configuration::base::IndexerConfiguration;
use crate::storage::{StorageProcessor};
use crate::types::delta::TransactionDelta;
use crate::types::response::{DataEnum, GetDataResponse};

#[derive(Clone, Debug)]
pub struct IndexerProcessorImpl {
    tx: crossbeam::channel::Sender<GetDataResponse>,
    storage: Arc<Box<dyn StorageProcessor>>,
}


impl IndexerProcessorImpl {
    pub fn new(tx: crossbeam::channel::Sender<GetDataResponse>, storage: Arc<Box<dyn StorageProcessor>>) -> Self {
        Self { tx, storage }
    }
}

#[async_trait::async_trait]
impl Component for IndexerProcessorImpl {
    type Event = IndexerEvent;
    type Configuration = IndexerConfiguration;
    type Inner = Self;

    fn inner(&mut self) -> &mut Self::Inner {
        unreachable!()
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(1)
    }

    async fn handle_tick_event(&mut self) -> IndexerResult<()> {
        Ok(())
    }

    async fn handle_event(&mut self, event: &Self::Event) -> IndexerResult<()> {
        if let Err(e) = self.do_handle_event(event).await {
            error!("handle_event error:{:?}",e)
        }
        Ok(())
    }
}

impl IndexerProcessorImpl {
    async fn do_handle_event(&mut self, event: &Self::Event) -> IndexerResult<()> {
        info!("do_handle_event,event:{:?}",event);
        match event {
            IndexerEvent::NewTxComing(data) => {
                self.do_handle_new_tx_coming(data).await
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
        }
        Ok(())
    }
    pub(crate) async fn do_handle_new_tx_coming(&self, data: &Vec<u8>) {
        let data = self.parse_zmq_data(&data);
        if let Some(data) = data {
            self.tx.send(data).unwrap();
        }
    }
    fn parse_zmq_data(&self, data: &Vec<u8>) -> Option<GetDataResponse> {
        Some(GetDataResponse {
            data_type: DataEnum::NewTx,
            data: data.clone(),
        })
    }

    pub(crate) async fn do_handle_get_balance(&self, address: &AddressType, tx: &crossbeam::channel::Sender<BalanceType>) -> IndexerResult<()> {
        let ret = self.storage.get_balance(address).await?;
        let _ = tx.send(ret);
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
impl IndexProcessor for IndexerProcessorImpl {}