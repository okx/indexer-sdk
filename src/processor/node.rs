use crate::event::TxIdType;
use std::collections::HashSet;

#[derive(Clone)]
pub struct TxNode {
    pub(crate) current_hash: TxIdType,
    pub(crate) nexts: HashSet<TxNode>,
}

impl TxNode {
    pub fn new(current_hash: TxIdType) -> Self {
        Self {
            current_hash,
            nexts: Default::default(),
        }
    }
}
