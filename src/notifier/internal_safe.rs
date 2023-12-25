use std::sync::{Arc};
use parking_lot::{Mutex, Condvar};

#[repr(C)]
#[derive(Clone, Default, Debug)]
pub struct InternalSafeChannel<T> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

// impl<T> InternalSafeChannel<T> {
//     // 创建新的 Channel
//     pub fn new() -> Self {
//         let inner = Arc::new((Mutex::new(None), Condvar::new()));
//         InternalSafeChannel { inner }
//     }
//
//     // 发送数据
//     pub fn send(&self, data: T) {
//         let (lock, cvar) = &*self.inner;
//         let mut guard = lock.lock().unwrap();
//         *guard = Some(data);
//         cvar.notify_one();
//     }
//
//     // 接收数据
//     pub fn recv(&self) -> Option<T> {
//         let (lock, cvar) = &*self.inner;
//         let mut guard = lock.lock().unwrap();
//
//         // 等待直到有数据
//         while guard.is_none() {
//             guard = cvar.wait(guard).unwrap();
//         }
//
//         // 取出数据并清空
//         let data = guard.take();
//         cvar.notify_one();
//         data
//     }
// }
impl<T> InternalSafeChannel<T> {
    pub fn new() -> Self {
        let inner = Arc::new((Mutex::new(None), Condvar::new()));
        InternalSafeChannel { inner }
    }

    pub fn send(&self, data: T) {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock();
        *guard = Some(data);
        cvar.notify_one();
    }

    pub fn recv(&self) -> Option<T> {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock();

        while guard.is_none() {
            cvar.wait(&mut guard);
        }

        let data = guard.take();
        cvar.notify_one();
        data
    }
}