use crate::storage::MockStorage;
use crate::sync::MockSync;
use bitcoincore_rpc::bitcoin::Transaction;
use indexer_sdk::client::drect::DirectClient;
use indexer_sdk::client::event::ClientEvent;
use indexer_sdk::client::SyncClient;
use indexer_sdk::event::TxIdType;
use indexer_sdk::storage::db::memory::MemoryDB;
use indexer_sdk::storage::kv::KVStorageProcessor;
use indexer_sdk::types::delta::TransactionDelta;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockPending {
    pub(crate) client: DirectClient<KVStorageProcessor<MemoryDB>>,
    pub(crate) synchronizer: Rc<RefCell<MockSync>>,

    pub stroage: MockStorage,
    block_confirmed: async_channel::Receiver<(u32, Vec<TxIdType>)>,

    block_number: Arc<Mutex<u32>>,
}

unsafe impl Send for MockPending {}

unsafe impl Sync for MockPending {}

impl MockPending {
    pub fn start(&mut self) {
        loop {
            let event = self.client.block_get_event().unwrap();
            self.handle_event(event)
        }
    }

    fn handle_event(&mut self, event: ClientEvent) {
        match event {
            ClientEvent::Transaction(tx) => {
                let response = self.simulate_tx(tx);
                self.client.update_delta(response).unwrap();
            }
            ClientEvent::TxDroped(tx) => {
                self.client.remove_tx_traces(vec![tx.clone()]).unwrap();
            }
            ClientEvent::TxConfirmed(tx) => {}
            ClientEvent::GetHeight => {
                let synchronizer = self.synchronizer.borrow();
                let number = self.block_number.lock().unwrap();
                let number = *number;
                self.client.report_height(number).unwrap();
            }
        }
    }
    fn simulate_tx(&mut self, tx: Transaction) -> TransactionDelta {
        let mut last_traces = vec![];
        loop {
            let data = self.block_confirmed.try_recv();
            if let Err(_) = data {
                break;
            }
            last_traces.push(data.unwrap())
        }

        //  remove traces
        for (h, txs) in last_traces {
            self.client.remove_tx_traces(txs).unwrap();
        }

        let tx_id: TxIdType = tx.txid().into();

        //     execute_tx
        let mock_key = b"mock_key1";
        let mock_value = b"mock_value";
        self.stroage.set(&tx_id, mock_key, mock_value);

        return TransactionDelta {
            tx_id,
            deltas: Default::default(),
        };
    }
    pub fn new(
        client: DirectClient<KVStorageProcessor<MemoryDB>>,
        synchronizer: Rc<RefCell<MockSync>>,
        stroage: MockStorage,
        block_confirmed: async_channel::Receiver<(u32, Vec<TxIdType>)>,
        block_number: Arc<Mutex<u32>>,
    ) -> Self {
        Self {
            client,
            synchronizer,
            stroage,
            block_confirmed,
            block_number,
        }
    }
}

#[test]
pub fn test_asd() {}
