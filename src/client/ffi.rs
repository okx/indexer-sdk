use crate::client::drect::DirectClient;
use crate::client::event::RequestEvent;
use crate::client::SyncClient;
use crate::configuration::base::{
    IndexerConfiguration, LogConfiguration, NetConfiguration, ZMQConfiguration,
};
use crate::event::IndexerEvent;
use crate::factory::common::sync_create_and_start_processor;
use crate::storage::db::memory::MemoryDB;
use crate::storage::kv::KVStorageProcessor;
use core::ffi::c_char;
use log::{info, warn};
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::ffi::CString;
use std::ops::DerefMut;

static mut NOTIFIER: Lazy<Option<DirectClient<KVStorageProcessor<MemoryDB>>>> = Lazy::new(|| None);

fn get_notifier() -> &'static mut DirectClient<KVStorageProcessor<MemoryDB>> {
    unsafe {
        let ret = NOTIFIER.deref_mut();
        ret.as_mut().unwrap()
    }
}

fn get_option_notifier() -> &'static mut Option<DirectClient<KVStorageProcessor<MemoryDB>>> {
    unsafe {
        let ret = NOTIFIER.deref_mut();
        ret
    }
}

#[repr(C, packed)]
pub struct ByteArray {
    data: *const u8,
    length: usize,
}

#[no_mangle]
pub extern "C" fn start_processor() {
    let zmq_url = std::env::var("ZMQ_URL").unwrap();
    let zmq_topics = std::env::var("ZMQ_TOPIC").unwrap();
    let db_path = std::env::var("DB_PATH")
        .map(|v| v.to_string())
        .unwrap_or("./indexer_db".to_string());
    let cache_block = std::env::var("CACHE_BLOCK")
        .unwrap_or("10".to_string())
        .parse::<u32>()
        .unwrap_or(10);
    let btc_rpc_url = std::env::var("BTC_RPC_URL").unwrap();
    let btc_rpc_username = std::env::var("BTC_RPC_USERNAME").unwrap();
    let btc_rpc_password = std::env::var("BTC_RPC_PASSWORD").unwrap();

    let log_level = std::env::var("LOG_LEVEL").unwrap_or("debug".to_string());
    let log_level = if log_level == "debug" {
        log::Level::Debug
    } else if log_level == "info" {
        log::Level::Info
    } else if log_level == "warn" {
        log::Level::Warn
    } else if log_level == "error" {
        log::Level::Error
    } else {
        log::Level::Info
    };

    info!("zmq_url: {}, zmq_topics: {}", zmq_url, zmq_topics);
    let zmq_topics: Vec<String> = zmq_topics.split(",").map(|v| v.to_string()).collect();
    let ret = sync_create_and_start_processor(IndexerConfiguration {
        mq: ZMQConfiguration {
            zmq_url,
            zmq_topic: zmq_topics,
        },
        net: NetConfiguration {
            url: btc_rpc_url,
            username: btc_rpc_username,
            password: btc_rpc_password,
        },
        db_path,
        save_block_cache_count: cache_block,
        log_configuration: LogConfiguration { log_level },
    });
    let old = get_option_notifier();
    *old = Some(ret);
}

#[no_mangle]
pub extern "C" fn get_event() -> ByteArray {
    let notifier = get_notifier();
    let binding = notifier.get().into_boxed_slice();
    let l = binding.len();
    let ptr = Box::into_raw(binding) as *const u8;
    std::mem::forget(ptr);
    ByteArray {
        data: ptr,
        length: l,
    }
}

// fn bytes_to_byte_array(data: &[u8]) -> ByteArray {
//     let ptr = data.as_ptr();
//     // let ptr = Box::into_raw(Box::new(data)) as *const u8;
//     std::mem::forget(ptr);
//     ByteArray {
//         data: ptr,
//         length: data.len(),
//     }
// }

#[no_mangle]
pub extern "C" fn js_get_event() -> *mut c_char {
    let notifier = get_notifier();
    let byte_data = notifier.get();
    let byte_data = hex::encode(&byte_data);
    let cstr = CString::new(byte_data).unwrap();
    let c_string_ptr = cstr.into_raw();
    return c_string_ptr;
}

#[no_mangle]
pub extern "C" fn free_bytes(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn push_event(data: *const u8, len: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let event = RequestEvent::from_bytes(bytes);
    info!("receive ffi event from indexer: {:?}", &event);
    let index_event: Option<IndexerEvent> = event.clone().into();
    if index_event.is_none() {
        warn!("receive unknown event: {:?}", &event);
        return;
    }
    let event = index_event.unwrap();
    let notifier = get_notifier();
    notifier.sync_push_event(event);
}
//
#[no_mangle]
pub extern "C" fn get_data(ptr: *mut c_char) -> *mut c_char {
    let str = unsafe {
        assert!(!ptr.is_null());
        CStr::from_ptr(ptr)
    }
    .to_str()
    .unwrap();
    let bytes = hex::decode(str).unwrap();
    let event = RequestEvent::from_bytes(&bytes);
    info!("receive ffi get data from indexer: {:?}", &event);
    let notifier = get_notifier();
    match event {
        RequestEvent::GetAllBalance(address) => {
            let ret = notifier.get_all_balance(address).unwrap();
            let ret = serde_json::to_vec(&ret).unwrap();
            return return_hex_str_point(&ret);
        }
        RequestEvent::GetBalance(address, token) => {
            let ret = notifier.get_balance(address, token).unwrap();
            let ret = &ret.to_bytes();
            return return_hex_str_point(&ret);
        }
        _ => {
            warn!("receive unknown event: {:?}", &event);
            return return_hex_str_point(&vec![]);
        }
    }
}

fn return_hex_str_point(data: &[u8]) -> *mut c_char {
    let byte_data = hex::encode(&data);
    let cstr = CString::new(byte_data).unwrap();
    let c_string_ptr = cstr.into_raw();
    c_string_ptr
}

#[test]
pub fn test_asd() {
    use crate::client::event::AddressTokenWrapper;
    let a = AddressTokenWrapper {
        address: Default::default(),
        token: Default::default(),
    };
    let mut a = serde_json::to_vec(&a).unwrap();
    a.push(1);
    let v = hex::encode(&a);
    println!("{}", v);
}
