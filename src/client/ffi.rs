use crate::client::common::CommonClient;
use crate::client::drect::DirectClient;
use crate::configuration::base::{IndexerConfiguration, NetConfiguration, ZMQConfiguration};
use crate::factory::common::sync_create_and_start_processor;
use crate::storage::db::level_db::LevelDB;
use crate::storage::kv::KVStorageProcessor;
use log::info;
use once_cell::sync::Lazy;
use std::ops::DerefMut;

static mut NOTIFIER: Lazy<Option<DirectClient<KVStorageProcessor<LevelDB>>>> = Lazy::new(|| None);

fn get_notifier() -> &'static mut DirectClient<KVStorageProcessor<LevelDB>> {
    unsafe {
        let ret = NOTIFIER.deref_mut();
        ret.as_mut().unwrap()
    }
}

fn get_option_notifier() -> &'static mut Option<DirectClient<KVStorageProcessor<LevelDB>>> {
    unsafe {
        let ret = NOTIFIER.deref_mut();
        ret
    }
}

#[repr(C)]
pub struct ByteArray {
    data: *const u8,
    length: usize,
}

#[no_mangle]
pub extern "C" fn start_processor() {
    let zmq_url = std::env::var("ZMQ_URL").unwrap();
    let zmq_topics = std::env::var("ZMQ_TOPIC").unwrap();
    info!("zmq_url: {}, zmq_topics: {}", zmq_url, zmq_topics);
    let zmq_topics: Vec<String> = zmq_topics.split(",").map(|v| v.to_string()).collect();
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .init();
    let ret = sync_create_and_start_processor(IndexerConfiguration {
        mq: ZMQConfiguration {
            zmq_url,
            zmq_topic: zmq_topics,
        },
        net: NetConfiguration::default(),
    });
    let old = get_option_notifier();
    *old = Some(ret);
}

#[no_mangle]
pub extern "C" fn get_data() -> ByteArray {
    let notifier = get_notifier();
    let binding = notifier.get();
    let ptr = binding.as_ptr();
    std::mem::forget(ptr);
    ByteArray {
        data: ptr,
        length: binding.len(),
    }
}

#[no_mangle]
pub extern "C" fn push_data(data: *const u8, len: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    println!("Received bytes: {:?}", bytes);
}
