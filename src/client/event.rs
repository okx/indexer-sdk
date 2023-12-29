use crate::event::{AddressType, IndexerEvent, TokenType, TxIdType};
use crate::types::delta::TransactionDelta;
use bitcoincore_rpc::bitcoin::consensus::{deserialize, serialize};
use bitcoincore_rpc::bitcoin::Transaction;
use serde::{Deserialize, Serialize};

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
    pub fn from_bytes(data: &[u8]) -> ClientEvent {
        let suffix = data[data.len() - 1];
        match suffix {
            0 => {
                let tx: Transaction = deserialize(&data[0..data.len() - 1]).unwrap();
                ClientEvent::Transaction(tx)
            }
            1 => ClientEvent::GetHeight,
            2 => {
                let tx_id = TxIdType::from_bytes(&data[0..data.len() - 1]);
                ClientEvent::TxDroped(tx_id)
            }
            3 => {
                let tx_id = TxIdType::from_bytes(&data[0..data.len() - 1]);
                ClientEvent::TxConfirmed(tx_id)
            }
            _ => {
                panic!("unknown suffix:{}", suffix);
            }
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

#[derive(Clone, Debug)]
pub enum RequestEvent {
    GetBalance(AddressType, TokenType),
    GetAllBalance(AddressType),
    PushDelta(TransactionDelta),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddressTokenWrapper {
    pub address: AddressType,
    pub token: TokenType,
}

impl Into<Option<IndexerEvent>> for RequestEvent {
    fn into(self) -> Option<IndexerEvent> {
        match self {
            RequestEvent::GetBalance(address_type, token_type) => None,
            RequestEvent::GetAllBalance(address_type) => None,
            RequestEvent::PushDelta(delta) => Some(IndexerEvent::UpdateDelta(delta)),
        }
    }
}
impl RequestEvent {
    pub fn get_suffix(&self) -> u8 {
        match self {
            RequestEvent::GetBalance(_, _) => 0,
            RequestEvent::GetAllBalance(_) => 1,
            RequestEvent::PushDelta(_) => 2,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            RequestEvent::GetBalance(address_type, token_type) => {
                let mut ret = address_type.to_bytes();
                ret.extend_from_slice(&token_type.to_bytes());
                ret.push(self.get_suffix());
                ret
            }
            RequestEvent::GetAllBalance(address_type) => {
                let mut ret = address_type.to_bytes();
                ret.push(self.get_suffix());
                ret
            }
            RequestEvent::PushDelta(delta) => {
                let mut ret = serde_json::to_vec(delta).unwrap();
                ret.push(self.get_suffix());
                ret
            }
        }
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let suffix = data[data.len() - 1];
        match suffix {
            0 => {
                let address: AddressTokenWrapper =
                    serde_json::from_slice(&data[0..data.len() - 1]).unwrap();
                RequestEvent::GetBalance(address.address, address.token)
            }
            1 => {
                let address_type = AddressType::from_bytes(&data[0..data.len() - 1]);
                RequestEvent::GetAllBalance(address_type)
            }
            2 => {
                let delta: TransactionDelta =
                    serde_json::from_slice(&data[0..data.len() - 1]).unwrap();
                RequestEvent::PushDelta(delta)
            }
            _ => {
                panic!("unknown suffix:{}", suffix);
            }
        }
    }
}
