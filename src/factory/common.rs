use crate::client::common::CommonClient;
use crate::client::drect::DirectClient;
use crate::component::zmq::component::ZeroMQComponent;
use crate::configuration::base::IndexerConfiguration;
use crate::processor::common::IndexerProcessorImpl;
use crate::storage::db::level_db::LevelDB;
use crate::storage::db::memory::MemoryDB;
use crate::storage::kv::KVStorageProcessor;
use crate::{Component, ComponentTemplate};
use bitcoincore_rpc::{Auth, Client};
use core::arch;
use log::error;
use std::cell::RefCell;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::{panic, thread};
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub async fn async_create_and_start_processor(
    origin_exit: watch::Receiver<()>,
    origin_cfg: IndexerConfiguration,
) -> (
    DirectClient<KVStorageProcessor<LevelDB>>,
    Vec<JoinHandle<()>>,
) {
    panic::set_hook(Box::new(|panic_info| {
        println!("panic occurred: {:?}", panic_info);
        error!("panic occurred: {:?}", panic_info);
        exit(-1);
    }));
    let flag = Arc::new(AtomicBool::new(false));
    // let db = MemoryDB::default();
    let db = LevelDB::default();
    let processor = KVStorageProcessor::new(db);
    let client = create_client_from_configuration(origin_cfg.clone());

    let (notify_tx, notify_rx) = async_channel::unbounded();

    let mut processor_wrapper = ComponentTemplate::new(IndexerProcessorImpl::new(
        notify_tx.clone(),
        processor.clone(),
        client,
        flag.clone(),
    ));
    let indexer_tx = processor_wrapper.event_tx().unwrap();

    let mut ret = vec![];
    let mut zmq_wrapper = ComponentTemplate::new(ZeroMQComponent::new(
        origin_cfg.clone(),
        indexer_tx.clone(),
        flag.clone(),
    ));
    zmq_wrapper.init(origin_cfg.clone()).await.unwrap();
    ret.extend(zmq_wrapper.start(origin_exit.clone()).await.unwrap());

    processor_wrapper.init(origin_cfg.clone()).await.unwrap();
    ret.extend(processor_wrapper.start(origin_exit.clone()).await.unwrap());

    let client = CommonClient::new(notify_rx.clone(), indexer_tx.clone());
    (DirectClient::new(processor.clone(), client), ret)
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
) -> DirectClient<KVStorageProcessor<LevelDB>> {
    let (tx, rx) = watch::channel(());
    let rt = Runtime::new().unwrap();
    let ret = rt.block_on(async_create_and_start_processor(rx, origin_cfg));
    thread::spawn(move || {
        rt.block_on(async {
            let handlers = ret.1;
            for h in handlers {
                h.await.unwrap();
            }
        });
    });

    ret.0
}
