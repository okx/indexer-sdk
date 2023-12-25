use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone)]
pub struct Channel<T> {
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T> Channel<T> {
    // 创建新的 Channel
    pub fn new() -> Self {
        let inner = Arc::new((Mutex::new(None), Condvar::new()));
        Channel { inner }
    }

    // 发送数据
    pub fn send(&self, data: T) {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();
        *guard = Some(data);
        cvar.notify_one();
    }

    // 接收数据
    pub fn recv(&self) -> Option<T> {
        let (lock, cvar) = &*self.inner;
        let mut guard = lock.lock().unwrap();

        // 等待直到有数据
        while guard.is_none() {
            guard = cvar.wait(guard).unwrap();
        }

        // 取出数据并清空
        let data = guard.take();
        cvar.notify_one();
        data
    }
}