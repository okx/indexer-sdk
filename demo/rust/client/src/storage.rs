use indexer_sdk::client::drect::DirectClient;
use indexer_sdk::client::SyncClient;
use indexer_sdk::event::TxIdType;
use indexer_sdk::storage::db::memory::MemoryDB;
use indexer_sdk::storage::db::thread_safe::ThreadSafeDB;
use indexer_sdk::storage::kv::KVStorageProcessor;

#[derive(Clone)]
pub struct ConfirmedDB {}

impl ConfirmedDB {
    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        //  .....
        None
    }
}

#[derive(Clone)]
pub struct MockStorage {
    client: DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>,

    confirmed_db: ConfirmedDB,
}

impl MockStorage {
    pub fn set(&mut self, tx_id: &TxIdType, key: &[u8], value: &[u8]) {
        //  or use custom db by yourself
        self.client.simple_set(tx_id, key, value.to_vec()).unwrap();
    }
    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let value = self.confirmed_db.get(key);
        if value.is_some() {
            return value;
        }
        return self.client.simple_get(key).unwrap();
    }
    pub fn new(client: DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>) -> Self {
        Self {
            client,
            confirmed_db: ConfirmedDB {},
        }
    }
}
