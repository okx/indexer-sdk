use bitcoincore_rpc::bitcoin::Transaction;
use log::{error, info};
use mylibrary::client::drect::DirectClient;
use mylibrary::client::Client;
use mylibrary::configuration::base::{IndexerConfiguration, NetConfiguration, ZMQConfiguration};
use mylibrary::event::TxIdType;
use mylibrary::factory::common::async_create_and_start_processor;
use mylibrary::storage::StorageProcessor;
use mylibrary::types::delta::TransactionDelta;
use mylibrary::types::response::GetDataResponse;
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
    let (client, tasks) = async_create_and_start_processor(
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
    rx: async_channel::Receiver<Transaction>,
    tx: async_channel::Sender<Transaction>,
}

#[derive(Clone)]
pub struct Executor<T: StorageProcessor + Clone + 'static> {
    rx: async_channel::Receiver<Transaction>,
    client: DirectClient<T>,
}

impl<T: StorageProcessor + Clone + 'static> Executor<T> {
    pub fn new(rx: async_channel::Receiver<Transaction>, client: DirectClient<T>) -> Self {
        Self { rx, client }
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
            let tx = data.unwrap();
            let ctx_res = self.execute(&tx).await;
            if let Err(e) = ctx_res {
                error!("execute error:{:?}", e);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            let ctx = ctx_res.unwrap();

            self.client
                .update_delta(ctx.delta.clone())
                .await
                .expect("unreachable");
        }
    }
    pub async fn execute(&self, tx: &Transaction) -> Result<TraceContext, anyhow::Error> {
        let tx_id: TxIdType = tx.txid().into();
        info!("start to execut transaction:{:?}", &tx_id);

        Ok(TraceContext {
            delta: TransactionDelta {
                tx_id,
                deltas: Default::default(),
            },
        })
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
        rx: async_channel::Receiver<Transaction>,
        tx: async_channel::Sender<Transaction>,
    ) -> Self {
        Self { rx, tx }
    }
}
