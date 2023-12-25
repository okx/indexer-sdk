use crossbeam::channel::{Receiver, TryRecvError};
use log::info;
use crate::event::IndexerEvent;

#[repr(C)]
#[derive(Clone)]
pub struct CommonNotifier {
    rx: Receiver<Vec<u8>>,
    tx: async_channel::Sender<IndexerEvent>,
}

impl CommonNotifier {
    pub fn asd(&self)->i32{
        1
    }
    pub fn get(&self) -> Vec<u8> {
        let res = self.rx.try_recv();
        match res {
            Ok(ret) => {
                info!("get data from channel");
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