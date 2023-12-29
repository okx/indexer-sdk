use crate::types::delta::TransactionDelta;
use bigdecimal::num_bigint::{BigInt, ToBigInt};
use bigdecimal::num_traits::FromBytes;
use bigdecimal::num_traits::ToBytes;
use bitcoincore_rpc::bitcoin::consensus::{deserialize, serialize};
use bitcoincore_rpc::bitcoin::hashes::Hash;
use bitcoincore_rpc::bitcoin::{Block, Txid};
use primitive_types::U256;
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
impl IndexerEvent {
    pub fn get_suffix(&self) -> u8 {
        match self {
            IndexerEvent::NewTxComing(_, _) => 0,
            IndexerEvent::GetBalance(_, _) => 1,
            IndexerEvent::UpdateDelta(_) => 2,
            IndexerEvent::TxConfirmed(_) => 3,
            // IndexerEvent::RawBlockComing(_, _) => 4,
            IndexerEvent::TxFromRestoreByTxId(_) => 5,
            IndexerEvent::TxRemoved(_) => 6,
            IndexerEvent::ReportHeight(_) => 7,
            IndexerEvent::ReportReorg(_) => 8,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = match self {
            IndexerEvent::UpdateDelta(tx) => {
                let mut data = serde_json::to_vec(&tx).unwrap();
                data
            }
            IndexerEvent::TxConfirmed(tx_id) => {
                let mut data = tx_id.to_bytes();
                data
            }
            IndexerEvent::TxFromRestoreByTxId(tx_id) => {
                let mut data = tx_id.to_bytes();
                data
            }
            IndexerEvent::TxRemoved(tx_id) => {
                let mut data = tx_id.to_bytes();
                data
            }
            IndexerEvent::ReportHeight(height) => {
                let mut data = height.to_le_bytes().to_vec();
                data
            }
            IndexerEvent::ReportReorg(txs) => {
                let mut data = serde_json::to_vec(&txs).unwrap();
                data
            }
            _ => {
                panic!("not support");
            }
        };
        data.push(self.get_suffix());
        data
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let suffix = data[data.len() - 1];
        match suffix {
            0 => {
                let tx: TransactionDelta =
                    serde_json::from_slice(&data[0..data.len() - 1]).unwrap();
                IndexerEvent::UpdateDelta(tx)
            }
            2 => {
                let tx_id = TxIdType::from_bytes(&data[0..data.len() - 1]);
                IndexerEvent::TxConfirmed(tx_id)
            }
            5 => {
                let tx_id = TxIdType::from_bytes(&data[0..data.len() - 1]);
                IndexerEvent::TxRemoved(tx_id)
            }
            6 => {
                let height = u32::from_be_bytes(data[0..data.len() - 1].try_into().unwrap());
                IndexerEvent::ReportHeight(height)
            }
            7 => {
                let tx_ids = serde_json::from_slice(&data[0..data.len() - 1]).unwrap();
                IndexerEvent::ReportReorg(tx_ids)
            }
            _ => {
                panic!("unknown suffix:{}", suffix);
            }
        }
    }
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
impl BalanceType {
    pub fn to_bytes(&self) -> Vec<u8> {
        let bg = self.0.to_bigint().unwrap();
        bg.to_le_bytes().to_vec()
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        let bg = BigInt::from_le_bytes(data);
        BalanceType(bigdecimal::BigDecimal::from(bg))
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
