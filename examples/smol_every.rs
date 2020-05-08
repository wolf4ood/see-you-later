use see_you_later::every;
use smol::{self, Task};
use std::time::Duration;
use wait_for_me::CountDownLatch;
#[smol_potat::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(10);
    let inner_latch = latch.clone();
    let (cancel, task) = every(Duration::from_millis(100), || async {
        inner_latch.count_down().await;
    });
    Task::spawn(async move {
        latch.wait().await;
        cancel.cancel().await
    })
    .detach();
    task.await;
    Ok(())
}
