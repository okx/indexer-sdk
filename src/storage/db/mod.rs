mod level_db;

use crate::error::IndexerResult;

pub trait DB {
    type Batch;
    fn set(&mut self, key: &[u8], value: &[u8]) -> IndexerResult<()>;
    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>>;

    fn write_batch(&mut self, batch: Self::Batch, sync: bool) -> IndexerResult<()>;
}
