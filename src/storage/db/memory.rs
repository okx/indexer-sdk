use crate::error::IndexerResult;
use crate::storage::db::DB;
use rusty_leveldb::WriteBatch;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct MemoryDB {
    datas: HashMap<Vec<u8>, Vec<u8>>,
}

impl DB for MemoryDB {
    fn set(&mut self, key: &[u8], value: &[u8]) -> IndexerResult<()> {
        self.datas.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        Ok(self.datas.get(key).cloned())
    }

    fn write_batch(&mut self, batch: WriteBatch, sync: bool) -> IndexerResult<()> {
        batch.iter().for_each(|(k, v)| {
            if v.is_none() {
                self.datas.remove(k);
            } else {
                let v = v.unwrap().to_vec();
                self.datas.insert(k.to_vec(), v);
            }
        });
        Ok(())
    }

    fn iter_all<KF, VF, K, V>(
        &mut self,
        prefix: &[u8],
        kf: KF,
        vf: VF,
    ) -> IndexerResult<Vec<(K, V)>>
    where
        KF: Fn(Vec<u8>) -> K,
        VF: Fn(Vec<u8>) -> Option<V>,
    {
        let mut ret = vec![];
        for (k, v) in self.datas.iter() {
            if k.starts_with(prefix) {
                let v = vf(v.clone());
                if v.is_some() {
                    ret.push((kf(k.clone()), v.unwrap()));
                }
            }
        }
        Ok(ret)
    }
}
