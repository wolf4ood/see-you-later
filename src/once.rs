use crate::cancel::{CancelToken, CancelWaker};

use futures_timer::Delay;
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

/// Schedule a onceshot task after the [`Duration`][std::time::Duration] parameter.

/// Returns the scheduled task which needs to be awaited on and the cancel token.

pub fn once<T, C>(delay: Duration, task: C) -> (CancelToken, DelayedTask<T>)
where
    T: Future<Output = ()>,
    C: FnOnce() -> T,
{
    let done = AtomicBool::new(false);
    let waker = CancelWaker::new(done);
    (
        CancelToken::new(waker.clone()),
        DelayedTask {
            waker,
            delay: Delay::new(delay),
            task: task(),
        },
    )
}

pin_project! {
    /// A Future that represent a delayed task
    pub struct DelayedTask<T: Future<Output = ()>> {
        waker: CancelWaker,
        #[pin]
        delay: Delay,
        #[pin]
        task: T,
    }
}

impl<T: Future<Output = ()>> DelayedTask<T> {
    pub(crate) fn new(waker: CancelWaker, delay: Delay, task: T) -> DelayedTask<T> {
        DelayedTask {
            waker,
            delay: delay,
            task: task,
        }
    }
}

impl<T: Future<Output = ()>> Future for DelayedTask<T> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.waker.0.done.load(Ordering::Relaxed) {
            true => Poll::Ready(()),
            false => {
                this.waker.0.waker.register(cx.waker());
                if this.waker.0.done.load(Ordering::Relaxed) {
                    Poll::Ready(())
                } else {
                    match this.delay.poll(cx) {
                        Poll::Pending => Poll::Pending,
                        Poll::Ready(_) => match this.task.poll(cx) {
                            Poll::Ready(v) => Poll::Ready(v),
                            Poll::Pending => Poll::Pending,
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::once;
    use smol::{self, Task};
    use std::time::Duration;

    use futures::join;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    #[test]
    fn once_test() {
        let atomic = Arc::new(AtomicI32::new(0));
        let atomic_c = atomic.clone();
        smol::run(async {
            let (_, task) = once(Duration::from_secs(1), || async {
                atomic_c.store(1, Ordering::Relaxed);
            });
            task.await;
        });

        assert_eq!(1, atomic.load(Ordering::Relaxed));
    }

    #[test]
    fn once_cancel_test() {
        let atomic = Arc::new(AtomicI32::new(0));
        let atomic_c = atomic.clone();
        smol::run(async {
            let (token, task) = once(Duration::from_secs(1), || async move {
                atomic_c.store(1, Ordering::Relaxed);
            });
            let handle = Task::spawn(async move { task.await });
            join!(token.cancel(), handle)
        });
        assert_eq!(0, atomic.load(Ordering::Relaxed));
    }
}
