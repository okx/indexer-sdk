use crate::types::delta::TransactionDelta;
use bitcoincore_rpc::bitcoin::{Block, Txid};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

#[derive(Clone)]
pub enum IndexerEvent {
    NewTxComing(Vec<u8>, u32),
    NewTxComingByTxId(TxIdType),

    RawBlockComing(Block, u32),

    GetBalance(AddressType, crossbeam::channel::Sender<BalanceType>),

    UpdateDelta(TransactionDelta),
    TxConsumed(TxIdType),
}

impl Debug for IndexerEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexerEvent::NewTxComing(_, _) => {
                write!(f, "NewTxComing")
            }
            IndexerEvent::GetBalance(_, _) => {
                write!(f, "GetBalance")
            }
            IndexerEvent::UpdateDelta(_) => {
                write!(f, "UpdateDelta")
            }
            IndexerEvent::TxConsumed(_) => {
                write!(f, "TxConsumed")
            }
            IndexerEvent::RawBlockComing(_, _) => {
                write!(f, "RawBlockComing")
            }
            IndexerEvent::NewTxComingByTxId(_) => {
                write!(f, "NewTxComingByTxId")
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct BalanceType(pub bigdecimal::BigDecimal);

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct AddressType(pub Vec<u8>);
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct TokenType(pub Vec<u8>);

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct TxIdType(pub String);

impl AddressType {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}
impl TokenType {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}
impl TxIdType {
    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(&self.0).unwrap()
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(hex::encode(bytes))
    }
}
impl From<Txid> for TxIdType {
    fn from(value: Txid) -> Self {
        Self(hex::encode(&value))
    }
}
impl Into<Txid> for TxIdType {
    fn into(self) -> Txid {
        Txid::from_str(&self.0).unwrap()
    }
}
#[derive(Clone)]
pub struct BalanceDelta {}

#[derive(Clone)]
pub struct TxResultInfo {
    pub tx_hash: String,
}
// 需要维护的信息
// 1. 通过address 查询某个token的所有asset 余额
// 2. 通过address 查询全部的token 对应的余额
