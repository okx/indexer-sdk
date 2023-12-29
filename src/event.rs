use crate::types::delta::TransactionDelta;
use bitcoincore_rpc::bitcoin::hashes::Hash;
use bitcoincore_rpc::bitcoin::{Block, Txid};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

#[derive(Clone)]
pub enum IndexerEvent {
    NewTxComing(Vec<u8>, u32),
    TxFromRestoreByTxId(TxIdType),

    // RawBlockComing(Block, u32),
    GetBalance(AddressType, crossbeam::channel::Sender<BalanceType>),

    UpdateDelta(TransactionDelta),

    TxRemoved(TxIdType),

    TxConfirmed(TxIdType),

    ReportHeight(u32),

    ReportReorg(Vec<TxIdType>),
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
            IndexerEvent::TxConfirmed(v) => {
                write!(f, "TxConfirmed :{:?}", v)
            }
            // IndexerEvent::RawBlockComing(_, _) => {
            //     write!(f, "RawBlockComing")
            // }
            IndexerEvent::TxFromRestoreByTxId(v) => {
                write!(f, "TxFromRestoreByTxId:{:?}", v)
            }
            IndexerEvent::TxRemoved(v) => {
                write!(f, "TxRemoved: {}", v.0)
            }
            IndexerEvent::ReportHeight(v) => {
                write!(f, "ReportHeight: {}", v)
            }
            IndexerEvent::ReportReorg(v) => {
                write!(f, "ReportReorg: {:?}", v)
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct BalanceType(pub bigdecimal::BigDecimal);

impl From<i32> for BalanceType {
    fn from(value: i32) -> Self {
        BalanceType(bigdecimal::BigDecimal::from(value))
    }
}
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct AddressType(pub Vec<u8>);

// TODO
impl Serialize for AddressType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = hex::encode(&self.0);
        String::serialize(&data, serializer)
    }
}
impl<'de> Deserialize<'de> for AddressType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded: String = Deserialize::deserialize(deserializer)?;
        Ok(AddressType(hex::decode(encoded).unwrap()))
    }
}

impl Serialize for TokenType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = hex::encode(&self.0);
        String::serialize(&data, serializer)
    }
}
impl<'de> Deserialize<'de> for TokenType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded: String = Deserialize::deserialize(deserializer)?;
        Ok(TokenType(hex::decode(encoded).unwrap()))
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct TokenType(pub Vec<u8>);

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct TxIdType(pub String);

impl AddressType {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
}
impl TokenType {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(data.to_vec())
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
        Self(value.to_string())
    }
}
impl From<String> for TxIdType {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl Into<Txid> for TxIdType {
    fn into(self) -> Txid {
        // let tx = hex::decode(&self.0).unwrap();
        Txid::from_str(&self.0).unwrap()
    }
}

#[derive(Clone)]
pub struct TxResultInfo {
    pub tx_hash: String,
}
