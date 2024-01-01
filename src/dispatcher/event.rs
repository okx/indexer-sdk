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

impl DispatchEvent {
    pub fn get_indexer_event(&self) -> Option<&IndexerEvent> {
        match self {
            DispatchEvent::IndexerEvent(event) => Some(event),
            _ => None,
        }
    }
    pub fn get_zmq_event(&self) -> Option<&ZeroMQEvent> {
        match self {
            DispatchEvent::ZeroMQEvent(event) => Some(event),
            _ => None,
        }
    }
    pub fn get_waitsync_event(&self) -> Option<&WaitSyncEvent> {
        match self {
            DispatchEvent::WaitSyncEvent(event) => Some(event),
            _ => None,
        }
    }
}
