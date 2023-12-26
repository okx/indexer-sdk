use crate::event::{AddressType, BalanceType, TokenType};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TransactionDelta {
    pub tx_id: String,
    pub deltas: HashMap<AddressType, Vec<(TokenType, BalanceType)>>,
}
