mod event;

use crate::client::event::ClientEvent;
use crate::component::waitsync::event::WaitSyncEvent;
use crate::configuration::base::IndexerConfiguration;
use crate::error::IndexerResult;
use crate::event::IndexerEvent;
use crate::{Component, HookComponent};
use async_channel::Sender;
use bitcoincore_rpc::RpcApi;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use wg::{AsyncWaitGroup, WaitGroup};

#[derive(Clone)]
pub struct WaitIndexerCatchupComponent {
    wg: AsyncWaitGroup,
    net_client: Arc<bitcoincore_rpc::Client>,

    // a little tricky
    grap_rx: async_channel::Receiver<IndexerEvent>,
    grap_tx: async_channel::Sender<ClientEvent>,
}

impl WaitIndexerCatchupComponent {
    pub fn new(
        wg: AsyncWaitGroup,
        net_client: Arc<bitcoincore_rpc::Client>,
        grap_tx: async_channel::Sender<ClientEvent>,
        grap_rx: async_channel::Receiver<IndexerEvent>,
    ) -> Self {
        Self {
            wg,
            net_client,
            grap_rx,
            grap_tx,
        }
    }
}

#[async_trait::async_trait]
impl Component for WaitIndexerCatchupComponent {
    type Event = WaitSyncEvent;
    type Configuration = IndexerConfiguration;
    type Inner = Self;

    fn inner(&mut self) -> &mut Self::Inner {
        unreachable!()
    }

    async fn handle_event(&mut self, event: &Self::Event) -> IndexerResult<()> {
        match event {
            WaitSyncEvent::IndexerOrg(wg) => self.do_handle_indexer_org(wg).await?,
        }
        Ok(())
    }
}
impl WaitIndexerCatchupComponent {
    async fn do_handle_indexer_org(&mut self, event: &WaitGroup) -> IndexerResult<()> {
        Ok(())
    }
}

impl HookComponent for WaitIndexerCatchupComponent {
    async fn before_start(&mut self, _: Sender<Self::Event>) -> IndexerResult<()> {
        let grap_rx = self.grap_rx.clone();
        let grap_tx = self.grap_tx.clone();
        loop {
            let latest_block = self.net_client.get_block_count();
            if let Err(e) = latest_block {
                error!("get latest block error:{}", e);
                continue;
            }
            let net_latest_block = latest_block.unwrap();
            if let Err(e) = grap_tx.send(ClientEvent::GetHeight).await {
                error!("grap tx error:{}", e);
                continue;
            }
            let rx = grap_rx.recv().await;
            if let Err(e) = rx {
                error!("grap rx error:{}", e);
                continue;
            }
            let event = rx.unwrap();
            if let IndexerEvent::ReportHeight(h) = event {
                info!(
                    "indexer latest height:{},chain latest height:{}",
                    h, net_latest_block
                );
                if h as u64 >= net_latest_block {
                    info!("indexer catch up,waitsync done!");
                    break;
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        self.wg.done();
    }
}

#[tokio::test]
pub async fn test_wg() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::{
        spawn,
        time::{sleep, Duration},
    };
    use wg::AsyncWaitGroup;

    let wg = AsyncWaitGroup::new();
    let ctr = Arc::new(AtomicUsize::new(0));

    for _ in 0..5 {
        let ctrx = ctr.clone();
        let t_wg = wg.add(1);
        spawn(async move {
            // mock some time consuming task
            sleep(Duration::from_millis(50)).await;
            ctrx.fetch_add(1, Ordering::Relaxed);

            // mock task is finished
            t_wg.done();
        });
    }

    wg.wait().await;
    assert_eq!(ctr.load(Ordering::Relaxed), 5);
}
