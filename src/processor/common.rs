use crate::client::event::ClientEvent;
use crate::configuration::base::IndexerConfiguration;
use crate::dispatcher::event::DispatchEvent;
use crate::error::IndexerResult;
use crate::event::{AddressType, BalanceType, IndexerEvent, TxIdType};
use crate::processor::node::TxNode;
use crate::storage::prefix::DeltaStatus;
use crate::storage::StorageProcessor;
use crate::types::delta::TransactionDelta;
use crate::{Component, HookComponent, IndexProcessor};
use async_channel::{Receiver, Sender};
use bitcoincore_rpc::bitcoin::consensus::{deserialize, serialize};
use bitcoincore_rpc::bitcoin::{Transaction, Txid};
use bitcoincore_rpc::RpcApi;
use chrono::Local;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use wg::AsyncWaitGroup;

#[derive(Clone)]
pub struct IndexerProcessorImpl<T: StorageProcessor> {
    config: IndexerConfiguration,
    tx: async_channel::Sender<ClientEvent>,
    storage: T,
    btc_client: Arc<bitcoincore_rpc::Client>,

    flag: Arc<AtomicBool>,
    wg: AsyncWaitGroup,

    client_tx: Sender<ClientEvent>,
    // a little tricky
    grap_tx: Sender<DispatchEvent>,
    grap_rx: Receiver<DispatchEvent>,

    last_indexer_height: Option<u32>,
    current_indexer_height: Option<u32>,
    current_chain_latest_height: Option<(u32, i64)>,

    analyses: HashMap<TxIdType, TxNode>,
}

unsafe impl<T: StorageProcessor> Send for IndexerProcessorImpl<T> {}

unsafe impl<T: StorageProcessor> Sync for IndexerProcessorImpl<T> {}

const MAX_UPDATE_CHAIN_HEIGHT_INTERVAL: i64 = 60 * 3;
impl<T: StorageProcessor> IndexerProcessorImpl<T> {
    pub fn new(
        config: IndexerConfiguration,
        wg: AsyncWaitGroup,
        tx: Sender<ClientEvent>,
        storage: T,
        client: Arc<bitcoincore_rpc::Client>,
        client_tx: Sender<ClientEvent>,
        flag: Arc<AtomicBool>,
        grap_tx: Sender<DispatchEvent>,
        grap_rx: Receiver<DispatchEvent>,
    ) -> Self {
        Self {
            config,
            tx,
            storage,
            btc_client: client,
            client_tx,
            grap_tx,
            flag,
            wg,
            last_indexer_height: None,
            current_indexer_height: None,
            current_chain_latest_height: None,
            grap_rx,
            analyses: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor> HookComponent<DispatchEvent> for IndexerProcessorImpl<T> {
    async fn before_start(
        &mut self,
        sender: Sender<DispatchEvent>,
        rx: Receiver<DispatchEvent>,
    ) -> IndexerResult<()> {
        self.wg.wait().await;
        self.wait_catchup(rx.clone()).await?;
        self.restore_from_mempool(sender).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor> Component<DispatchEvent> for IndexerProcessorImpl<T> {
    async fn handle_event(&mut self, event: &DispatchEvent) -> IndexerResult<()> {
        let event = event.get_indexer_event().unwrap();
        if let Err(e) = self.do_handle_event(event).await {
            error!("handle_event error:{:?}", e)
        }
        Ok(())
    }

    async fn interest(&self, event: &DispatchEvent) -> bool {
        event.get_indexer_event().is_some()
    }
}

impl<T: StorageProcessor> IndexerProcessorImpl<T> {
    async fn restore_from_mempool(&mut self, sender: Sender<DispatchEvent>) -> IndexerResult<()> {
        self.do_handle_sync_mempool(sender).await?;
        Ok(())
    }

    async fn do_handle_sync_mempool(&mut self, tx: Sender<DispatchEvent>) -> IndexerResult<()> {
        let all_unconsumed = self.storage.get_all_un_consumed_txs().await?;
        info!("all unconsumed txs:{:?}", all_unconsumed);
        let txs = {
            // sort by timestamp to execute tx in order
            let txs = self.btc_client.get_raw_mempool_verbose()?;
            let mut append = vec![];
            for (k, ts) in &all_unconsumed {
                let tx_id: Txid = k.clone().into();
                if !txs.contains_key(&tx_id) {
                    append.push((k.clone(), *ts));
                }
            }
            let mut sorted_pairs: Vec<_> = txs
                .into_iter()
                .map(|(tx_id, info)| {
                    let tx_id: TxIdType = tx_id.into();
                    (tx_id, info.time as i64)
                })
                .collect();
            sorted_pairs.extend_from_slice(append.as_slice());
            sorted_pairs.sort_by(|a, b| a.1.cmp(&b.1));
            sorted_pairs
        };

        for (tx_id, _) in txs {
            info!("get tx from mempool or db:{:?}", &tx_id);
            tx.send(DispatchEvent::IndexerEvent(
                IndexerEvent::TxFromRestoreByTxId(tx_id),
            ))
            .await
            .unwrap();
        }
        self.flag.store(true, Ordering::Relaxed);

        Ok(())
    }

    async fn wait_catchup(&mut self, rx: Receiver<DispatchEvent>) -> IndexerResult<()> {
        let grap_tx = self.client_tx.clone();
        let grap_rx = rx.clone();
        loop {
            let latest_block = self.btc_client.get_block_count();
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
            let event = event.get_indexer_event();
            if event.is_none() {
                continue;
            }
            let event = event.unwrap();
            if let IndexerEvent::ReportHeight(h) = event {
                info!(
                    "indexer latest height:{},chain latest height:{}",
                    h, net_latest_block
                );
                if *h as u64 >= net_latest_block {
                    info!("indexer catch up,waitsync done!");
                    self.current_indexer_height = Some(*h);
                    break;
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        Ok(())
    }
    async fn do_handle_event(&mut self, event: &IndexerEvent) -> IndexerResult<()> {
        info!("do_handle_event,event:{:?}", event);
        match event {
            IndexerEvent::NewTxComing(data, _) => {
                self.do_handle_new_tx_coming(data, false).await?;
            }
            IndexerEvent::GetBalance(address, tx) => {
                self.do_handle_get_balance(address, tx).await?;
            }
            IndexerEvent::UpdateDelta(data) => {
                self.do_handle_update_delta(data).await?;
            }
            IndexerEvent::TxConfirmed(tx_id) => {
                self.do_handle_tx_confirmed(tx_id, DeltaStatus::Confirmed)
                    .await?;
            }
            IndexerEvent::TxFromRestoreByTxId(tx_id) => {
                self.do_handle_restore_tx_by_tx_id(tx_id).await?;
            }
            IndexerEvent::TxRemoved(tx_id) => {
                self.do_handle_tx_removed(tx_id).await?;
            }
            IndexerEvent::ReportHeight(h) => {
                self.do_handle_block_catch_up(h).await?;
            }
            IndexerEvent::ReportReorg(v) => {
                self.do_handle_report_reorg(*v).await?;
            }
        }
        Ok(())
    }

    // force_dispatch:true: data from restore
    pub(crate) async fn do_handle_new_tx_coming(
        &mut self,
        data: &Vec<u8>,
        from_restore: bool,
    ) -> IndexerResult<()> {
        let data = self.parse_zmq_data(&data);
        if let Some((tx_id, tx)) = data {
            let seen = self.storage.seen_and_store_txs(&tx).await?;
            if seen.is_seen() {
                if from_restore {
                    if seen.is_executed() {
                        info!("tx_id:{:?} is seen and  has been executed,skip", tx_id);
                        return Ok(());
                    } else {
                        info!(
                            "tx_id:{:?} from restore  is seen but  has not been executed,start to dispatch",
                            tx_id
                        );
                    }
                } else {
                    info!("tx_id:{:?} has been seen,skip", tx_id);
                    return Ok(());
                }
            } else {
                info!("tx_id:{:?} has not been executed,start to dispatch", tx_id);
                self.analyse_transaction(&tx);
            }
            let latest_chain_height = self.get_latest_chain_height()?;
            let latest_indexer_height = self.get_current_indexer_height();

            if latest_chain_height > latest_indexer_height {
                warn!(
                    "indexer is not catch up,chain_height:{},indexer_height:{}",
                    latest_chain_height, latest_indexer_height
                );
                return self.restart().await;
            }

            self.storage
                .save_height_tx(latest_indexer_height, tx_id.clone())
                .await?;
            self.tx.send(ClientEvent::Transaction(tx)).await.unwrap();
        }
        Ok(())
    }

    fn analyse_transaction(&mut self, tx: &Transaction) {
        let tx_id: TxIdType = tx.txid().into();
        let node = self.analyses.get(&tx_id);
        if node.is_some() {
            return;
        }
        let current_node = TxNode::new(tx_id);
        // build by input
        for input in &tx.input {
            let prev_tx_id: TxIdType = input.previous_output.txid.into();
            let mut prev_node = self.analyses.get_mut(&prev_tx_id);
            if prev_node.is_none() {
                let tx_node = TxNode::new(prev_tx_id.clone());
                tx_node.nexts.add(current_node.clone());
                self.analyses.insert(prev_tx_id, tx_node);
            } else {
                let prev_node = prev_node.unwrap();
                prev_node.nexts.add(current_node.clone());
            }
        }
    }
    fn get_current_child_by_tx_id(&self, tx_id: &TxIdType) -> Vec<TxIdType> {
        let mut ret = vec![];
        ret.push(tx_id.clone());

        loop {
            let node = self.analyses.get(tx_id);
            if node.is_none() {
                break;
            }
            let node = node.unwrap();
            if node.nexts.is_empty() {
                break;
            }
            let nexts = node.nexts.clone();
            for next in nexts {
                ret.extend(self.loop_re_dispatch_by_tx_id(&next.current_hash));
            }
        }
        ret
    }
    fn get_latest_chain_height(&mut self) -> IndexerResult<u32> {
        let dt = Local::now();
        let now = dt.timestamp();
        if let Some((h, ts)) = self.current_chain_latest_height {
            let delta = now - ts;
            if delta <= MAX_UPDATE_CHAIN_HEIGHT_INTERVAL {
                return Ok(h);
            }
        }
        let height = self.btc_client.get_block_count()? as u32;
        self.current_chain_latest_height = Some((height, now));
        Ok(height)
    }
    fn get_current_indexer_height(&mut self) -> u32 {
        self.current_indexer_height.unwrap()
    }
    fn parse_zmq_data(&self, data: &Vec<u8>) -> Option<(TxIdType, Transaction)> {
        let tx: Transaction = deserialize(&data).expect("Failed to deserialize transaction");
        Some((tx.txid().into(), tx))
    }

    pub(crate) async fn do_handle_get_balance(
        &self,
        _: &AddressType,
        _: &crossbeam::channel::Sender<BalanceType>,
    ) -> IndexerResult<()> {
        todo!()
    }

    async fn do_handle_update_delta(&mut self, data: &TransactionDelta) -> IndexerResult<()> {
        self.storage.add_transaction_delta(data).await?;
        Ok(())
    }
    async fn do_handle_tx_confirmed(
        &mut self,
        tx_id: &TxIdType,
        status: DeltaStatus,
    ) -> IndexerResult<()> {
        self.storage.remove_transaction_delta(tx_id, status).await?;
        Ok(())
    }
    async fn do_handle_restore_tx_by_tx_id(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        let txid: Txid = tx_id.clone().into();
        info!("do_handle_force_tx_by_tx_id,txid:{:?}", txid);
        let transaction = self.btc_client.get_raw_transaction(&txid, None)?;
        let data = serialize(&transaction);
        self.do_handle_new_tx_coming(&data, true).await?;

        Ok(())
    }
    async fn do_handle_tx_removed(&mut self, tx_id: &TxIdType) -> IndexerResult<()> {
        self.do_handle_tx_confirmed(tx_id, DeltaStatus::InActive)
            .await?;
        self.tx
            .send(ClientEvent::TxDroped(tx_id.clone()))
            .await
            .unwrap();
        Ok(())
    }
    async fn do_handle_report_reorg(&mut self, org: u32) -> IndexerResult<()> {
        let current_height = self.current_indexer_height.unwrap();
        for i in org..current_height + 1 {
            self.storage.remove_height_traces(i).await.map_err(|e| {
                error!("remove_height_traces error:{:?}", e);
                e
            })?;
        }
        self.restart().await
    }
    async fn restart(&mut self) -> IndexerResult<()> {
        let h = self.current_indexer_height.unwrap();
        self.wait_catchup(self.grap_rx.clone()).await?;
        self.clean(h).await?;
        // maybe we need to flush the grap_tx?
        loop {
            let res = self.grap_tx.try_recv();
            if res.is_err() {
                break;
            }
        }
        self.restore_from_mempool(self.grap_tx.clone()).await?;
        Ok(())
    }
    async fn clean(&mut self, h: u32) -> IndexerResult<()> {
        self.flag.store(false, Ordering::Relaxed);
        self.analyses.clear();
        self.storage.remove_height_traces(h).await?;
        Ok(())
    }
    async fn do_handle_block_catch_up(&mut self, h: &u32) -> IndexerResult<()> {
        self.current_indexer_height = Some(*h);
        if self.last_indexer_height.is_none() {
            self.last_indexer_height = Some(*h);
        }
        self.storage.remove_height_traces(*h).await.map_err(|e| {
            error!("remove_height_traces error:{:?}", e);
            e
        })?;
        // if h % self.config.save_block_cache_count == 0 {
        //     // try to flush
        //     for i in h - self.config.save_block_cache_count..h + 1 {
        //         self.storage.remove_height_traces(*i).await.map_err(|e| {
        //             error!("remove_height_traces error:{:?}", e);
        //             e
        //         })?;
        //     }
        // }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: StorageProcessor> IndexProcessor<DispatchEvent> for IndexerProcessorImpl<T> {}
