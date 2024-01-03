use crate::event::TxIdType;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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

impl Hash for TxNode {
    // FIXME
    fn hash<H: Hasher>(&self, state: &mut H) {
        let data = serde_json::to_string(self).unwrap();
        data.hash(state);
    }
}
#[test]
pub fn test_asd() {}
