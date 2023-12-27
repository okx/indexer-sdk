mod level_db;
mod prefix;

use crate::error::IndexerResult;
use rusty_leveldb::WriteBatch;

pub trait DB {
    fn set(&mut self, key: &[u8], value: &[u8]) -> IndexerResult<()>;
    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>>;
    fn write_batch(&mut self, batch: WriteBatch, sync: bool) -> IndexerResult<()>;

    fn iter_all<KF, VF, K, V>(
        &mut self,
        prefix: &[u8],
        kf: KF,
        vf: VF,
    ) -> IndexerResult<Vec<(K, V)>>
    where
        KF: Fn(Vec<u8>) -> K,
        VF: Fn(Vec<u8>) -> Option<V>;
}
