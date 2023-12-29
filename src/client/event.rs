use crate::event::TxIdType;
use bitcoincore_rpc::bitcoin::consensus::serialize;
use bitcoincore_rpc::bitcoin::Transaction;

#[derive(Clone)]
pub enum ClientEvent {
    Transaction(Transaction),
    GetHeight,
    TxDroped(TxIdType),
    TxConfirmed(TxIdType),
}

impl ClientEvent {
    pub fn get_suffix(&self) -> u8 {
        match self {
            ClientEvent::Transaction(_) => 0,
            ClientEvent::GetHeight => 1,
            ClientEvent::TxDroped(_) => 2,
            ClientEvent::TxConfirmed(_) => 3,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: add suffix to distinguish different event
        match self {
            ClientEvent::Transaction(tx) => {
                let mut ret = serialize(tx);
                ret.push(self.get_suffix());
                ret
            }
            ClientEvent::GetHeight => {
                vec![self.get_suffix()]
            }
            ClientEvent::TxDroped(tx_id) => {
                let mut ret = tx_id.to_bytes();
                ret.push(self.get_suffix());
                ret
            }
            ClientEvent::TxConfirmed(tx_id) => {
                let mut ret = tx_id.to_bytes();
                ret.push(self.get_suffix());
                ret
            }
        }
    }
}
