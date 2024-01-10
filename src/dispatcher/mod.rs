pub mod event;

use crate::configuration::base::IndexerConfiguration;
use crate::error::IndexerResult;
use crate::{Event, HookComponent};
use log::{info, warn};
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub struct Dispatcher<E: Event + Clone> {
    pub components: Vec<Box<dyn HookComponent<E>>>,
    rx: async_channel::Receiver<E>,
    tx: async_channel::Sender<E>,
}

impl<E: Event + Clone> Default for Dispatcher<E> {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded();
        Self {
            components: vec![],
            rx,
            tx,
        }
    }
}
impl<E: Event + Clone> Dispatcher<E> {
    pub fn tx(&self) -> async_channel::Sender<E> {
        self.tx.clone()
    }
}
impl<E: Event + Clone> Dispatcher<E> {
    pub async fn init(&mut self, config: IndexerConfiguration) -> IndexerResult<()> {
        for component in self.components.iter_mut() {
            component.init(config.clone()).await?;
        }
        Ok(())
    }
    pub fn register_component(&mut self, component: Box<dyn HookComponent<E>>) {
        self.components.push(component);
    }

    pub async fn start(
        &'static mut self,
        exit: watch::Receiver<()>,
    ) -> IndexerResult<Vec<JoinHandle<()>>> {
        let mut handles = vec![];
        for component in self.components.iter_mut() {
            let handle = component.start(exit.clone()).await?;
            handles.extend(handle);
        }
        handles.push(tokio::task::spawn(async {
            self.do_start(exit).await;
        }));
        Ok(handles)
    }

    pub async fn do_start(&mut self, mut exit: watch::Receiver<()>) {
        info!("dispatcher starting");
        loop {
            tokio::select! {
                _ = exit.changed() => {
                    break;
                }
                event = self.rx.recv() => {
                    info!("recv event:{:?}", &event);
                    match event{
                        Ok(event) => {
                            for component in self.components.iter_mut() {
                                if component.interest(&event).await{
                                    hit=true;
                                    component.push_event(&event).await.unwrap();
                                }
                            }
                              // for component in self.components.iter_mut() {
                        //     if component.interest(&event) {
                        //         component.handle_event(&event).await?;
                        //     }
                        // }
                        }
                        Err(e) => {
                            warn!("recv error:{:?}", e)
                        }
                    }
                }
            }
        }
    }
}

#[test]
pub fn test_asd() {}
