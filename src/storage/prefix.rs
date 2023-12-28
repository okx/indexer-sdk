use crate::event::{AddressType, TokenType, TxIdType};

pub enum KeyPrefix {
    State,
    TransactionDelta,    // index -> TransactionWrapper
    TransactionIndexMap, // tx_id -> index
    AddressTokenBalance, // address|token -> balance
    SeenTx,              // tx_id -> timestamp
}
pub enum DeltaStatus {
    Default,
    Executed,
    Confirmed,
    InActive,
}
impl DeltaStatus {
    pub fn to_u8(&self) -> u8 {
        match self {
            DeltaStatus::Default => 0,
            DeltaStatus::Executed => 1,
            DeltaStatus::Confirmed => 2,
            DeltaStatus::InActive => 3,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => DeltaStatus::Default,
            1 => DeltaStatus::Executed,
            2 => DeltaStatus::Confirmed,
            3 => DeltaStatus::InActive,
            _ => panic!("invalid delta status"),
        }
    }
}
impl KeyPrefix {
    pub fn get_prefix(&self) -> &[u8] {
        match self {
            KeyPrefix::State => b"a",
            KeyPrefix::TransactionDelta => b"b",
            KeyPrefix::AddressTokenBalance => b"c",
            KeyPrefix::TransactionIndexMap => b"d",
            KeyPrefix::SeenTx => b"e",
        }
    }
    pub fn get_suffix<'a>(&self, key: &'a [u8]) -> &'a [u8] {
        let prefix = self.get_prefix();
        &key[prefix.len()..]
    }
    pub fn build_prefix(&self, key: &[u8]) -> Vec<u8> {
        let mut ret = self.get_prefix().to_vec();
        ret.extend_from_slice(key);
        ret
    }

    pub fn build_transaction_index_map_key(tx_id: &TxIdType) -> Vec<u8> {
        let mut ret = Self::TransactionIndexMap.get_prefix().to_vec();
        ret.extend_from_slice(tx_id.to_bytes().as_slice());
        ret
    }
    pub fn build_transaction_data_key(index: u32) -> Vec<u8> {
        let mut ret = Self::TransactionDelta.get_prefix().to_vec();
        ret.extend_from_slice(&index.to_le_bytes());
        ret
    }
    pub fn build_state_key() -> Vec<u8> {
        Self::State.get_prefix().to_vec()
    }
    pub fn build_address_token_key(address: &AddressType, token_type: &TokenType) -> Vec<u8> {
        let mut ret = Self::AddressTokenBalance.get_prefix().to_vec();
        ret.extend_from_slice(address.to_bytes().as_slice());
        ret.extend_from_slice(token_type.to_bytes().as_slice());
        ret
    }
    pub fn build_seen_tx_key(tx_id: &TxIdType) -> Vec<u8> {
        let mut ret = Self::SeenTx.get_prefix().to_vec();
        ret.extend_from_slice(tx_id.to_bytes().as_slice());
        ret
    }
    pub fn get_tx_id_from_seen_key(key: &[u8]) -> TxIdType {
        let suffix = Self::SeenTx.get_suffix(key);
        let tx_id = TxIdType::from_bytes(suffix.try_into().unwrap());
        tx_id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum SeenStatus {
    UnExecuted,
    Executed,
}
pub const SEEN_DATA_STATUS_INDEX: usize = 8;
impl SeenStatus {
    pub fn to_u8(&self) -> u8 {
        match self {
            SeenStatus::UnExecuted => 0,
            SeenStatus::Executed => 1,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => SeenStatus::UnExecuted,
            1 => SeenStatus::Executed,
            _ => panic!("invalid seen status"),
        }
    }
}
