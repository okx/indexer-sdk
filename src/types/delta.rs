use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransactionDelta {
    pub tx_id: TxIdType,
    pub deltas: HashMap<AddressType, Vec<(TokenType, BalanceType)>>,
}
