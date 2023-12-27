use crate::types::delta::TransactionDelta;
use bitcoincore_rpc::bitcoin::Block;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BalanceType(pub bigdecimal::BigDecimal);

pub type AddressType = Vec<u8>;
pub type TokenType = Vec<u8>;

pub type TxIdType = String;

#[derive(Clone)]
pub struct BalanceDelta {}

#[derive(Clone)]
pub struct TxResultInfo {
    pub tx_hash: String,
}
// 需要维护的信息
// 1. 通过address 查询某个token的所有asset 余额
// 2. 通过address 查询全部的token 对应的余额
