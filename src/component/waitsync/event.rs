use wg::WaitGroup;

pub enum WaitSyncEvent {
    IndexerOrg(WaitGroup),
}
