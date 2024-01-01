pub mod event;

use crate::configuration::base::IndexerConfiguration;
use crate::error::IndexerResult;
use crate::{Component, Event};
use std::any::Any;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub struct Dispatcher<E: Event + Clone> {
    pub components: Vec<Box<dyn Component<E>>>,
    rx: async_channel::Receiver<Box<dyn Any + Send + Sync>>,
    tx: async_channel::Sender<Box<dyn Any + Send + Sync>>,
}

impl<E: Event + Clone> Dispatcher<E> {
    pub async fn init(&mut self, config: IndexerConfiguration) -> IndexerResult<()> {
        for component in self.components.iter_mut() {
            component.init(config.clone()).await?;
        }
        Ok(())
    }
    pub fn register_component(&mut self, component: Box<dyn Component<E>>) {
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
        loop {
            tokio::select! {
                _ = exit.changed() => {
                    break;
                }
                event = self.rx.recv() => {
                    if let Ok(event) = event {
                        for component in self.components.iter_mut() {

                        }
                        // for component in self.components.iter_mut() {
                        //     if component.interest(&event) {
                        //         component.handle_event(&event).await?;
                        //     }
                        // }
                    }
                }
            }
        }
    }
}
