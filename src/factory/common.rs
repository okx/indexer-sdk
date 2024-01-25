use crate::client::common::CommonClient;
use crate::client::drect::DirectClient;
use crate::component::catchup::CacheUpComponent;
use crate::component::zmq::component::ZeroMQComponent;
use crate::configuration::base::{IndexerConfiguration, NetConfiguration, ZMQConfiguration};
use crate::dispatcher::Dispatcher;
use crate::processor::common::IndexerProcessorImpl;
use crate::storage::db::memory::MemoryDB;
use crate::storage::db::thread_safe::ThreadSafeDB;
use crate::storage::kv::KVStorageProcessor;
use crate::{wait_exit_signal, ComponentTemplate};
use bitcoincore_rpc::{Auth, Client};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use wg::AsyncWaitGroup;

pub fn new_client_for_test(
    url: String,
    user_name: String,
    password: String,
) -> DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>> {
    let db = ThreadSafeDB::new(MemoryDB::default());
    let processor = KVStorageProcessor::new(db);
    let client = Arc::new(create_client_from_configuration(IndexerConfiguration {
        mq: ZMQConfiguration {
            zmq_url: "".to_string(),
            zmq_topic: vec![],
        },
        net: NetConfiguration {
            url,
            username: user_name.to_string(),
            password,
        },
        db_path: "".to_string(),
        save_block_cache_count: 1,
        log_configuration: Default::default(),
    }));
    let (_, notify_rx) = async_channel::unbounded();
    let (tx, _) = async_channel::unbounded();
    let inner_client = CommonClient::new(notify_rx.clone(), tx.clone());
    let rt = Arc::new(Runtime::new().unwrap());
    DirectClient::new(rt, client, processor, inner_client)
}
pub async fn async_create_and_start_processor(
    origin_exit: watch::Receiver<()>,
    origin_cfg: IndexerConfiguration,
    rt: Arc<Runtime>,
) -> (
    DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>>,
    Vec<JoinHandle<()>>,
) {
    let flag = Arc::new(AtomicBool::new(false));
    // let db = LevelDB::new(origin_cfg.db_path.as_str()).unwrap();
    let db = ThreadSafeDB::new(MemoryDB::default());
    let processor = KVStorageProcessor::new(db);
    let client = Arc::new(create_client_from_configuration(origin_cfg.clone()));
    let (notify_tx, notify_rx) = async_channel::unbounded();

    let dispatcher = Box::leak(Box::new(Dispatcher::default()));
    let tx = dispatcher.tx();

    let wg = AsyncWaitGroup::new();
    let mq_wg = wg.add(1);
    let catch_up_wg = wg.add(1);

    let index_processor = {
        let (tx, rx) = async_channel::unbounded();
        let indexer_processor = IndexerProcessorImpl::new(
            origin_cfg.clone(),
            wg.clone(),
            notify_tx.clone(),
            processor.clone(),
            client.clone(),
            notify_tx.clone(),
            flag.clone(),
            tx.clone(),
            rx.clone(),
        );
        let indexer = ComponentTemplate::new_with_tx_rx(indexer_processor, tx.clone(), rx.clone());
        indexer
    };

    // let wait_cachup = ComponentTemplate::new(WaitIndexerCatchupComponent::new(
    //     catch_up_wg,
    //     client.clone(),
    //     notify_tx.clone(),
    // ));
    let catchup = ComponentTemplate::new(CacheUpComponent::new(
        client.clone(),
        catch_up_wg,
        tx.clone(),
        flag.clone(),
    ));

    let zmq = ComponentTemplate::new(ZeroMQComponent::new(
        mq_wg,
        origin_cfg.clone(),
        tx.clone(),
        flag.clone(),
    ));

    dispatcher.register_component(Box::new(index_processor));
    dispatcher.register_component(Box::new(catchup));
    dispatcher.register_component(Box::new(zmq));

    dispatcher.init(origin_cfg.clone()).await.unwrap();
    let ret = dispatcher
        .start(rt.clone(), origin_exit.clone())
        .await
        .unwrap();
    let inner_client = CommonClient::new(notify_rx.clone(), tx.clone());
    (
        DirectClient::new(rt.clone(), client.clone(), processor.clone(), inner_client),
        ret,
    )
}

pub(crate) fn create_client_from_configuration(
    config: IndexerConfiguration,
) -> bitcoincore_rpc::Client {
    Client::new(
        config.net.url.as_str(),
        Auth::UserPass(config.net.username.clone(), config.net.password.clone()),
    )
    .unwrap()
}

pub fn sync_create_and_start_processor(
    origin_cfg: IndexerConfiguration,
    rt: Arc<Runtime>,
) -> DirectClient<KVStorageProcessor<ThreadSafeDB<MemoryDB>>> {
    let (tx, rx) = watch::channel(());
    let rt2 = rt.clone();
    let ret = rt2.block_on(async_create_and_start_processor(
        rx,
        origin_cfg,
        rt2.clone(),
    ));

    thread::spawn(move || {
        rt.block_on(async {
            let handlers = ret.1;
            wait_exit_signal().await.unwrap();
            tx.send(()).unwrap();
            for h in handlers {
                h.await.unwrap();
            }
        });
    });

    ret.0
}
