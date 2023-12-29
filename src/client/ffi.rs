use crate::client::common::CommonClient;
use crate::client::drect::DirectClient;
use crate::client::event::{ClientEvent, RequestEvent};
use crate::client::SyncClient;
use crate::configuration::base::{IndexerConfiguration, NetConfiguration, ZMQConfiguration};
use crate::event::IndexerEvent;
use crate::factory::common::sync_create_and_start_processor;
use crate::storage::db::level_db::LevelDB;
use crate::storage::kv::KVStorageProcessor;
use log::{info, warn};
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
pub extern "C" fn get_event() -> ByteArray {
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
pub extern "C" fn push_event(data: *const u8, len: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let event = RequestEvent::from_bytes(bytes);
    info!("receive ffi event: {:?}", &event);
    let index_event: Option<IndexerEvent> = event.clone().into();
    if index_event.is_none() {
        warn!("receive unknown event: {:?}", &event);
        return;
    }
    let event = index_event.unwrap();
    let notifier = get_notifier();
    notifier.sync_push_event(event);
}

#[no_mangle]
pub extern "C" fn get_data(data: *const u8, len: usize) -> ByteArray {
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let event = RequestEvent::from_bytes(bytes);
    let notifier = get_notifier();
    match event {
        RequestEvent::GetAllBalance(address) => {
            let ret = notifier.get_all_balance(address).unwrap();
            let ret = serde_json::to_vec(&ret).unwrap();
            return bytes_to_byte_array(&ret);
        }
        RequestEvent::GetBalance(address, token) => {
            let ret = notifier.get_balance(address, token).unwrap();
            let ret = &ret.to_bytes();
            return bytes_to_byte_array(ret);
        }
        _ => {
            warn!("receive unknown event: {:?}", &event);
            return ByteArray {
                data: std::ptr::null(),
                length: 0,
            };
        }
    }
}

fn bytes_to_byte_array(data: &[u8]) -> ByteArray {
    let ptr = data.as_ptr();
    std::mem::forget(ptr);
    ByteArray {
        data: ptr,
        length: data.len(),
    }
}
