use bitcoincore_rpc::bitcoin::Transaction;
use indexer_sdk::client::drect::DirectClient;
use indexer_sdk::client::event::ClientEvent;
use indexer_sdk::client::Client;
use indexer_sdk::configuration::base::{IndexerConfiguration, NetConfiguration, ZMQConfiguration};
use indexer_sdk::event::TxIdType;
use indexer_sdk::factory::common::async_create_and_start_processor;
use indexer_sdk::storage::StorageProcessor;
use indexer_sdk::types::delta::TransactionDelta;
use indexer_sdk::types::response::GetDataResponse;
use log::{error, info};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::time::Duration;

#[tokio::main]
pub async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .init();

    let (tx, rx) = watch::channel(());
    let mut handlers = vec![];
    let (client, tasks, _) = async_create_and_start_processor(
        rx,
        IndexerConfiguration {
            mq: ZMQConfiguration {
                zmq_url: "tcp://0.0.0.0:28332".to_string(),
                zmq_topic: vec!["sequence".to_string(), "rawtx".to_string()],
            },
            net: NetConfiguration {
                url: "http://localhost:18443".to_string(),
                username: "bitcoinrpc".to_string(),
                password: "bitcoinrpc".to_string(),
            },
            db_path: "./db".to_string(),
        },
    )
    .await;
    handlers.extend(tasks);

    let (tx, rx) = async_channel::unbounded();
    let dispatcher = MockDispatcher::new(client.rx(), tx);
    handlers.push(dispatcher.start().await.unwrap());

    let mut executor = Executor::new(rx, client);
    handlers.push(executor.start().await);

    for h in handlers {
        h.await.unwrap();
    }
}

#[derive(Clone)]
pub struct MockDispatcher {
    rx: async_channel::Receiver<ClientEvent>,
    tx: async_channel::Sender<ClientEvent>,
}

#[derive(Clone)]
pub struct Executor<T: StorageProcessor + Clone + 'static> {
    rx: async_channel::Receiver<ClientEvent>,
    client: DirectClient<T>,
    height: u32,
}

impl<T: StorageProcessor + Clone + 'static> Executor<T> {
    pub fn new(rx: async_channel::Receiver<ClientEvent>, client: DirectClient<T>) -> Self {
        Self {
            rx,
            client,
            height: 0,
        }
    }
}

#[derive(Clone)]
pub struct TraceContext {
    pub delta: TransactionDelta,
}

impl<T: StorageProcessor + Clone + 'static> Executor<T> {
    pub async fn start(&mut self) -> JoinHandle<()> {
        let mut internal = self.clone();
        tokio::spawn(async move { internal.do_start().await })
    }
    async fn do_start(&mut self) {
        info!("executor starting");
        loop {
            let data = self.rx.recv().await;
            if let Err(e) = data {
                error!("recv error:{:?}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let event = data.unwrap();
            let ctx_res = self.execute(&event).await;
            if let Err(e) = ctx_res {
                error!("execute error:{:?}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let ctx = ctx_res.unwrap();
            if let Some(ctx) = ctx {
                self.client
                    .update_delta(ctx.delta.clone())
                    .await
                    .expect("unreachable");
            }
        }
    }
    pub async fn execute(
        &mut self,
        tx: &ClientEvent,
    ) -> Result<Option<TraceContext>, anyhow::Error> {
        match tx {
            ClientEvent::Transaction(tx) => {
                let tx_id: TxIdType = tx.txid().into();
                info!("start to execut transaction:{:?}", &tx_id);

                Ok(Some(TraceContext {
                    delta: TransactionDelta {
                        tx_id,
                        deltas: Default::default(),
                    },
                }))
            }
            ClientEvent::GetHeight => {
                self.height = self.height + 50;
                self.client.report_height(self.height).await.unwrap();
                Ok(None)
            }
            _ => {
                info!("ignore event");
                Ok(None)
            }
        }
    }
}
impl MockDispatcher {
    pub async fn start(&self) -> Result<JoinHandle<()>, anyhow::Error> {
        let rx = self.rx.clone();
        let tx = self.tx.clone();
        Ok(tokio::spawn(async move {
            loop {
                let data = rx.recv().await;
                if let Err(e) = data {
                    error!("recv error:{:?}", e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                let transaction = data.unwrap();
                tx.send(transaction).await.expect("unreachable")
            }
        }))
    }
    pub fn new(
        rx: async_channel::Receiver<ClientEvent>,
        tx: async_channel::Sender<ClientEvent>,
    ) -> Self {
        Self { rx, tx }
    }
}
