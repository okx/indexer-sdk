use tokio::task::JoinHandle;
use std::time::Duration;
use async_channel::Sender;
use log::{info, warn};
use crate::error::IndexerResult;
use tokio::sync::watch;
use tokio::task;
use crate::configuration::base::{IndexerConfiguration, ZMQConfiguration};
use crate::factory::common::sync_create_and_start_processor;
use crate::notifier::common::CommonNotifier;

pub mod storage;
pub mod error;
pub mod event;
pub mod processor;
pub mod types;
pub mod component;
pub mod configuration;
pub mod notifier;
pub mod factory;



#[no_mangle]
pub extern "C" fn create_processor() ->CommonNotifier {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .init();
    let ret=sync_create_and_start_processor(IndexerConfiguration{
        mq: ZMQConfiguration {
            zmq_url: "tcp://0.0.0.0:5555".to_string(),
            zmq_topic: vec![],
        },
    });
    ret
}


#[derive(Clone, Debug)]
pub struct IndexerContext {}


#[async_trait::async_trait]
pub trait IndexProcessor: Component {}

#[async_trait::async_trait]
pub trait Component: Send + Sync {
    type Event: Send + Sync + Clone;

    type Configuration: Send + Sync + Clone;
    type Inner: Component<Event=Self::Event>;

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
pub struct ComponentTemplate<T: Component<Event=E> + Clone + 'static, E: Send + Sync + Clone> {
    internal: T,
    rx: async_channel::Receiver<E>,
    tx: Sender<E>,
    _marker: std::marker::PhantomData<E>,
}

impl<T: Component<Event=E> + Clone, E: Send + Sync + Clone> ComponentTemplate<T, E> {
    pub fn new(internal: T) -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self {
            internal,
            rx,
            tx,
            _marker: Default::default(),
        }
    }
}


#[async_trait::async_trait]
impl<T: Component<Event=E> + Clone + 'static, E: Send + Sync + Clone + 'static> Component
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
}

impl<T: Component<Event=E> + Clone, E: Send + Sync + Clone + 'static> ComponentTemplate<T, E> {
    async fn on_start(&mut self, mut exit: watch::Receiver<()>) -> IndexerResult<()> {
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
    use std::thread::sleep;
    use tokio::sync::watch;
    use crate::configuration::base::{IndexerConfiguration, ZMQConfiguration};
    use crate::factory::common::{async_create_and_start_processor, sync_create_and_start_processor};
    use super::*;


    // #[test]
    // pub fn test_notifier() {
    //     env_logger::builder()
    //         .filter_level(log::LevelFilter::Debug)
    //         .format_target(false)
    //         .init();
    //     let (exit_tx, exit_rx) = watch::channel(());
    //     let notifier = sync_create_and_start_processor(exit_rx.clone(), IndexerConfiguration {
    //         mq: ZMQConfiguration { zmq_url: "tcp://0.0.0.0:5555".to_string(), zmq_topic: vec![] },
    //     });
    //     loop {
    //         let data = notifier.get();
    //         if data.len() > 0 {
    //             info!("receive data {:?}", data);
    //         }
    //         sleep(Duration::from_secs(3))
    //     }
    // }

    #[tokio::test]
    pub async fn test_async() {
        let (exit_tx, exit_rx) = watch::channel(());
        let notifier = async_create_and_start_processor(exit_rx.clone(), IndexerConfiguration {
            mq: ZMQConfiguration { zmq_url: "tcp://0.0.0.0:5555".to_string(), zmq_topic: vec![] },
        }).await.0;
        println!("1");
        loop {
            let data = notifier.get();
            if data.len() > 0 {
                info!("notifier receive data:{:?}",data);
            }
            sleep(Duration::from_secs(3))
        }
    }
}