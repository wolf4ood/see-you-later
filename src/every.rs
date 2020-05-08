use crate::cancel::{CancelToken, CancelWaker};

use crate::DelayedTask;
use futures_timer::Delay;
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

pub fn every<T, C>(delay: Duration, task: C) -> (CancelToken, ScheduledTask<C, T>)
where
    T: Future<Output = ()>,
    C: Fn() -> T,
{
    let done = AtomicBool::new(false);
    let waker = CancelWaker::new(done);
    let token = CancelToken::new(waker.clone());

    let current = DelayedTask::new(waker.clone(), Delay::new(delay), task());
    let task = ScheduledTask {
        waker: waker,
        task,
        duration: delay,
        current: current,
    };
    (token, task)
}

pin_project! {
    pub struct ScheduledTask<C: Fn() -> T, T: Future<Output = ()>> {
        waker: CancelWaker,
        task: C,
        duration: Duration,
        #[pin]
        current: DelayedTask<T>,
    }
}

impl<C: Fn() -> T, T: Future<Output = ()>> Future for ScheduledTask<C, T> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        match this.waker.0.done.load(Ordering::Relaxed) {
            true => Poll::Ready(()),
            false => {
                this.waker.0.waker.register(cx.waker());
                if this.waker.0.done.load(Ordering::Relaxed) {
                    Poll::Ready(())
                } else {
                    loop {
                        match this.current.as_mut().poll(cx) {
                            Poll::Pending => return Poll::Pending,
                            Poll::Ready(_) => {
                                let current = DelayedTask::new(
                                    this.waker.clone(),
                                    Delay::new(this.duration.clone()),
                                    (this.task)(),
                                );
                                this.current.set(current);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::every;
    use smol::{self, Task, Timer};
    use std::time::Duration;

    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    #[test]
    fn every_test() {
        let atomic = Arc::new(AtomicI32::new(0));
        let atomic_c = atomic.clone();
        smol::run(async move {
            let atomic_inner = atomic_c.clone();
            let (cancel, task) = every(Duration::from_secs(1), || async {
                atomic_inner.fetch_add(1, Ordering::Relaxed);
            });
            Task::spawn(async move {
                Timer::after(Duration::from_secs(3)).await;
                cancel.cancel().await
            })
            .detach();
            task.await
        });

        assert!(atomic.load(Ordering::Relaxed) >= 2);
    }
}
