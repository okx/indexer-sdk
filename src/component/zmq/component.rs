use std::time::Duration;
use bitcoincore_rpc::bitcoin;
use bitcoincore_rpc::bitcoin::{Block, Transaction};
use bitcoincore_rpc::bitcoin::consensus::{Decodable, deserialize};
use bitcoincore_rpc::bitcoin::p2p::message_blockdata::Inventory;
use log::{error, info, warn};
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use zeromq::{Socket, ZmqMessage};
use crate::Component;
use crate::configuration::base::IndexerConfiguration;
use crate::error::IndexerResult;
use crate::event::IndexerEvent;
use zeromq::SocketRecv;

#[derive(Clone)]
pub struct ZeroMQComponent {
    config: IndexerConfiguration,
    sender: async_channel::Sender<IndexerEvent>,
}


#[async_trait::async_trait]
impl Component for ZeroMQComponent {
    type Event = IndexerEvent;
    type Configuration = IndexerConfiguration;
    type Inner = Self;

    fn inner(&mut self) -> &mut Self::Inner {
        unreachable!()
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(300)
    }

    async fn init(&mut self, cfg: Self::Configuration) -> IndexerResult<()> {
        self.config = cfg.clone();
        Ok(())
    }

    async fn start(&mut self, exit: Receiver<()>) -> IndexerResult<Vec<JoinHandle<()>>> {
        let node = ZeroMQNode::new(self.config.clone(), self.sender.clone());
        Ok(vec![node.start(exit.clone()).await])
    }
}

impl ZeroMQComponent {
    pub fn new(config: IndexerConfiguration, sender: async_channel::Sender<IndexerEvent>) -> Self {
        Self { config, sender }
    }
}


#[derive(Clone)]
struct ZeroMQNode {
    config: IndexerConfiguration,
    sender: async_channel::Sender<IndexerEvent>,
}

impl ZeroMQNode {
    pub fn new(config: IndexerConfiguration, sender: async_channel::Sender<IndexerEvent>) -> Self {
        Self { config, sender }
    }
    async fn start(&self, exit: Receiver<()>) -> JoinHandle<()> {
        let node = self.clone();
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
            loop {
                tokio::select! {
                        event=socket.recv()=>{
                            if let Err(e)=event{
                                error!("receive msg failed:{:?}",e);
                                continue
                            }
                            let message=event.unwrap();
                            node.handle_message(&message).await;
                        }
                }
            }
        })
    }

    async fn handle_message(&self, message: &ZmqMessage) {
        let data = message.clone().into_vec();
        if data.is_empty() {
            warn!("receive empty message");
            return;
        }
        if data.len() != 3 {
            warn!("receive invalid message");
            return;
        }
        let topic = data.get(0).unwrap();
        let body = data.get(1).unwrap();
        let sequence = data.get(2).unwrap();
        let topic = String::from_utf8_lossy(&topic[..]).to_string();
        let event = if topic == "rawtx" {
            let raw_tx_data = body.to_vec();
            let transaction: Transaction = deserialize(&raw_tx_data).expect("Failed to deserialize transaction");
            let sequence_number = u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            let event = IndexerEvent::NewTxComing(raw_tx_data, sequence_number);
            info!("receive new raw tx,tx_id:{},sequence:{}",transaction.txid(),sequence_number);
            Some(event)
        } else if topic == "hashblock" {
            let block_hash = hex::encode(&body.to_vec());
            let sequence_number = u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            info!("receive new block hash:{},sequence:{}",block_hash,sequence_number);
            None
        } else if topic == "hashtx" {
            let tx_hash = hex::encode(&body.to_vec());
            let sequence_number = u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            info!("receive new tx hash:{},sequence:{}",tx_hash,sequence_number);
            None
        } else if topic == "rawblock" {
            let raw_block_data = body.to_vec();
            let block: Block = bitcoin::consensus::deserialize(&raw_block_data).unwrap();
            let sequence_number = u32::from_le_bytes(sequence.to_vec().as_slice().try_into().unwrap());
            Some(IndexerEvent::RawBlockComing(block, sequence_number))
        } else if topic == "sequence" {
            let hash = hex::encode(&body[..32]);
            let label = body[32] as char;
            info!("receive sequence topic:{:?},hash:{},label:{}",topic,hash,label);
            None
        } else {
            warn!("receive unknown topic:{:?}",topic);
            None
        };
        if let Some(event) = event {
            self.sender.send(event).await.expect("unreachable");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use tokio::sync::watch;
    use crate::configuration::base::ZMQConfiguration;
    use super::*;

    #[tokio::test]
    pub async fn test_asd() {
        let config = IndexerConfiguration {
            mq: ZMQConfiguration {
                zmq_url: "tcp://0.0.0.0:5555".to_string(),
                zmq_topic: vec![],
            },
            net: Default::default(),
        };
        let (tx, rx) = async_channel::unbounded();
        let mut component = ZeroMQComponent::new(config, tx.clone());
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
        let config = IndexerConfiguration {
            mq: ZMQConfiguration {
                zmq_url: "tcp://0.0.0.0:28332".to_string(),
                zmq_topic: vec![],
            },
            net: Default::default(),
        };
        let (tx, rx) = async_channel::unbounded();
        let node = ZeroMQNode::new(config, tx);
        let handler = node.start(exit_rx).await;
        handler.await.expect("TODO: panic message");
        sleep(Duration::from_secs(10000000000));
        drop(exit_tx)
    }
}