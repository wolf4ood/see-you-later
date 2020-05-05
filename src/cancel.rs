use futures::task::AtomicWaker;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct CancelWakerInner {
    pub waker: AtomicWaker,
    pub done: AtomicBool,
}

#[derive(Clone)]
pub struct CancelWaker(pub Arc<CancelWakerInner>);

impl CancelWaker {
    pub fn new(done: AtomicBool) -> CancelWaker {
        CancelWaker(Arc::new(CancelWakerInner {
            done,
            waker: AtomicWaker::new(),
        }))
    }
}
pub struct CancelToken {
    waker: CancelWaker,
}

impl CancelToken {
    pub fn new(waker: CancelWaker) -> CancelToken {
        CancelToken { waker }
    }
}

impl CancelToken {
    pub async fn cancel(&self) {
        self.waker.0.done.store(true, Ordering::Relaxed);
        self.waker.0.waker.wake();
    }
}
