use crate::configuration::base::{IndexerConfiguration, ZMQConfiguration};
use crate::error::IndexerResult;
use crate::factory::common::sync_create_and_start_processor;
use async_channel::Sender;
use log::{info, warn};
use std::ops::DerefMut;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task;
use tokio::task::JoinHandle;

pub mod client;
pub mod component;
pub mod configuration;
pub mod error;
pub mod event;
pub mod factory;
pub mod processor;
pub mod storage;
pub mod types;

#[derive(Clone, Debug)]
pub struct IndexerContext {}

#[async_trait::async_trait]
pub trait IndexProcessor: Component {}

#[async_trait::async_trait]
pub trait Component: Send + Sync {
    type Event: Send + Sync + Clone;

    type Configuration: Send + Sync + Clone;
    type Inner: Component<Event = Self::Event>;

    fn inner(&mut self) -> &mut Self::Inner;
    fn interval(&self) -> Duration;

    async fn init(&mut self, _cfg: Self::Configuration) -> IndexerResult<()> {
        Ok(())
    }

    async fn start(&mut self, _: watch::Receiver<()>) -> IndexerResult<Vec<JoinHandle<()>>> {
        Ok(vec![])
    }

    async fn close(&self) -> IndexerResult<()> {
        Ok(())
    }

    fn event_tx(&mut self) -> IndexerResult<Sender<Self::Event>> {
        self.inner().event_tx()
    }
    async fn handle_tick_event(&mut self) -> IndexerResult<()> {
        Ok(())
    }

    async fn handle_event(&mut self, event: &Self::Event) -> IndexerResult<()> {
        self.inner().handle_event(event).await
    }
}

#[derive(Clone)]
pub struct ComponentTemplate<T: HookComponent<Event = E> + Clone + 'static, E: Send + Sync + Clone>
{
    internal: T,
    rx: async_channel::Receiver<E>,
    tx: Sender<E>,
    _marker: std::marker::PhantomData<E>,
}

impl<T: HookComponent<Event = E> + Clone, E: Send + Sync + Clone> ComponentTemplate<T, E> {
    pub fn new(internal: T) -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self {
            internal,
            rx,
            tx,
            _marker: Default::default(),
        }
    }
    pub fn grap_rx(&self) -> async_channel::Receiver<E> {
        self.rx.clone()
    }
}

#[async_trait::async_trait]
impl<T: HookComponent<Event = E> + Clone + 'static, E: Send + Sync + Clone + 'static> Component
    for ComponentTemplate<T, E>
{
    type Event = E;

    fn interval(&self) -> Duration {
        self.internal.interval()
    }

    type Inner = T;

    type Configuration = T::Configuration;

    fn inner(&mut self) -> &mut Self::Inner {
        &mut self.internal
    }

    async fn init(&mut self, cfg: Self::Configuration) -> IndexerResult<()> {
        self.internal.init(cfg).await
    }

    fn event_tx(&mut self) -> IndexerResult<Sender<Self::Event>> {
        Ok(self.tx.clone())
    }

    async fn start(&mut self, exit: watch::Receiver<()>) -> IndexerResult<Vec<JoinHandle<()>>> {
        let mut ret = self.internal.start(exit.clone()).await?;
        let mut node = self.clone();
        let task = task::spawn(async move {
            node.on_start(exit.clone()).await.unwrap();
        });
        ret.push(task);
        Ok(ret)
    }

    async fn handle_tick_event(&mut self) -> IndexerResult<()> {
        self.internal.handle_tick_event().await
    }
}

#[async_trait::async_trait]
pub trait HookComponent: Component {
    async fn before_start(&mut self, sender: Sender<Self::Event>) -> IndexerResult<()> {
        Ok(())
    }
}

impl<T: HookComponent<Event = E> + Clone, E: Send + Sync + Clone + 'static>
    ComponentTemplate<T, E>
{
    async fn on_start(&mut self, exit: watch::Receiver<()>) -> IndexerResult<()> {
        let tx = self.event_tx().unwrap();
        self.internal.before_start(tx).await?;
        let interval = self.interval();
        let mut interval = tokio::time::interval(interval);
        let rx = self.rx.clone();
        loop {
            tokio::select! {
                 event=rx.recv()=>{
                    match event{
                        Ok(event) => {
                            if let Err(e)= self.handle_event(&event).await{
                                    log::error!("handle event error: {:?}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("receive event error: {:?}", e);
                            break;
                        }
                    }
                  }
               _ = interval.tick() => {
                    if let Err(e)=self.handle_tick_event().await{
                        warn!("handle tick event error: {:?}", e)
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_family = "unix")]
pub async fn wait_exit_signal() -> std::io::Result<()> {
    use tokio::signal::unix::{self, SignalKind};

    let mut interrupt = unix::signal(SignalKind::interrupt())?;
    let mut quit = unix::signal(SignalKind::quit())?;
    let mut terminate = unix::signal(SignalKind::terminate())?;

    tokio::select! {
        res = tokio::signal::ctrl_c() => {
            res
        }
        _ = interrupt.recv() => {
            Ok(())
        }
        _ = quit.recv() => {
            Ok(())
        }
        _ = terminate.recv() => {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::base::{IndexerConfiguration, ZMQConfiguration};
    use crate::factory::common::{
        async_create_and_start_processor, sync_create_and_start_processor,
    };
    use bitcoincore_rpc::{bitcoin, Auth, Client, Error, RpcApi};
    use std::thread::sleep;
    use tokio::sync::watch;

    #[test]
    pub fn test_notifier() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .format_target(false)
            .init();
        let notifier = sync_create_and_start_processor(IndexerConfiguration {
            mq: ZMQConfiguration {
                zmq_url: "tcp://0.0.0.0:28332".to_string(),
                zmq_topic: vec!["".to_string()],
            },
            net: Default::default(),
        });
        loop {
            let data = notifier.get();
            if data.len() > 0 {
                info!("receive data {:?}", data);
            }
            sleep(Duration::from_secs(3))
        }
    }

    #[tokio::test]
    pub async fn test_get_transactions() {
        // best block hash: [0x8a1487d5453855b4c279262f173ef64a161f37a325443421278b0680d6b46eed, 0x9308695d1fbd8b5bdf0077aff201de2291a282f44645c25c04e0c1e79f1398b9, 0xd96bdc2ee063e8907094c15b15fb4cec1179e0507fd64e8f50979d897ebb2478, 0x7c814bdf818bb972ef4e918a42ee58b1bbcba3f05ca1bc73a789e3c2deb0095f, 0xac0d0c41296fa23117408194c4b0a2caab67dd6b3b0b9589bce038cb6d96883b, 0x5dc9f57a43da08056f909759c0e074b626fa2e98c4eaeae84c572997572e6625]
        // http://bitcoinrpc:bitcoinrpc@localhost:18443/
        let rpc = Client::new(
            "http://localhost:18443",
            Auth::UserPass("bitcoinrpc".to_string(), "bitcoinrpc".to_string()),
        )
        .unwrap();
        let hash = rpc.get_best_block_hash().unwrap();
        let txs = rpc.get_raw_mempool().unwrap();
        println!("block_hash: {:?},txs:{:?}", &hash, txs);
    }

    // #[tokio::test]
    // pub async fn test_async() {
    //     let (exit_tx, exit_rx) = watch::channel(());
    //     let notifier = async_create_and_start_processor(exit_rx.clone(), IndexerConfiguration {
    //         mq: ZMQConfiguration { zmq_url: "tcp://0.0.0.0:5555".to_string(), zmq_topic: vec![] },
    //     }).await.0;
    //     println!("1");
    //     loop {
    //         let data = notifier.get();
    //         if data.len() > 0 {
    //             info!("notifier receive data:{:?}",data);
    //         }
    //         sleep(Duration::from_secs(3))
    //     }
    // }
}
