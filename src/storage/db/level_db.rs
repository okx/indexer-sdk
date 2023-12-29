use crate::error::IndexerResult;
use crate::storage::db::DB;
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

impl Default for LevelDB {
    fn default() -> Self {
        let db = Rc::new(RefCell::new(
            rusty_leveldb::DB::open("./db", rusty_leveldb::Options::default()).unwrap(),
        ));
        LevelDB { db }
    }
}
impl LevelDB {
    pub fn new(path: &str) -> Self {
        let db = Rc::new(RefCell::new(
            rusty_leveldb::DB::open(path, rusty_leveldb::Options::default()).unwrap(),
        ));
        LevelDB { db }
    }
}
impl DB for LevelDB {
    fn set(&mut self, key: &[u8], value: &[u8]) -> IndexerResult<()> {
        let mut db = self.db.borrow_mut();
        db.put(key, value)?;
        db.flush()?;
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        let mut db = self.db.borrow_mut();
        Ok(db.get(key))
    }

    fn write_batch(&mut self, batch: WriteBatch, sync: bool) -> IndexerResult<()> {
        let mut db = self.db.borrow_mut();
        db.write(batch, sync)?;
        db.flush()?;
        Ok(())
    }

    // FIXME:BAD CODE
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_iter() {
        let mut db = LevelDB::default();

        let key = vec![0, 1, 2];

        {
            db.set(&vec![10, 11], 999u32.to_le_bytes().as_slice())
                .unwrap();
        }

        {
            let mut key1 = key.clone();
            key1.extend_from_slice(&[1, 2]);
            db.set(key1.as_slice(), 1u32.to_le_bytes().as_slice())
                .unwrap();
        }

        {
            let mut key2 = key.clone();
            key2.extend_from_slice(&[3, 4]);
            db.set(key2.as_slice(), 2u32.to_le_bytes().as_slice())
                .unwrap();
        }

        {
            let mut key3 = key.clone();
            key3.extend_from_slice(&[5, 6]);
            db.set(key3.as_slice(), 3u32.to_le_bytes().as_slice())
                .unwrap();
        }

        db.iter_all(
            vec![10, 11].as_slice(),
            |v| Some(v),
            |v| {
                let v = u32::from_le_bytes(v.as_slice().try_into().unwrap());
                print!("{} ", v);
                Some(v)
            },
        )
        .unwrap();
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
