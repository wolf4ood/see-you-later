use see_you_later::once;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

#[smol_potat::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let invoked = Arc::new(AtomicBool::new(false));
    let invoked1 = invoked.clone();
    let (_, task) = once(Duration::from_secs(1), || async {
        invoked1.store(true, Ordering::Relaxed);
    });

    task.await;

    assert_eq!(true, invoked.load(Ordering::Relaxed));
    Ok(())
}
