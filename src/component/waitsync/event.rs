use crate::Event;
use wg::WaitGroup;

#[derive(Clone)]
pub enum WaitSyncEvent {
    IndexerOrg(WaitGroup),
}

impl Event for WaitSyncEvent {}
