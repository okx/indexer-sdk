use crate::error::IndexerResult;
use crate::event::TxIdType;
use crate::storage::db::DB;
use crate::storage::prefix::KeyPrefix;
use rusty_leveldb::{LdbIterator, WriteBatch};
use std::cell::RefCell;
use std::rc::Rc;

pub struct LevelDB {
    db: Rc<RefCell<rusty_leveldb::DB>>,
}

unsafe impl Send for LevelDB {}
unsafe impl Sync for LevelDB {}

impl Clone for LevelDB {
    fn clone(&self) -> Self {
        LevelDB {
            db: self.db.clone(),
        }
    }
}
impl LevelDB {
    pub fn new(path: &str) -> IndexerResult<Self> {
        let db = Rc::new(RefCell::new(
            rusty_leveldb::DB::open(path, rusty_leveldb::Options::default()).unwrap(),
        ));
        Ok(LevelDB { db })
    }
}

impl Default for LevelDB {
    fn default() -> Self {
        let db = Rc::new(RefCell::new(
            rusty_leveldb::DB::open("./indexer_sdk_db", rusty_leveldb::Options::default()).unwrap(),
        ));
        LevelDB { db }
    }
}
impl DB for LevelDB {
    fn set(&mut self, tx_id: Option<TxIdType>, key: &[u8], value: &[u8]) -> IndexerResult<()> {
        let mut db = self.db.borrow_mut();
        db.put(key, value)?;

        if let Some(tx_id) = tx_id {
            let trace_key = KeyPrefix::build_tx_key_trace(&tx_id, key);
            db.put(&trace_key, &[])?;
        }
        db.flush()?;
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        let mut db = self.db.borrow_mut();
        Ok(db.get(key))
    }

    fn write_batch(
        &mut self,
        tx_id: Option<TxIdType>,
        batch: WriteBatch,
        sync: bool,
    ) -> IndexerResult<()> {
        let mut db = self.db.borrow_mut();
        // better way?
        let mut entry = vec![];
        if let Some(tx_id) = tx_id {
            batch.iter().for_each(|(k, _)| {
                entry.push((KeyPrefix::build_tx_key_trace(&tx_id, k), &[]));
                // new_batch.put(k, v) foreach once?
            });
        }
        let mut new_batch = WriteBatch::from(batch);
        if !entry.is_empty() {
            for (k, v) in entry {
                new_batch.put(&k, v);
            }
        }

        db.write(new_batch, sync)?;
        db.flush()?;
        Ok(())
    }

    // FIXME:BAD CODE
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
        let mut db = self.db.borrow_mut();
        let mut iter = db.new_iter()?;
        iter.seek(prefix);

        let mut ret = vec![];
        let v = current_key_val(&iter);
        if let Some((k, v)) = v {
            let value = vf(v);
            let key = kf(k);
            if value.is_some() {
                ret.push((key, value.unwrap()))
            }
        }
        loop {
            if !iter.valid() {
                return Ok(ret);
            }
            let next = iter.next();
            if next.is_none() {
                return Ok(ret);
            }
            let (k, v) = next.unwrap();
            if !k.starts_with(prefix) {
                return Ok(ret);
            }
            let key = kf(k);
            let value = vf(v);
            if value.is_some() {
                ret.push((key, value.unwrap()))
            }
        }
    }

    fn remove_tx_traces(&mut self, tx_id: Vec<TxIdType>) -> IndexerResult<()> {
        if tx_id.is_empty() {
            return Ok(());
        }
        let mut batch = WriteBatch::new();
        for tx_id in tx_id {
            let prefix = KeyPrefix::interator_tx_key_prefix(&tx_id);
            self.iter_all_mut(
                &prefix,
                |k| {
                    batch.delete(&k);
                },
                |_| None::<Vec<u8>>,
            )?;
        }
        let mut db = self.db.borrow_mut();
        db.write(batch, true)?;
        db.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    pub fn test_rm_tx_traces() {
        let mut db = LevelDB::new("./test_rm_tx_traces").unwrap();

        let mut batch = WriteBatch::new();
        let couple = vec![
            (b"abc".to_vec(), b"def".to_vec()),
            (b"123".to_vec(), b"456".to_vec()),
            (b"xxx".to_vec(), b"yyy".to_vec()),
            (b"zzz".to_vec(), b"".to_vec()),
            (b"010".to_vec(), b"".to_vec()),
        ];
        for (k, v) in couple.clone() {
            batch.put(&k, &v);
        }

        let tx_id = TxIdType::from_bytes(&[0u8; 32]);
        db.write_batch(Some(tx_id.clone()), batch, true).unwrap();

        let prefix = KeyPrefix::interator_tx_key_prefix(&tx_id);
        let couples = db.iter_all_mut(&prefix, |k| k, |v| Some(v)).unwrap();
        assert_eq!(couples.len(), couple.len());
        let mut data: HashMap<Vec<u8>, Vec<u8>> = HashMap::default();
        for (k, _) in &couples {
            let (_, kk) = KeyPrefix::split_tx_key_trace(k);
            let vv = db.get(&kk).unwrap().unwrap();
            data.insert(kk, vv);
        }
        for (k, v) in couple {
            assert_eq!(data.get(&k), Some(&v));
        }
    }
}

fn current_key_val<It: LdbIterator + ?Sized>(it: &It) -> Option<(Vec<u8>, Vec<u8>)> {
    let (mut k, mut v) = (vec![], vec![]);
    if it.current(&mut k, &mut v) {
        Some((k, v))
    } else {
        None
    }
}
