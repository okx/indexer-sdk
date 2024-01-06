use crate::error::IndexerResult;
use crate::event::TxIdType;
use crate::storage::db::DB;
use rusty_leveldb::WriteBatch;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct ThreadSafeDB<T: DB + Clone> {
    lock: Arc<Mutex<T>>,
}

impl<T: DB + Clone> ThreadSafeDB<T> {
    pub fn new(internal: T) -> Self {
        Self {
            lock: Arc::new(Mutex::new(internal)),
        }
    }
}

impl<T: DB + Clone> DB for ThreadSafeDB<T> {
    fn set(&mut self, tx_id: Option<TxIdType>, key: &[u8], value: &[u8]) -> IndexerResult<()> {
        let mut lock = self.lock.lock().unwrap();
        lock.set(tx_id, key, value)
    }

    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        let mut lock = self.lock.lock().unwrap();
        lock.get(key)
    }

    fn delete(&mut self, key: &[u8]) -> IndexerResult<()> {
        let mut lock = self.lock.lock().unwrap();
        lock.delete(key)
    }

    fn write_batch(
        &mut self,
        tx_id: Option<TxIdType>,
        batch: WriteBatch,
        sync: bool,
    ) -> IndexerResult<()> {
        let mut lock = self.lock.lock().unwrap();
        lock.write_batch(tx_id, batch, sync)
    }

    fn iter_all_mut<KF, VF, K, V>(
        &mut self,
        prefix: &[u8],
        kf: KF,
        vf: VF,
    ) -> IndexerResult<Vec<(K, V)>>
    where
        KF: FnMut(Vec<u8>) -> K,
        VF: FnMut(Vec<u8>) -> Option<V>,
    {
        let mut lock = self.lock.lock().unwrap();
        lock.iter_all_mut(prefix, kf, vf)
    }

    fn remove_tx_traces(&mut self, tx_id: Vec<TxIdType>) -> IndexerResult<()> {
        let mut lock = self.lock.lock().unwrap();
        lock.remove_tx_traces(tx_id)
    }
}
