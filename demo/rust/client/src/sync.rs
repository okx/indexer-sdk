use indexer_sdk::event::TxIdType;
use indexer_sdk::HookComponent;
use log::info;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

#[derive(Clone)]
pub struct MockSync {
    block_txs: async_channel::Sender<(u32, Vec<TxIdType>)>,
    mock_block_number: u32,
    pub(crate) current_number: Arc<Mutex<u32>>,
}

unsafe impl Send for MockSync {}

unsafe impl Sync for MockSync {}

impl MockSync {
    pub fn new(
        block_txs: async_channel::Sender<(u32, Vec<TxIdType>)>,
        current_number: Arc<Mutex<u32>>,
    ) -> Self {
        Self {
            block_txs,
            mock_block_number: 0,
            current_number: current_number,
        }
    }
}

#[derive(Clone)]
pub struct MockBlock {
    pub number: u32,
    pub txs: Vec<TxIdType>,
}

impl MockSync {
    pub fn start(&mut self) {
        loop {
            let block = self.period_sync_one_block();
            self.handle_block(&block);
            sleep(Duration::from_secs(3))
        }
    }
    fn handle_block(&mut self, block: &MockBlock) {
        info!("sync start to sync block,number:{:?}", block.number);
        //     ...... sync one  block
        let txs = block.txs.clone();
        let mut number = self.current_number.lock().unwrap();
        *number = block.number;
        let _ = self.block_txs.send_blocking((block.number, txs));
    }

    fn period_sync_one_block(&mut self) -> MockBlock {
        let tx_id = [0u8; 32];
        let tx_id = TxIdType::from_bytes(&tx_id);
        let block = MockBlock {
            number: self.mock_block_number,
            txs: vec![tx_id],
        };
        self.mock_block_number = self.mock_block_number + 1000;
        return block;
    }
}
