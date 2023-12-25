use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use log::info;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::types::delta::TransactionDelta;


#[derive(Clone, Debug, Default)]
struct CacheComponent {
    address_balances: HashMap<AddressType, AddressBalance>,

    // avoid to iterator the address_balances
    tx_delta_cache: HashMap<TxIdType, Vec<TxDeltaNode>>,
}

#[derive(Clone, Debug, Default)]
struct AddressBalance {
    token_balances: HashMap<TokenType, BalanceWrapper>, // token 对应的余额信息
}

#[derive(Clone)]
pub struct TxDeltaNode {
    pub address: AddressType,
    pub delta: Vec<(TokenType, BalanceType)>,
}

// #[derive(Clone, Debug, Default)]
// struct TokenBalance {
//     total: BalanceWrapper,
//     tx_balances: HashMap<TxIdType, BalanceWrapper>, // tx_id 对应的增量信息
// }

type BalanceWrapper = Rc<RefCell<BalanceType>>;


impl CacheComponent {
    pub fn add_transaction_delta(&mut self, transaction: &TransactionDelta) {
        if transaction.deltas.is_empty() {
            return;
        }
        let mut nodes = HashMap::new();
        for (address, delta) in &transaction.deltas {
            let address_bal = self.address_balances.entry(address.clone()).or_insert_with(|| {
                AddressBalance::default()
            });
            for (token, delta) in delta {
                let bal = address_bal.token_balances.entry(token.clone()).or_insert_with(|| {
                    BalanceWrapper::default()
                });
                let total = bal.borrow_mut();
                total.0 = total.0 + delta.0;
                info!("add_transaction_delta,address:{:?},token:{:?},delta:{:?},total:{:?}",address,token,delta,total);

                let node = (token.clone(), BalanceType(delta.0));
                let trace_data = nodes.entry(address.clone()).or_insert_with(|| {
                    TxDeltaNode {
                        address: address.clone(),
                        delta: vec![],
                    }
                });
                trace_data.delta.push(node);
            }
        }
        let nodes = nodes.values().into_iter().map(|v| {
            v.clone()
        }).collect();
        self.tx_delta_cache.insert(transaction.tx_id.clone(), nodes);
    }

    // 1. 需要维护 tx 对应的delta 信息 (delta 信息内部是address + balance_type信息)
    pub fn remove_transaction_delta(&mut self, tx_id: &TxIdType) {
        let cache = self.tx_delta_cache.get(tx_id);
        if cache.is_none() {
            info!("tx_delta_cache:{:?} is none",tx_id);
            return;
        }
        let cache = cache.unwrap();
        for node in cache {
            let address = &node.address;
            for (token, delta) in &node.delta {
                self.decrease_address_delta(address, token, delta)
            }
        }
    }
    fn decrease_address_delta(&mut self, address: &AddressType, token_type: &TokenType, delta: &BalanceType) {
        let address_bal = self.address_balances.get_mut(address);
        if address_bal.is_none() {
            info!("address_bal:{:?} is none",address);
            return;
        }
        let address_bal = address_bal.unwrap();
        let token_bal = address_bal.token_balances.get_mut(token_type);
        if token_bal.is_none() {
            info!("token_bal:{:?} is none",token_type);
            return;
        }

        let token_bal = token_bal.unwrap();
        let total = token_bal.borrow_mut();
        total.0 = total.0 - delta.0;

        info!("decrease_address_delta,address:{:?},token:{:?},delta:{:?},total:{:?}",address,token_type,delta,total);
    }
}


#[derive(Clone)]
pub struct CacheNode<T: Clone> {
    pub index: u32,
    pub cache_value: T,
}
