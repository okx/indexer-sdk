use std::fmt::{Debug, Formatter};
use primitive_types::U256;
use crate::types::delta::TransactionDelta;

#[derive(Clone)]
pub enum IndexerEvent {
    NewTxComing(Vec<u8>),

    GetBalance(AddressType, crossbeam::channel::Sender<BalanceType>),

    UpdateDelta(TransactionDelta),
    TxConsumed(TxIdType),
}

impl Debug for IndexerEvent{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexerEvent::NewTxComing(data) => {
                f.debug_struct("NewTxComing")
                    .field("data", &data)
                    .finish()
            }
            IndexerEvent::GetBalance(address, tx) => {
                f.debug_struct("GetBalance")
                    .field("address", &address)
                    .field("tx", &tx)
                    .finish()
            }
            IndexerEvent::UpdateDelta(delta) => {
                f.debug_struct("UpdateDelta")
                    .field("delta", &delta)
                    .finish()
            }
            IndexerEvent::TxConsumed(tx_id) => {
                f.debug_struct("TxConsumed")
                    .field("tx_id", &tx_id)
                    .finish()
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BalanceType(pub U256);

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