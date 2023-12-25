use std::collections::HashMap;
use crate::event::{AddressType, BalanceType, TokenType};

#[derive(Clone)]
pub struct TransactionDelta {
    pub tx_id: String,
    pub deltas: HashMap<AddressType, Vec<(TokenType, BalanceType)>>,
}