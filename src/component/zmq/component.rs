use crate::configuration::base::IndexerConfiguration;
use crate::dispatcher::event::DispatchEvent;
use crate::error::IndexerResult;
use crate::event::{IndexerEvent, TxIdType};
use crate::factory::common::create_client_from_configuration;
use crate::{Component, HookComponent};
use bitcoincore_rpc::bitcoin::consensus::{deserialize, serialize};
use bitcoincore_rpc::bitcoin::hashes::Hash;
use bitcoincore_rpc::bitcoin::{Block, BlockHash, Transaction};
use bitcoincore_rpc::RpcApi;
use log::{error, info, warn};
use may::go;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::vec;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use wg::AsyncWaitGroup;
use zeromq::SocketRecv;
use zeromq::{Socket, ZmqMessage};

#[derive(Clone)]
pub struct ZeroMQComponent {
    config: IndexerConfiguration,
    sender: async_channel::Sender<DispatchEvent>,
    flag: Arc<AtomicBool>,
    wg: AsyncWaitGroup,
}

#[async_trait::async_trait]
impl HookComponent<DispatchEvent> for ZeroMQComponent {}

#[async_trait::async_trait]
impl Component<DispatchEvent> for ZeroMQComponent {
    async fn init(&mut self, cfg: IndexerConfiguration) -> IndexerResult<()> {
        self.config = cfg.clone();
        Ok(())
    }

    async fn start(&mut self, exit: Receiver<()>) -> IndexerResult<Vec<JoinHandle<()>>> {
        let mut ret = vec![];
        let node = ZeroMQNode::new(self.config.clone(), self.sender.clone(), self.flag.clone());
        ret.push(node.start(exit.clone(), self.wg.clone()).await);
        Ok(ret)
    }

    async fn interest(&self, event: &DispatchEvent) -> bool {
        match event {
            DispatchEvent::ZeroMQEvent(_) => true,
            _ => false,
        }
    }
}

impl ZeroMQComponent {
    pub fn new(
        mq_wg: AsyncWaitGroup,
        config: IndexerConfiguration,
        sender: async_channel::Sender<DispatchEvent>,
        flag: Arc<AtomicBool>,
    ) -> Self {
        Self {
            config,
            sender,
            flag,
            wg: mq_wg,
        }
    }
}

#[derive(Clone)]
struct ZeroMQNode {
    config: IndexerConfiguration,
    sender: async_channel::Sender<DispatchEvent>,
    flag: Arc<AtomicBool>,
    client: Arc<bitcoincore_rpc::Client>,
}

impl ZeroMQNode {
    pub fn new(
        config: IndexerConfiguration,
        sender: async_channel::Sender<DispatchEvent>,
        flag: Arc<AtomicBool>,
    ) -> Self {
        let client = create_client_from_configuration(config.clone());
        Self {
            config,
            sender,
            flag,
            client: Arc::new(client),
        }
    }
    async fn start(&self, _: Receiver<()>, wg: AsyncWaitGroup) -> JoinHandle<()> {
        let node = self.clone();
        let flag = self.flag.clone();
        tokio::task::spawn(async move {
            let mut socket = zeromq::SubSocket::new();
            socket
                .connect(node.config.mq.zmq_url.clone().as_str())
                .await
                .expect("Failed to connect");
            // for topic in &node.config.mq.zmq_topic {
            //     socket.subscribe(topic).await.unwrap();
            // }
            socket.subscribe("sequence").await.unwrap();
            socket.subscribe("rawblock").await.unwrap();
            wg.done();
            loop {
                tokio::select! {
                        event=socket.recv()=>{
                            if let Err(e)=event{
                                error!("receive msg failed:{:?}",e);
                                continue
                            }

                            loop{
                                let synced=flag.load(Ordering::Relaxed);
                                if synced{
                                    break;
                                }
                                info!("processor is not synced yet,wait 3s");
                                tokio::time::sleep(Duration::from_secs(3)).await
                            }

                            let message=event.unwrap();
                            if let Err(e)=node.handle_message(&message).await{
                                error!("handle message failed:{:?}",e);
                                continue
                        }
                        }
                }
            }
        })
    }

    async fn handle_message(&self, message: &ZmqMessage) -> IndexerResult<()> {
        let data = message.clone().into_vec();
        if data.is_empty() {
            warn!("receive empty message");
            return Ok(());
        }
        if data.len() != 3 {
            warn!("receive invalid message:{:?}", &data);
            return Ok(());
        }
        let topic = data.get(0).unwrap();
        let body = data.get(1).unwrap();
        let sequence = data.get(2).unwrap();
        let topic = String::from_utf8_lossy(&topic[..]).to_string();
        let events = if topic == "rawtx" {
            let raw_tx_data = body.to_vec();
            let transaction: Transaction =
                deserialize(&raw_tx_data).expect("Failed to deserialize transaction");
            let sequence_number =
                u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            info!(
                "receive new raw tx,tx_id:{},sequence:{}",
                transaction.txid(),
                sequence_number
            );
            let event = IndexerEvent::NewTxComing(raw_tx_data, sequence_number);
            vec![event]
        } else if topic == "hashblock" {
            let data = body.to_vec();
            let block_hash = hex::encode(&data);
            let sequence_number =
                u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            info!(
                "receive new block hash:{},sequence:{}",
                block_hash, sequence_number
            );
            let client = self.client.clone();
            let mut block_info = None;
            loop {
                let block = client.get_block(&BlockHash::from_slice(&data).unwrap());
                if let Err(e) = block {
                    error!("get block info failed:{:?},have to sleep", e);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    continue;
                }
                block_info = Some(block.unwrap());
                break;
            }
            let block_info: Block = block_info.unwrap();
            let txs: Vec<TxIdType> = block_info
                .txdata
                .into_iter()
                .map(|v| v.txid().into())
                .collect();
            info!(
                "block txs,block_hash:{:?},txs count:{:?}",
                block_hash,
                txs.len()
            );
            let sender = self.sender.clone();
            go!(move || {
                for tx in txs {
                    sender
                        .send_blocking(DispatchEvent::IndexerEvent(IndexerEvent::NewTxComing(
                            tx.to_bytes(),
                            sequence_number,
                        )))
                        .expect("unreachable")
                }
            });

            vec![]
        } else if topic == "hashtx" {
            let tx_hash = hex::encode(&body.to_vec());
            let sequence_number =
                u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            info!(
                "receive new tx hash:{},sequence:{}",
                tx_hash, sequence_number
            );
            vec![]
        } else if topic == "rawblock" {
            let sequence_number =
                u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            let body = &body.to_vec();
            let new_block = deserialize::<Block>(&body).expect("Failed to deserialize block");
            info!(
                "receive new raw block,sequence:{},hash:{:?}",
                sequence_number,
                new_block.block_hash()
            );
            let events = new_block
                .txdata
                .iter()
                .map(|tx| IndexerEvent::TxConfirmed(tx.txid().into()))
                .collect::<Vec<IndexerEvent>>();
            events
        } else if topic == "sequence" {
            let hash = hex::encode(&body[..32]);
            let label = body[32] as char;
            info!(
                "receive sequence topic:{:?},tx_hash:{},label:{}",
                topic, hash, label
            );
            if label == 'R' {
                vec![IndexerEvent::TxRemoved(TxIdType::from(hash))]
            } else if label == 'A' {
                let tx_hash = TxIdType::from(hash).into();
                let tx = self.client.get_raw_transaction(&tx_hash, None)?;
                let tx = serialize(&tx);
                vec![IndexerEvent::NewTxComing(tx, 0)]
            } else if label == 'C' {
                vec![IndexerEvent::TxConfirmed(TxIdType::from(hash))]
            } else {
                warn!(
                    "receive unknown label:{:?},maybe we need to handle it",
                    label
                );
                vec![]
            }
        } else {
            warn!("receive unknown topic:{:?}", topic);
            vec![]
        };
        for event in events {
            self.sender
                .send(DispatchEvent::IndexerEvent(event))
                .await
                .expect("unreachable");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::base::ZMQConfiguration;
    use std::thread::sleep;
    use tokio::sync::watch;

    #[tokio::test]
    pub async fn test_asd() {
        let config = IndexerConfiguration::default();
        let (tx, _) = async_channel::unbounded();
        let wg = AsyncWaitGroup::new();
        let mut component =
            ZeroMQComponent::new(wg, config, tx.clone(), Arc::new(AtomicBool::new(true)));
        let (exit_tx, exit_rx) = watch::channel(());
        let nodes = component.start(exit_rx.clone()).await.unwrap();
        for node in nodes {
            node.await.unwrap();
        }
        sleep(Duration::from_secs(10000000000));
        drop(exit_tx)
    }

    #[tokio::test]
    pub async fn test_zeromq() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .format_target(false)
            .init();
        let (exit_tx, exit_rx) = watch::channel(());
        let mut config = IndexerConfiguration::default();
        config.mq.zmq_url = "tcp://0.0.0.0:28332".to_string();

        let (tx, _) = async_channel::unbounded();
        let node = ZeroMQNode::new(config, tx, Arc::new(AtomicBool::new(true)));
        let wg = AsyncWaitGroup::new();
        let handler = node.start(exit_rx, wg).await;
        handler.await.expect("TODO: panic message");
        sleep(Duration::from_secs(10000000000));
        drop(exit_tx)
    }
}
