use crate::client::common::CommonClient;
use crate::client::drect::DirectClient;
use crate::component::zmq::component::ZeroMQComponent;
use crate::configuration::base::IndexerConfiguration;
use crate::dispatcher::Dispatcher;
use crate::processor::common::IndexerProcessorImpl;
use crate::storage::db::memory::MemoryDB;
use crate::storage::kv::KVStorageProcessor;
use crate::{wait_exit_signal, ComponentTemplate};
use bitcoincore_rpc::{Auth, Client};
use log::error;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::{panic, thread};
use tokio::runtime;
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use wg::AsyncWaitGroup;

pub async fn async_create_and_start_processor(
    origin_exit: watch::Receiver<()>,
    origin_cfg: IndexerConfiguration,
) -> (
    DirectClient<KVStorageProcessor<MemoryDB>>,
    Vec<JoinHandle<()>>,
    Arc<Runtime>,
) {
    env_logger::builder()
        .filter_level(origin_cfg.log_configuration.log_level.clone())
        .format_target(false)
        .init();

    let rt = Arc::new(
        runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(),
    );

    panic::set_hook(Box::new(|panic_info| {
        println!("panic occurred: {:?}", panic_info);
        error!("panic occurred: {:?}", panic_info);
        exit(-1);
    }));
    let flag = Arc::new(AtomicBool::new(false));
    // let db = LevelDB::new(origin_cfg.db_path.as_str()).unwrap();
    let db = MemoryDB::default();
    let processor = KVStorageProcessor::new(db);
    let client = Arc::new(create_client_from_configuration(origin_cfg.clone()));
    let (notify_tx, notify_rx) = async_channel::unbounded();

    let dispatcher = Box::leak(Box::new(Dispatcher::default()));
    let tx = dispatcher.tx();

    let wg = AsyncWaitGroup::new();
    let mq_wg = wg.add(1);

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

    let zmq = ComponentTemplate::new(ZeroMQComponent::new(
        mq_wg,
        origin_cfg.clone(),
        tx.clone(),
        flag.clone(),
    ));

    dispatcher.register_component(Box::new(index_processor));
    // dispatcher.register_component(Box::new(wait_cachup));
    dispatcher.register_component(Box::new(zmq));

    dispatcher.init(origin_cfg.clone()).await.unwrap();
    let ret = dispatcher.start(origin_exit.clone()).await.unwrap();

    let client = CommonClient::new(notify_rx.clone(), tx.clone());
    (
        DirectClient::new(rt.clone(), processor.clone(), client),
        ret,
        rt.clone(),
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
) -> DirectClient<KVStorageProcessor<MemoryDB>> {
    let (tx, rx) = watch::channel(());
    let rt = Runtime::new().unwrap();
    let ret = rt.block_on(async_create_and_start_processor(rx, origin_cfg));
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
