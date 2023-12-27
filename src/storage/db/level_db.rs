use crate::error::IndexerResult;
use crate::storage::db::DB;
use rusty_leveldb::WriteBatch;

pub struct LevelDB {
    db: rusty_leveldb::DB,
}

impl Default for LevelDB {
    fn default() -> Self {
        let db = rusty_leveldb::DB::open("./db", rusty_leveldb::Options::default()).unwrap();
        LevelDB { db }
    }
}
impl DB for LevelDB {
    type Batch = WriteBatch;

    fn set(&mut self, key: &[u8], value: &[u8]) -> IndexerResult<()> {
        self.db.put(key, value)?;
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
        Ok(self.db.get(key))
    }

    fn write_batch(&mut self, batch: Self::Batch, sync: bool) -> IndexerResult<()> {
        self.db.write(batch, sync)?;
        Ok(())
    }
}
