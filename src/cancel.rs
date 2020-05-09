use futures::task::AtomicWaker;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub(crate) struct CancelWakerInner {
    pub waker: AtomicWaker,
    pub done: AtomicBool,
}

#[derive(Clone)]
pub(crate) struct CancelWaker(pub Arc<CancelWakerInner>);

impl CancelWaker {
    pub fn new(done: AtomicBool) -> CancelWaker {
        CancelWaker(Arc::new(CancelWakerInner {
            done,
            waker: AtomicWaker::new(),
        }))
    }
}

/// A Cancel token which provide the ability to cancel a scheduled task.
pub struct CancelToken {
    waker: CancelWaker,
}

impl CancelToken {
    pub(crate) fn new(waker: CancelWaker) -> CancelToken {
        CancelToken { waker }
    }
}

impl CancelToken {

    /// Cancel the scheduled async task if running.
    pub async fn cancel(self) {
        self.waker.0.done.store(true, Ordering::Relaxed);
        self.waker.0.waker.wake();
    }
}
