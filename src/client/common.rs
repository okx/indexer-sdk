use crate::client::event::ClientEvent;
use crate::client::Client;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TokenType, TxIdType};
use crate::types::delta::TransactionDelta;
use log::debug;

#[repr(C)]
#[derive(Clone)]
pub struct CommonClient {
    pub(crate) rx: async_channel::Receiver<ClientEvent>,
    pub(crate) tx: async_channel::Sender<IndexerEvent>,
}

impl Default for CommonClient {
    fn default() -> Self {
        let (tx, _) = async_channel::unbounded();
        let (_, rx) = async_channel::unbounded();
        Self { rx, tx }
    }
}

#[async_trait::async_trait]
impl Client for CommonClient {
    async fn get_event(&self) -> IndexerResult<Option<ClientEvent>> {
        self.do_get_data()
    }

    async fn push_event(&self, event: IndexerEvent) -> IndexerResult<()> {
        self.tx.send(event).await.unwrap();
        Ok(())
    }

    async fn get_balance(
        &mut self,
        address_type: AddressType,
        token_type: TokenType,
    ) -> IndexerResult<BalanceType> {
        self.do_get_balance(address_type, token_type)
    }

    async fn update_delta(&mut self, result: TransactionDelta) -> IndexerResult<()> {
        self.do_update_delta(result)
    }
    fn rx(&self) -> async_channel::Receiver<ClientEvent> {
        self.rx.clone()
    }

    async fn report_height(&self, height: u32) -> IndexerResult<()> {
        self.tx
            .send_blocking(IndexerEvent::ReportHeight(height))
            .unwrap();
        Ok(())
    }
    async fn report_reorg(&self, txs: Vec<TxIdType>) -> IndexerResult<()> {
        self.tx
            .send_blocking(IndexerEvent::ReportReorg(txs))
            .unwrap();
        Ok(())
    }
}

impl CommonClient {
    pub fn new(
        rx: async_channel::Receiver<ClientEvent>,
        tx: async_channel::Sender<IndexerEvent>,
    ) -> Self {
        Self { rx, tx }
    }

    pub(crate) fn do_get_balance(
        &self,
        address: AddressType,
        _: TokenType,
    ) -> IndexerResult<BalanceType> {
        let (tx, rx) = crossbeam::channel::bounded(1);
        self.tx
            .send_blocking(IndexerEvent::GetBalance(address, tx))
            .unwrap();
        let ret = rx.recv().unwrap();
        Ok(ret)
    }
    pub(crate) fn do_update_delta(&self, delta: TransactionDelta) -> IndexerResult<()> {
        self.tx
            .send_blocking(IndexerEvent::UpdateDelta(delta))
            .unwrap();
        Ok(())
    }
    pub(crate) fn do_get_data(&self) -> IndexerResult<Option<ClientEvent>> {
        let res = self.rx.try_recv();
        return match res {
            Ok(ret) => Ok(Some(ret)),
            Err(_) => Ok(None),
        };
    }
    pub(crate) fn block_get_data(&self) -> IndexerResult<Option<ClientEvent>> {
        let res = self.rx.recv_blocking();
        return match res {
            Ok(ret) => Ok(Some(ret)),
            Err(_) => Ok(None),
        };
    }

    pub fn sync_push_event(&self, event: IndexerEvent) {
        self.tx.send_blocking(event).unwrap();
    }

    pub fn get(&self) -> Vec<u8> {
        let data = self.do_get_data().unwrap();
        debug!("get event:{:?}", &data);
        if data.is_none() {
            return vec![];
        }
        let data = data.unwrap();
        let raw_data = data.to_bytes();
        raw_data
    }

    pub fn block_get(&self) -> Vec<u8> {
        let data = self.block_get_data().unwrap();
        debug!("get event:{:?}", &data);
        if data.is_none() {
            return vec![];
        }
        let data = data.unwrap();
        let raw_data = data.to_bytes();
        raw_data
    }
}
