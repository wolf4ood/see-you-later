<h1 align="center">see-you-later</h1>
<div align="center">
  <strong>
    Delay and schedule async task
  </strong>
</div>

<br />

<div align="center">
  <a href="https://github.com/wolf4ood/see-you-later/actions?query=workflow%3ATests">
    <img src="https://github.com/wolf4ood/see-you-later/workflows/Tests/badge.svg"
    alt="Tests status" />
  </a>
  
  <a href="https://coveralls.io/github/wolf4ood/see-you-later?branch=master">
    <img src="https://coveralls.io/repos/github/wolf4ood/see-you-later/badge.svg?branch=master"
    alt="Coverage status" />
  </a>
  <a href="https://crates.io/crates/see-you-later">
    <img src="https://img.shields.io/crates/d/see-you-later.svg?style=flat-square"
      alt="Download" />
  </a>
  <a href="https://docs.rs/see-you-later">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  
</div>


# Install


Install from [crates.io](https://crates.io)


```
[dependencies]
see-you-later = "0.1"
```


# Example

with [smol](https://github.com/stjepang/smol) oneshot schedule

```rust
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

```


with [smol](https://github.com/stjepang/smol) periodic schedule.


```rust
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
```