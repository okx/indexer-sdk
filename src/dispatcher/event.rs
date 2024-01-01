use crate::component::waitsync::event::WaitSyncEvent;
use crate::component::zmq::event::ZeroMQEvent;
use crate::event::IndexerEvent;
use crate::Event;

#[derive(Clone)]
pub enum DispatchEvent {
    IndexerEvent(IndexerEvent),
    ZeroMQEvent(ZeroMQEvent),
    WaitSyncEvent(WaitSyncEvent),
}

unsafe impl Send for DispatchEvent {}

unsafe impl Sync for DispatchEvent {}

impl Event for DispatchEvent {}
