// use crate::error::IndexerResult;
// use crate::storage::db::DB;
// use rusty_leveldb::WriteBatch;
//
// pub struct PrefixDB<T: DB> {
//     prefix: &'static [u8],
//     internal: T,
// }
//
// impl<T: DB> PrefixDB<T> {
//     pub fn new(prefix: &'static [u8], internal: T) -> Self {
//         Self { prefix, internal }
//     }
// }
//
// impl<T: DB> DB for PrefixDB<T> {
//     fn set(&mut self, key: &[u8], value: &[u8]) -> IndexerResult<()> {
//         self.internal.set(self.decorate_key(key), value)
//     }
//
//     fn get(&mut self, key: &[u8]) -> IndexerResult<Option<Vec<u8>>> {
//         self.internal.get(self.decorate_key(key))
//     }
//
//     fn write_batch(&mut self, batch: WriteBatch, sync: bool) -> IndexerResult<()> {
//         let iter = batch.iter();
//         let mut new_batch = WriteBatch::new();
//         iter.for_each(|(k, v)| {
//             let key = self.decorate_key(k);
//             match v {
//                 None => {
//                     new_batch.delete(key);
//                 }
//                 Some(v) => {
//                     new_batch.put(key, v);
//                 }
//             }
//         });
//         self.internal.write_batch(new_batch, sync)
//     }
//
//     fn iter_all<F, V>(&mut self, prefix: &[u8], f: F) -> IndexerResult<Vec<V>>
//     where
//         F: Fn(Vec<u8>) -> V,
//     {
//         self.internal.iter_all(self.decorate_key(prefix), f)
//     }
// }
//
// impl<T: DB> PrefixDB<T> {
//     fn decorate_key(&self, key: &[u8]) -> &[u8] {
//         let mut ret = self.prefix.to_vec();
//         ret.extend_from_slice(key);
//         ret.as_slice()
//     }
// }
