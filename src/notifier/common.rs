use crossbeam::channel::{Receiver, TryRecvError};
use log::info;
use crate::event::IndexerEvent;

#[derive(Clone)]
pub struct CommonNotifier {
    rx: Receiver<Vec<u8>>,

    tx: async_channel::Sender<IndexerEvent>,
}

impl CommonNotifier {
    pub fn get(&self) -> Vec<u8> {
        let res = self.rx.try_recv();
        match res {
            Ok(ret) => {
                ret
            }
            Err(v) => {
                match v {
                    TryRecvError::Empty => {}
                    TryRecvError::Disconnected => {}
                }
                vec![]
            }
        }
    }
    pub fn new(rx: Receiver<Vec<u8>>, tx: async_channel::Sender<IndexerEvent>) -> Self {
        Self { rx, tx }
    }
}