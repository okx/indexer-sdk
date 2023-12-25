use primitive_types::U256;

#[derive(Clone, Debug)]
pub enum IndexerEvent {
    NewTxComing(Vec<u8>),

    GetBalance(AddressType, crossbeam::channel::Sender<BalanceType>),
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