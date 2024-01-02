use crate::error::IndexerResult;
use crate::event::TxIdType;
use crate::storage::db::DB;
use rusty_leveldb::WriteBatch;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct MemoryDB {
    datas: Rc<RefCell<HashMap<Vec<u8>, Vec<u8>>>>,

    tx_traces: Rc<RefCell<HashMap<TxIdType, Vec<Vec<u8>>>>>,
}

unsafe impl Send for MemoryDB {}
unsafe impl Sync for MemoryDB {}
impl DB for MemoryDB {
    fn set(&mut self, tx_id: Option<TxIdType>, key: &[u8], value: &[u8]) -> IndexerResult<()> {
        let mut data = self.datas.borrow_mut();
        let mut traces = self.tx_traces.borrow_mut();
        if let Some(tx_id) = tx_id {
            traces.entry(tx_id).or_insert(vec![]).push(key.to_vec());
        }
        data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        let data = self.datas.borrow_mut();
        Ok(data.get(key).cloned())
    }

    fn write_batch(
        &mut self,
        tx_id: Option<TxIdType>,
        batch: WriteBatch,
        _: bool,
    ) -> IndexerResult<()> {
        let mut data = self.datas.borrow_mut();
        let mut entry = vec![];
        batch.iter().for_each(|(k, v)| {
            entry.push(k.to_vec());
            if v.is_none() {
                data.remove(k);
            } else {
                let v = v.unwrap().to_vec();
                data.insert(k.to_vec(), v);
            }
        });
        let mut tx_traces = self.tx_traces.borrow_mut();
        if let Some(tx_id) = tx_id {
            tx_traces.entry(tx_id).or_insert(vec![]).append(&mut entry);
        }
        Ok(())
    }

    fn iter_all_mut<KF, VF, K, V>(
        &mut self,
        prefix: &[u8],
        mut kf: KF,
        mut vf: VF,
    ) -> IndexerResult<Vec<(K, V)>>
    where
        KF: FnMut(Vec<u8>) -> K,
        VF: FnMut(Vec<u8>) -> Option<V>,
    {
        let mut ret = vec![];
        let data = self.datas.borrow_mut();
        for (k, v) in data.iter() {
            if k.starts_with(prefix) {
                let v = vf(v.clone());
                if v.is_some() {
                    ret.push((kf(k.clone()), v.unwrap()));
                }
            }
        }
        Ok(ret)
    }

    fn remove_tx_traces(&mut self, tx_ids: Vec<TxIdType>) -> IndexerResult<()> {
        for tx_id in tx_ids {
            let traces = self.tx_traces.borrow_mut().remove(&tx_id);
            if let None = traces {
                return Ok(());
            }
            let mut data = self.datas.borrow_mut();
            let traces = traces.unwrap();
            for key in traces {
                data.remove(&key);
            }
        }

        Ok(())
    }
}
