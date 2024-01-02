use crate::client::SyncClient;
use crate::component::org::event::OrgEvent;
use crate::dispatcher::event::DispatchEvent;
use crate::error::IndexerResult;
use crate::{Component, HookComponent};
use bitcoincore_rpc::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct OrgComponent<T: SyncClient + Send + Sync> {
    net_client: Arc<Client>,
    indexer_client: T,
}

impl<T: SyncClient + Send + Sync> OrgComponent<T> {
    pub(crate) async fn do_handle_event(&self, event: &OrgEvent) -> IndexerResult<()> {
        match event {
            OrgEvent::ReportOrg => {
                self.do_handle_org().await?;
            }
        }
        Ok(())
    }
    async fn do_handle_org(&self) -> IndexerResult<()> {
        todo!()
    }
}

#[async_trait::async_trait]
impl<T: SyncClient + Send + Sync> Component<DispatchEvent> for OrgComponent<T> {
    async fn handle_event(&mut self, e: &DispatchEvent) -> IndexerResult<()> {
        let event = e.get_org_event().unwrap();
        self.do_handle_event(event).await
    }

    async fn interest(&self, e: &DispatchEvent) -> bool {
        e.get_org_event().is_some()
    }
}

#[async_trait::async_trait]
impl<T: SyncClient + Send + Sync> HookComponent<DispatchEvent> for OrgComponent<T> {}
