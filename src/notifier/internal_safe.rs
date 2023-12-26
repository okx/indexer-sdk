// use std::sync::{Arc};
//
// #[repr(C)]
// #[derive(Clone, Default, Debug)]
// pub struct InternalSafeChannel<T> {
//     inner: Arc<(Mutex<Option<T>>, Condvar)>,
// }
//
// impl<T> InternalSafeChannel<T> {
//     pub fn new() -> Self {
//         let inner = Arc::new((Mutex::new(None), Condvar::new()));
//         InternalSafeChannel { inner }
//     }
//
//     pub fn send(&self, data: T) {
//         let (lock, cvar) = &*self.inner;
//         let mut guard = lock.lock();
//         *guard = Some(data);
//         cvar.notify_one();
//     }
//
//     pub fn recv(&self) -> Option<T> {
//         let (lock, cvar) = &*self.inner;
//         let mut guard = lock.lock();
//
//         while guard.is_none() {
//             cvar.wait(&mut guard);
//         }
//
//         let data = guard.take();
//         cvar.notify_one();
//         data
//     }
// }