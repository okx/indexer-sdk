use crate::event::{AddressType, TokenType, TxIdType};

pub enum KeyPrefix {
    State,
    TransactionDelta,    // index -> TransactionWrapper
    TransactionIndexMap, // tx_id -> index
    AddressTokenBalance, // address|token -> balance
    SeenTx,              // tx_id -> timestamp
}
pub enum DeltaStatus {
    Active,
    Inactive,
}
impl DeltaStatus {
    pub fn to_u8(&self) -> u8 {
        match self {
            DeltaStatus::Active => 0,
            DeltaStatus::Inactive => 1,
        }
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => DeltaStatus::Active,
            1 => DeltaStatus::Inactive,
            _ => panic!("invalid delta status"),
        }
    }
}
impl KeyPrefix {
    pub fn get_prefix(&self) -> &[u8] {
        match self {
            KeyPrefix::State => b"state",
            KeyPrefix::TransactionDelta => b"td",
            KeyPrefix::AddressTokenBalance => b"atb",
            KeyPrefix::TransactionIndexMap => b"tim",
            KeyPrefix::SeenTx => b"s",
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
        ret.extend_from_slice(address.as_slice());
        ret.extend_from_slice(token_type.as_slice());
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
