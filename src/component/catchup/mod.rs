use crate::dispatcher::event::DispatchEvent;
use crate::error::IndexerResult;
use crate::event::IndexerEvent::TxConfirmed;
use crate::event::TxIdType;
use crate::{Component, HookComponent};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use bitcoincore_rpc::bitcoin::BlockHash;
use bitcoincore_rpc::{Client, RpcApi};
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use wg::AsyncWaitGroup;

#[derive(Clone)]
pub struct CacheUpComponent {
    btc_client: Arc<Client>,

    current_block_info: Option<BlockWrapper>,
    wg: AsyncWaitGroup,

    tx: Sender<DispatchEvent>,
    flag: Arc<AtomicBool>,
}

#[derive(Clone)]
struct BlockWrapper {
    height: u64,
    hash: BlockHash,
}

#[async_trait]
impl Component<DispatchEvent> for CacheUpComponent {
    async fn interest(&self, _: &DispatchEvent) -> bool {
        false
    }
}

#[async_trait]
impl HookComponent<DispatchEvent> for CacheUpComponent {
    async fn before_start(
        &mut self,
        _: Sender<DispatchEvent>,
        _: Receiver<DispatchEvent>,
    ) -> IndexerResult<()> {
        let info = self.btc_client.get_blockchain_info()?;
        self.current_block_info = Some(BlockWrapper {
            height: info.blocks,
            hash: info.best_block_hash,
        });
        self.wg.done();
        Ok(())
    }

    fn interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(60))
    }

    async fn handle_tick_event(&mut self) -> IndexerResult<()> {
        let info = self.btc_client.get_blockchain_info()?;
        let current_info = self.current_block_info.as_ref().unwrap();
        if info.blocks > current_info.height {
            info!(
                "need to catchup block,from:{},to:{}",
                current_info.height + 1,
                info.blocks
            );
            self.catch_up_block(current_info.height + 1, info.blocks)
                .await?;
        }

        Ok(())
    }
}

impl CacheUpComponent {
    async fn catch_up_block(&mut self, from: u64, to: u64) -> IndexerResult<()> {
        for i in from..to + 1 {
            let info = self.btc_client.get_block_hash(i)?;
            let block = self.btc_client.get_block(&info)?;

            let events: Vec<DispatchEvent> = block
                .txdata
                .into_iter()
                .map(|v| {
                    let tx_id = v.txid();
                    let tx_id_type: TxIdType = tx_id.into();
                    DispatchEvent::IndexerEvent(TxConfirmed(tx_id_type))
                })
                .collect();

            let synced = self.flag.load(Ordering::Relaxed);
            if synced {
                info!("synced,start to send events:{}", i);
                for event in events {
                    let _ = self.tx.send(event).await;
                }
            }

            info!("catchup block:{}", i);
            self.current_block_info = Some(BlockWrapper {
                height: i,
                hash: info,
            });
        }
        Ok(())
    }
    pub fn new(
        btc_client: Arc<Client>,
        wg: AsyncWaitGroup,
        tx: Sender<DispatchEvent>,
        flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            btc_client,
            current_block_info: None,
            wg,
            tx,
            flag,
        }
    }
}
