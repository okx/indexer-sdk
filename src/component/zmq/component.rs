use std::time::Duration;
use log::{error, info};
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
        Duration::from_secs(1)
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

            socket.subscribe("").await.unwrap();
            loop {
                tokio::select! {
                        event=socket.recv()=>{
                            if let Err(e)=event{
                                error!("receive msg failed:{:?}",e);
                                continue
                            }
                            let message=event.unwrap();
                            info!("received message:{:?}",message);
                            node.handle_message(&message).await;
                        }
                }
            }
        })
    }

    async fn handle_message(&self, message: &ZmqMessage) {
        let data = message.clone().into_vec();
        let mut ret = vec![];
        data
            .into_iter()
            .for_each(|v| {
                let bytes: Vec<u8> = v.into();
                ret.extend(bytes)
            });
        self.sender.send(IndexerEvent::NewTxComing(ret)).await.expect("unreachable");
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