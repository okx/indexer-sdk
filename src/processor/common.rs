use std::time::Duration;
use log::info;
use crate::error::IndexerResult;
use crate::event::{AddressType, IndexerEvent};
use crate::{Component, IndexProcessor};
use crate::configuration::base::IndexerConfiguration;
use crate::notifier::internal_safe::InternalSafeChannel;

#[derive(Clone, Debug)]
pub struct IndexerProcessorImpl {
    tx: InternalSafeChannel<Vec<u8>>,


}


impl IndexerProcessorImpl {
    pub fn new(tx: InternalSafeChannel<Vec<u8>>) -> Self {
        Self { tx }
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
        info!("handle_event,event:{:?}",event);
        match event {
            IndexerEvent::NewTxComing(data) => {
                self.do_handle_new_tx_coming(data).await
            }
            IndexerEvent::GetBalance(address, _) => {
                self.do_handle_get_balance(address).await
            }
        }
        Ok(())
    }
}

impl IndexerProcessorImpl {
    pub(crate) async fn do_handle_new_tx_coming(&self, data: &Vec<u8>) {
        self.tx.send(data.clone());
    }
    pub(crate) async fn do_handle_get_balance(&self, address: &AddressType) {
        todo!()
    }
}


#[async_trait::async_trait]
impl IndexProcessor for IndexerProcessorImpl {}