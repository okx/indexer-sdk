use crate::Event;
use wg::WaitGroup;

#[derive(Clone)]
pub enum WaitSyncEvent {
    IndexerOrg(WaitGroup),
    ReportHeight(u32),
}

impl Event for WaitSyncEvent {}
