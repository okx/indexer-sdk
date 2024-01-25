use crate::configuration::base::IndexerConfiguration;
use crate::error::IndexerResult;
use async_channel::{Receiver, Sender};
use downcast_rs::{impl_downcast, Downcast};
use log::{info, warn};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task;
use tokio::task::JoinHandle;

pub mod client;
pub mod component;
pub mod configuration;
pub mod dispatcher;
pub mod error;
pub mod event;
pub mod factory;
pub mod processor;
pub mod storage;
pub mod types;

#[derive(Clone, Debug)]
pub struct IndexerContext {}

#[async_trait::async_trait]
pub trait IndexProcessor<T: Event + Clone>: Component<T> {}

pub trait Event: Downcast + Send + Sync + Debug {
    fn is_async(&self) -> bool {
        true
    }
}
impl_downcast!(Event);

impl Event for Arc<Box<dyn Event>> {}

#[async_trait::async_trait]
pub trait Component<T: Event + Clone>: Send + Sync {
    fn component_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    async fn init(&mut self, _cfg: IndexerConfiguration) -> IndexerResult<()> {
        Ok(())
    }

    async fn start(&mut self, _: watch::Receiver<()>) -> IndexerResult<Vec<JoinHandle<()>>> {
        Ok(vec![])
    }

    async fn close(&self) -> IndexerResult<()> {
        Ok(())
    }

    async fn handle_event(&mut self, _: &T) -> IndexerResult<()> {
        Ok(())
    }

    async fn interest(&self, _: &T) -> bool;
}

#[derive(Clone)]
pub struct ComponentTemplate<T: HookComponent<E> + Clone + 'static, E: Clone + Event> {
    internal: T,
    rx: async_channel::Receiver<E>,
    tx: Sender<E>,
}

impl<T: HookComponent<E> + Clone + 'static, E: Clone + Event> ComponentTemplate<T, E> {
    pub fn event_tx(&self) -> Sender<E> {
        self.tx.clone()
    }
}
impl<T: HookComponent<E> + Clone, E: Clone + Event> ComponentTemplate<T, E> {
    pub fn new(internal: T) -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self { internal, rx, tx }
    }
    pub fn new_with_tx_rx(internal: T, tx: Sender<E>, rx: Receiver<E>) -> Self {
        Self { internal, rx, tx }
    }
}

#[async_trait::async_trait]
impl<T: HookComponent<E> + Clone + 'static, E: Clone + Event> Component<E>
    for ComponentTemplate<T, E>
{
    async fn init(&mut self, cfg: IndexerConfiguration) -> IndexerResult<()> {
        info!("component {} init", self.component_name());
        self.internal.init(cfg).await
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

    async fn handle_event(&mut self, e: &E) -> IndexerResult<()> {
        self.internal.handle_event(e).await
    }

    async fn interest(&self, event: &E) -> bool {
        self.internal.interest(event).await
    }
}

#[async_trait::async_trait]
pub trait HookComponent<E: Clone + Event>: Component<E> {
    async fn before_start(
        &mut self,
        _: Sender<E>,
        _: Receiver<E>,
        exit: watch::Receiver<()>,
    ) -> IndexerResult<()> {
        Ok(())
    }

    fn interval(&self) -> Option<Duration> {
        None
    }

    async fn handle_tick_event(&mut self) -> IndexerResult<()> {
        Ok(())
    }

    async fn push_event(&mut self, _: &E) -> IndexerResult<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: HookComponent<E> + Clone, E: Clone + Event> HookComponent<E> for ComponentTemplate<T, E> {
    async fn before_start(
        &mut self,
        sender: Sender<E>,
        receiver: Receiver<E>,
        exit: watch::Receiver<()>,
    ) -> IndexerResult<()> {
        self.internal.before_start(sender, receiver, exit).await
    }

    fn interval(&self) -> Option<Duration> {
        self.internal.interval()
    }

    async fn handle_tick_event(&mut self) -> IndexerResult<()> {
        self.internal.handle_tick_event().await
    }

    async fn push_event(&mut self, event: &E) -> IndexerResult<()> {
        let _ = self.tx.send(event.clone()).await;
        Ok(())
    }
}
impl<T: HookComponent<E> + Clone, E: Clone + Event> ComponentTemplate<T, E> {
    async fn on_start(&mut self, mut exit: watch::Receiver<()>) -> IndexerResult<()> {
        info!("component {} starting", self.component_name());
        let tx = self.event_tx();
        let rx = self.rx.clone();
        self.internal.before_start(tx, rx, exit.clone()).await?;
        let rx = self.rx.clone();
        let interval = self.interval();
        if interval.is_none() {
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
                       },
                     _ = exit.changed() => {
                        self.event_tx().close();
                    let name=self.component_name();
                    info!("{:?},receive exit signal, exit.",name);
                    break;
                        }
                }
            }
        } else {
            let interval = interval.unwrap();
            let mut interval = tokio::time::interval(interval);
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
                    _ = exit.changed() => {
                        self.event_tx().close();
                    let name=self.component_name();
                    info!("{:?},receive exit signal, exit.",name);
                    break;
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
    use crate::factory::common::sync_create_and_start_processor;
    use bitcoincore_rpc::{Auth, Client, RpcApi};
    use std::thread::sleep;

    #[test]
    pub fn test_notifier() {
        let notifier = sync_create_and_start_processor(IndexerConfiguration::default());
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
