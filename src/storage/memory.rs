use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use log::info;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, TokenType, TxIdType};
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;


#[derive(Clone, Debug, Default)]
pub struct MemoryStorageProcessor {
    address_balances: HashMap<AddressType, AddressBalance>,

    // avoid to iterator the address_balances
    tx_delta_cache: HashMap<TxIdType, Vec<TxDeltaNode>>,
}

unsafe impl Send for MemoryStorageProcessor {}

unsafe impl Sync for MemoryStorageProcessor {}

#[derive(Clone, Debug, Default)]
struct AddressBalance {
    token_balances: HashMap<TokenType, BalanceWrapper>, // token 对应的余额信息
}

#[derive(Clone, Debug)]
pub struct TxDeltaNode {
    pub address: AddressType,
    pub delta: Vec<(TokenType, BalanceType)>,
}

type BalanceWrapper = Rc<RefCell<BalanceType>>;


#[async_trait::async_trait]
impl StorageProcessor for MemoryStorageProcessor {
    async fn get_balance(&self, address: &AddressType) -> IndexerResult<BalanceType> {
        todo!()
    }

    async fn add_transaction_delta(&mut self, transaction: &TransactionDelta) -> IndexerResult<()> {
        if transaction.deltas.is_empty() {
            return Ok(());
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
                let mut total = bal.borrow_mut();
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
        Ok(())
    }

    // 1.  tx -> delta info (delta:address + balance_type)
    async fn remove_transaction_delta(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        let cache = self.tx_delta_cache.get(tx_id);
        if cache.is_none() {
            info!("tx_delta_cache:{:?} is none",tx_id);
            return Ok(());
        }
        let cache = cache.unwrap();
        for node in cache {
            let address = &node.address;
            for (token, delta) in &node.delta {
                // self.decrease_address_delta(address, token, delta)
                let address_bal = self.address_balances.get_mut(address);
                if address_bal.is_none() {
                    info!("address_bal:{:?} is none",address);
                    return Ok(());
                }
                let address_bal = address_bal.unwrap();
                let token_bal = address_bal.token_balances.get_mut(token);
                if token_bal.is_none() {
                    info!("token_bal:{:?} is none",token);
                    return Ok(());
                }

                let token_bal = token_bal.unwrap();
                let mut total = token_bal.borrow_mut();
                total.0 = total.0 - delta.0;

                info!("decrease_address_delta,address:{:?},token:{:?},delta:{:?},total:{:?}",address,token,delta,total);
            }
        }
        Ok(())
    }
}

impl MemoryStorageProcessor {
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
        let mut total = token_bal.borrow_mut();
        total.0 = total.0 - delta.0;

        info!("decrease_address_delta,address:{:?},token:{:?},delta:{:?},total:{:?}",address,token_type,delta,total);
    }
}

#[derive(Clone)]
pub struct CacheNode<T: Clone> {
    pub index: u32,
    pub cache_value: T,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_asd() {}
}