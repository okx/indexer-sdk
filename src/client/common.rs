use crate::client::Client;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TokenType, TxIdType};
use crate::types::delta::TransactionDelta;
use crate::types::response::GetDataResponse;
use crossbeam::channel::{Receiver, TryRecvError};
use log::info;

#[repr(C)]
#[derive(Clone)]
pub struct CommonClient {
    rx: Receiver<GetDataResponse>,
    tx: async_channel::Sender<IndexerEvent>,
}

impl Default for CommonClient {
    fn default() -> Self {
        let (tx, _) = async_channel::unbounded();
        let (_, rx) = crossbeam::channel::unbounded();
        Self { rx, tx }
    }
}

#[async_trait::async_trait]
impl Client for CommonClient {
    async fn get_data(&self) -> IndexerResult<Option<GetDataResponse>> {
        self.do_get_data()
    }

    async fn push_data(&self, data: Vec<u8>) -> IndexerResult<()> {
        let data: TransactionDelta = serde_json::from_slice(data.as_slice()).unwrap();
        self.tx.send(IndexerEvent::UpdateDelta(data)).await.unwrap();
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
}

impl CommonClient {
    pub fn new(rx: Receiver<GetDataResponse>, tx: async_channel::Sender<IndexerEvent>) -> Self {
        Self { rx, tx }
    }

    fn do_tx_consumed(&mut self, tx_id: TxIdType) -> IndexerResult<()> {
        self.tx
            .send_blocking(IndexerEvent::TxConsumed(tx_id))
            .unwrap();
        Ok(())
    }
    fn do_get_balance(
        &self,
        address: AddressType,
        token_type: TokenType,
    ) -> IndexerResult<BalanceType> {
        let (tx, rx) = crossbeam::channel::bounded(1);
        self.tx
            .send_blocking(IndexerEvent::GetBalance(address, tx))
            .unwrap();
        let ret = rx.recv().unwrap();
        Ok(ret)
    }
    fn do_update_delta(&self, delta: TransactionDelta) -> IndexerResult<()> {
        self.tx
            .send_blocking(IndexerEvent::UpdateDelta(delta))
            .unwrap();
        Ok(())
    }
    fn do_get_data(&self) -> IndexerResult<Option<GetDataResponse>> {
        let res = self.rx.try_recv();
        return match res {
            Ok(ret) => {
                info!("get data from channel");
                Ok(Some(ret))
            }
            Err(v) => {
                match v {
                    TryRecvError::Empty => {
                        return Ok(None);
                    }
                    TryRecvError::Disconnected => {}
                }
                Ok(None)
            }
        };
    }

    pub fn get(&self) -> Vec<u8> {
        let data = self.do_get_data().unwrap();
        if data.is_none() {
            return vec![];
        }
        let data = data.unwrap();
        if data.data.is_empty() {
            return vec![];
        }
        let mut ret = data.data.clone();
        ret.push(data.data_type.to_u8());
        ret
    }
}
