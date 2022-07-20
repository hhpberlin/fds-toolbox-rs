use tokio::sync::Notify;
use std::sync::atomic::{Ordering, AtomicBool};

struct Event {
    waiters: Notify,
    state: AtomicBool,
}

impl Event {
    pub fn new() -> Self {
        Self {
            waiters: Notify::new(),
            state: AtomicBool::new(false),
        }
    }
    
    pub async fn wait(&self) {
        while !self.state.load(Ordering::Acquire) {
            self.waiters.notified().await;
        }
    }
    
    pub fn set(&self) {
        self.state.store(true, Ordering::Release);
        self.waiters.notify_waiters();
    }
}