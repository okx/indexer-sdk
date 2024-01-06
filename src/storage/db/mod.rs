pub mod level_db;
pub mod memory;
pub mod prefix;
pub mod thread_safe;

use crate::error::IndexerResult;
use crate::event::TxIdType;
use rusty_leveldb::WriteBatch;

pub trait DB {
    fn set(&mut self, tx_id: Option<TxIdType>, key: &[u8], value: &[u8]) -> IndexerResult<()>;
    fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>>;

    fn delete(&mut self, key: &[u8]) -> IndexerResult<()>;

    fn write_batch(
        &mut self,
        tx_id: Option<TxIdType>,
        batch: WriteBatch,
        sync: bool,
    ) -> IndexerResult<()>;

    fn iter_all_mut<KF, VF, K, V>(
        &mut self,
        prefix: &[u8],
        kf: KF,
        vf: VF,
    ) -> IndexerResult<Vec<(K, V)>>
    where
        KF: FnMut(Vec<u8>) -> K,
        VF: FnMut(Vec<u8>) -> Option<V>;

    fn remove_tx_traces(&mut self, tx_id: Vec<TxIdType>) -> IndexerResult<()>;
}
