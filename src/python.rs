use std::ops::DerefMut;
use log::info;
use once_cell::sync::Lazy;
use crate::configuration::base::{IndexerConfiguration, ZMQConfiguration};
use crate::factory::common::sync_create_and_start_processor;
use crate::notifier::common::CommonNotifier;

static mut NOTIFIER: Lazy<CommonNotifier> = Lazy::new(|| CommonNotifier::default());

fn get_notifier() -> &'static mut CommonNotifier {
    unsafe { NOTIFIER.deref_mut() }
}

#[repr(C)]
pub struct ByteArray {
    data: *const u8,
    length: usize,
}

#[no_mangle]
pub extern "C" fn create_processor() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_target(false)
        .init();
    let ret = sync_create_and_start_processor(IndexerConfiguration {
        mq: ZMQConfiguration {
            zmq_url: "tcp://0.0.0.0:5555".to_string(),
            zmq_topic: vec![],
        },
    });
    let old = get_notifier();
    *old = ret;
}

#[no_mangle]
pub extern "C" fn get_data() -> ByteArray {
    let notifier = get_notifier();
    let binding = notifier.get();
    info!("get data {:?},len:{:?}", &binding,binding.len());
    let ptr = binding.as_ptr();
    std::mem::forget(ptr);
    ByteArray { data: ptr, length: binding.len() }
}

