use crossbeam::channel::{Receiver, TryRecvError};
use log::info;
use crate::event::IndexerEvent;
use crate::notifier::internal_safe::InternalSafeChannel;

#[repr(C)]
#[derive(Clone)]
pub struct CommonNotifier {
    // rx: Receiver<Vec<u8>>,
    rx: InternalSafeChannel<Vec<u8>>,

    tx: async_channel::Sender<IndexerEvent>,
}


impl Default for CommonNotifier {
    fn default() -> Self {
        let (tx, _) = async_channel::unbounded();
        Self { rx: Default::default(), tx }
    }
}

impl CommonNotifier {
    pub fn asd(&self) -> i32 {
        1
    }
    pub fn get(&self) -> Vec<u8> {
        self.rx.recv().unwrap_or(vec![])
        // let res = self.rx.try_recv();
        // match res {
        //     Ok(ret) => {
        //         info!("get data from channel");
        //         ret
        //     }
        //     Err(v) => {
        //         match v {
        //             TryRecvError::Empty => {}
        //             TryRecvError::Disconnected => {}
        //         }
        //         vec![]
        //     }
        // }
    }
    // pub fn new(rx: Receiver<Vec<u8>>, tx: async_channel::Sender<IndexerEvent>) -> Self {
    //     Self { rx, tx }
    // }
    pub fn new(rx: InternalSafeChannel<Vec<u8>>, tx: async_channel::Sender<IndexerEvent>) -> Self {
        Self { rx, tx }
    }
}